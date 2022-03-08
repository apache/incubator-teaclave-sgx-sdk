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
use crate::sys::error::{FsError, FsResult};
use crate::sys::host::HostFile;
use crate::sys::keys::FsKeyGen;
use crate::sys::metadata::MetadataInfo;
use crate::sys::node::{FileNode, FileNodeRef};
use crate::sys::EncryptMode;
use sgx_types::error::errno::*;
use sgx_types::error::SgxStatus;
use sgx_types::types::{Key128bit, Mac128bit};
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

mod close;
mod flush;
mod node;
mod open;
mod other;
mod read;
mod write;

#[derive(Debug)]
pub struct ProtectedFile {
    file: Mutex<FileInner>,
}

const MAX_PAGES_IN_CACHE: usize = 48;

#[derive(Debug)]
struct FileInner {
    host_file: HostFile,
    metadata: MetadataInfo,
    root_mht: FileNodeRef,
    key_gen: FsKeyGen,
    opts: OpenOptions,
    need_writing: bool,
    end_of_file: bool,
    offset: usize,
    last_error: FsError,
    status: FileStatus,
    recovery_path: PathBuf,
    cache: LruCache<FileNode>,
}

impl ProtectedFile {
    pub fn open<P: AsRef<Path>>(path: P, opts: &OpenOptions, mode: &OpenMode) -> FsResult<Self> {
        let file = FileInner::open(path.as_ref(), opts, mode)?;
        Ok(Self {
            file: Mutex::new(file),
        })
    }

    pub fn write(&self, buf: &[u8]) -> FsResult<usize> {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.write(buf).map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> FsResult<usize> {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.write_at(buf, offset).map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn read(&self, buf: &mut [u8]) -> FsResult<usize> {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.read(buf).map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> FsResult<usize> {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.read_at(buf, offset).map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn tell(&self) -> FsResult<u64> {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.tell().map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn seek(&self, pos: SeekFrom) -> FsResult<u64> {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.seek(pos).map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn set_len(&self, size: u64) -> FsResult {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.set_len(size).map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn flush(&self) -> FsResult {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.flush().map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn file_size(&self) -> FsResult<u64> {
        let file = self
            .file
            .lock()
            .unwrap_or_else(|posion_error| posion_error.into_inner());
        file.file_size()
    }

    pub fn get_eof(&self) -> bool {
        let file = self
            .file
            .lock()
            .unwrap_or_else(|posion_error| posion_error.into_inner());
        file.get_eof()
    }

    pub fn get_error(&self) -> FsError {
        let file = self
            .file
            .lock()
            .unwrap_or_else(|posion_error| posion_error.into_inner());
        file.get_last_error()
    }

    pub fn clear_cache(&self) -> FsResult {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.clear_cache().map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn clear_error(&self) -> FsResult {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.clear_error().map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn get_metadata_mac(&self) -> FsResult<Mac128bit> {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.get_metadata_mac().map_err(|error| {
            file.set_last_error(error);
            error
        })
    }

    pub fn close(&self) -> FsResult {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.close(CloseMode::Normal).map(|_| ())
    }

    pub fn rename<P: AsRef<str>, Q: AsRef<str>>(&self, old_name: P, new_name: Q) -> FsResult {
        let mut file = self.file.lock().map_err(|posion_error| {
            let mut file = posion_error.into_inner();
            file.set_last_error(SgxStatus::Unexpected);
            file.set_file_status(FileStatus::MemoryCorrupted);
            SgxStatus::Unexpected
        })?;
        file.rename(old_name.as_ref(), new_name.as_ref())
            .map_err(|error| {
                file.set_last_error(error);
                error
            })
    }

    pub fn remove<P: AsRef<Path>>(path: P) -> FsResult {
        FileInner::remove(path.as_ref())
    }

    #[cfg(feature = "tfs")]
    pub fn export_key<P: AsRef<Path>>(path: P) -> FsResult<Key128bit> {
        let mut file = FileInner::open(
            path.as_ref(),
            &OpenOptions::new().read(true),
            &OpenMode::AutoKey,
        )?;
        file.close(CloseMode::Export).map(|key| key.unwrap())
    }

    #[cfg(feature = "tfs")]
    pub fn import_key<P: AsRef<Path>>(path: P, key: Key128bit) -> FsResult {
        let mut file = FileInner::open(
            path.as_ref(),
            &OpenOptions::new().read(true).update(true),
            &OpenMode::ImportKey(key),
        )?;
        file.close(CloseMode::Import).map(|_| ())
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileStatus {
    Ok,
    NotInitialized,
    FlushError,
    WriteToDiskFailed,
    CryptoError,
    Corrupted,
    MemoryCorrupted,
    Closed,
}

impl FileStatus {
    #[inline]
    pub fn is_ok(&self) -> bool {
        matches!(*self, FileStatus::Ok)
    }
}

impl Default for FileStatus {
    #[inline]
    fn default() -> Self {
        FileStatus::NotInitialized
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub binary: bool,
    pub update: bool,
}

#[allow(dead_code)]
impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            binary: false,
            update: false,
        }
    }

    #[inline]
    pub fn read(mut self, read: bool) -> Self {
        self.read = read;
        self
    }
    #[inline]
    pub fn write(mut self, write: bool) -> Self {
        self.write = write;
        self
    }
    #[inline]
    pub fn append(mut self, append: bool) -> Self {
        self.append = append;
        self
    }
    #[inline]
    pub fn update(mut self, update: bool) -> Self {
        self.update = update;
        self
    }
    #[inline]
    pub fn binary(mut self, binary: bool) -> Self {
        self.binary = binary;
        self
    }
    #[inline]
    pub fn readonly(&self) -> bool {
        self.read && !self.update
    }

    pub fn check_access_mode(&self) -> FsResult {
        match (self.read, self.write, self.append) {
            (true, false, false) => Ok(()),
            (false, true, false) => Ok(()),
            (false, false, true) => Ok(()),
            _ => Err(eos!(EINVAL)),
        }
    }
}

impl Default for OpenOptions {
    fn default() -> OpenOptions {
        OpenOptions::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenMode {
    AutoKey,
    UserKey(Key128bit),
    IntegrityOnly,
    ImportKey(Key128bit),
}

impl OpenMode {
    #[inline]
    pub fn is_auto_key(&self) -> bool {
        matches!(*self, Self::AutoKey)
    }

    #[inline]
    pub fn is_integrity_only(&self) -> bool {
        matches!(*self, Self::IntegrityOnly)
    }

    #[inline]
    pub fn user_key(&self) -> Option<&Key128bit> {
        match self {
            Self::UserKey(key) => Some(key),
            _ => None,
        }
    }

    #[inline]
    pub fn import_key(&self) -> Option<&Key128bit> {
        match self {
            Self::ImportKey(key) => Some(key),
            _ => None,
        }
    }
}

impl From<EncryptMode> for OpenMode {
    fn from(encrypt_mode: EncryptMode) -> OpenMode {
        match encrypt_mode {
            EncryptMode::EncryptAutoKey => Self::AutoKey,
            EncryptMode::EncryptWithIntegrity(key) => Self::UserKey(key),
            EncryptMode::IntegrityOnly => Self::IntegrityOnly,
        }
    }
}

impl From<&EncryptMode> for OpenMode {
    fn from(encrypt_mode: &EncryptMode) -> OpenMode {
        match encrypt_mode {
            EncryptMode::EncryptAutoKey => Self::AutoKey,
            EncryptMode::EncryptWithIntegrity(key) => Self::UserKey(*key),
            EncryptMode::IntegrityOnly => Self::IntegrityOnly,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CloseMode {
    Normal,
    Import,
    Export,
}
