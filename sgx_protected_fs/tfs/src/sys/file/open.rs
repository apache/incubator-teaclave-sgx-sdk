// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

use crate::sys::cache::LruCache;
use crate::sys::error::FsResult;
use crate::sys::file::{FileInner, FileStatus, OpenMode, OpenOptions};
use crate::sys::host::{self, HostFile, HostFs};
use crate::sys::keys::{FsKeyGen, RestoreKey};
use crate::sys::metadata::MetadataInfo;
use crate::sys::metadata::{
    FILENAME_MAX_LEN, FULLNAME_MAX_LEN, MD_USER_DATA_SIZE, SGX_FILE_ID, SGX_FILE_MAJOR_VERSION,
};
use crate::sys::node::{FileNode, FileNodeRef};
use sgx_types::error::errno::*;
use sgx_types::error::SgxStatus;
use sgx_types::memeq::ConstTimeEq;
use sgx_types::metadata::SE_PAGE_SIZE;
use sgx_types::types::Key128bit;
use std::borrow::ToOwned;
use std::path::Path;

macro_rules! is_page_aligned {
    ($num:expr) => {
        $num & (SE_PAGE_SIZE - 1) == 0
    };
}

pub const DEFAULT_CACHE_SIZE: usize = 48 * SE_PAGE_SIZE;

impl FileInner {
    pub fn open(
        path: &Path,
        opts: &OpenOptions,
        mode: &OpenMode,
        cache_size: Option<usize>,
    ) -> FsResult<Self> {
        let cache_size = Self::check_cache_size(cache_size)?;
        let file_name = path.file_name().ok_or(EINVAL)?.to_str().ok_or(EINVAL)?;
        Self::check_open_param(path, file_name, opts, mode)?;

        let key_gen = FsKeyGen::new(mode.user_key().cloned())?;

        Self::check_file_exist(opts, mode, path)?;

        let mut host_file = HostFile::open(path, opts.readonly())?;
        let file_size = host_file.size();

        let mut recovery_file_name = file_name.to_owned();
        recovery_file_name.push_str("_recovery");
        let recovery_path = path.with_file_name(recovery_file_name);

        let mut need_writing = false;
        let mut offset = 0;
        let (host_file, metadata, root_mht) = if file_size > 0 {
            // existing file
            ensure!(!opts.write, eos!(EACCES));

            let (host_file, metadata, root_mht) =
                match Self::open_file(&mut host_file, file_name, &key_gen, mode) {
                    Ok((metadata, root_mht)) => (host_file, metadata, root_mht),
                    Err(e) if e.equal_to_sgx_error(SgxStatus::RecoveryNeeded) => {
                        let mut host_file =
                            Self::recover_and_reopen_file(host_file, path, &recovery_path, opts)?;

                        let (metadata, root_mht) =
                            Self::open_file(&mut host_file, file_name, &key_gen, mode)?;
                        (host_file, metadata, root_mht)
                    }
                    Err(e) => bail!(e),
                };

            if opts.append && !opts.update {
                offset = metadata.encrypted_plain.size;
            }
            (host_file, metadata, root_mht)
        } else {
            let metadata = Self::new_file(file_name, mode)?;
            need_writing = true;
            (host_file, metadata, FileNode::new_root_ref(mode.into()))
        };

        let mut protected_file = Self {
            host_file,
            metadata,
            root_mht,
            key_gen,
            opts: *opts,
            need_writing,
            end_of_file: false,
            max_cache_page: cache_size,
            offset,
            last_error: esgx!(SgxStatus::Success),
            status: FileStatus::NotInitialized,
            recovery_path,
            cache: LruCache::new(cache_size),
        };

        protected_file.status = FileStatus::Ok;
        Ok(protected_file)
    }

    fn open_file(
        host_file: &mut dyn HostFs,
        file_name: &str,
        key_gen: &dyn RestoreKey,
        mode: &OpenMode,
    ) -> FsResult<(MetadataInfo, FileNodeRef)> {
        let mut metadata = MetadataInfo::default();
        metadata.read_from_disk(host_file)?;

        ensure!(
            metadata.node.metadata.plaintext.file_id == SGX_FILE_ID,
            esgx!(SgxStatus::NotSgxFile)
        );
        ensure!(
            metadata.node.metadata.plaintext.major_version == SGX_FILE_MAJOR_VERSION,
            eos!(ENOTSUP)
        );
        ensure!(!metadata.update_flag(), esgx!(SgxStatus::RecoveryNeeded));

        let encrypt_flags = mode.into();
        ensure!(encrypt_flags == metadata.encrypt_flags(), eos!(EINVAL));

        let key = mode
            .import_key()
            .cloned()
            .unwrap_or(metadata.restore_key(key_gen)?);
        metadata.decrypt(&key)?;

        let meta_file_name = metadata.file_name()?;
        ensure!(meta_file_name == file_name, esgx!(SgxStatus::NameMismatch));

        let mut root_mht = FileNode::new_root(encrypt_flags);
        if metadata.encrypted_plain.size > MD_USER_DATA_SIZE {
            root_mht.read_from_disk(host_file)?;
            root_mht.decrypt(
                &metadata.encrypted_plain.mht_key,
                &metadata.encrypted_plain.mht_gmac,
            )?;
            root_mht.new_node = false;
        }
        Ok((metadata, FileNode::build_ref(root_mht)))
    }

    #[inline]
    fn new_file(file_name: &str, mode: &OpenMode) -> FsResult<MetadataInfo> {
        let mut metadata = MetadataInfo::new();

        metadata.set_encrypt_flags(mode.into());
        metadata.encrypted_plain.file_name[0..file_name.len()]
            .copy_from_slice(file_name.as_bytes());

        Ok(metadata)
    }

    #[inline]
    fn recover_and_reopen_file(
        host_file: HostFile,
        path: &Path,
        recovery_path: &Path,
        opts: &OpenOptions,
    ) -> FsResult<HostFile> {
        let file_size = host_file.size();
        drop(host_file);

        host::recovery(path, recovery_path)?;
        let host_file = HostFile::open(path, opts.readonly())?;
        ensure!(host_file.size() == file_size, esgx!(SgxStatus::Unexpected));
        Ok(host_file)
    }

    fn check_file_exist(opts: &OpenOptions, mode: &OpenMode, path: &Path) -> FsResult {
        let is_exist = host::try_exists(path)?;

        if opts.read || mode.import_key().is_some() {
            ensure!(is_exist, eos!(ENOENT));
        }
        if opts.write && is_exist {
            // try to delete existing file
            host::remove(path)?;
            // re-check
            let is_exist = host::try_exists(path)?;
            ensure!(!is_exist, eos!(EACCES));
        }

        Ok(())
    }

    #[inline]
    fn check_open_param(path: &Path, name: &str, opts: &OpenOptions, mode: &OpenMode) -> FsResult {
        let path_len = path.to_str().ok_or(EINVAL)?.len();
        ensure!(
            (path_len > 0 && path_len < FULLNAME_MAX_LEN - 1),
            eos!(EINVAL)
        );

        let name_len = name.len();
        ensure!(name_len > 0, eos!(EINVAL));
        ensure!(name_len < FILENAME_MAX_LEN - 1, eos!(ENAMETOOLONG));

        opts.check_access_mode()?;

        if let Some(key) = mode.import_key() {
            ensure!(key.ct_ne(&Key128bit::default()), eos!(EINVAL));
        }
        Ok(())
    }

    #[inline]
    fn check_cache_size(cache_size: Option<usize>) -> FsResult<usize> {
        cache_size
            .or(Some(DEFAULT_CACHE_SIZE))
            .and_then(|cache_size| {
                if is_page_aligned!(cache_size) && cache_size >= DEFAULT_CACHE_SIZE {
                    Some(cache_size / SE_PAGE_SIZE)
                } else {
                    None
                }
            })
            .ok_or_else(|| eos!(EINVAL))
    }
}
