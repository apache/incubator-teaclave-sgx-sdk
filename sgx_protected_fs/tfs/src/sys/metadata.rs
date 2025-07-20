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
// under the License.

use crate::sys::error::FsResult;
use crate::sys::file::OpenMode;
use crate::sys::host::HostFs;
use crate::sys::keys::{DeriveKey, KeyType, RestoreKey};
use crate::sys::node::{META_DATA_PHY_NUM, NODE_SIZE};
use sgx_crypto::aes::gcm::{Aad, AesGcm, Nonce};
use sgx_types::error::SgxStatus;
use sgx_types::types::{Attributes, CpuSvn, Key128bit, KeyId, KeyPolicy, Mac128bit};
use std::ffi::CStr;
use std::mem;

pub const SGX_FILE_ID: u64 = 0x5347_585F_4649_4C45;
pub const SGX_FILE_MAJOR_VERSION: u8 = 0x01;
pub const SGX_FILE_MINOR_VERSION: u8 = 0x00;

impl_enum! {
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum EncryptFlags {
        AutoKey = 0x00,
        UserKey = 0x01,
        IntegrityOnly = 0x02,
    }
}

impl EncryptFlags {
    #[inline]
    pub fn is_auto_key(&self) -> bool {
        matches!(*self, EncryptFlags::AutoKey)
    }

    #[inline]
    pub fn is_user_key(&self) -> bool {
        matches!(*self, EncryptFlags::UserKey)
    }

    #[inline]
    pub fn is_integrity_only(&self) -> bool {
        matches!(*self, EncryptFlags::IntegrityOnly)
    }
}

impl From<&OpenMode> for EncryptFlags {
    fn from(mode: &OpenMode) -> Self {
        match mode {
            OpenMode::AutoKey(_) | OpenMode::ExportKey | OpenMode::ImportKey(_) => Self::AutoKey,
            OpenMode::UserKey(_) => Self::UserKey,
            OpenMode::IntegrityOnly => Self::IntegrityOnly,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct MetadataPlain {
    pub file_id: u64,
    pub major_version: u8,
    pub minor_version: u8,
    pub encrypt_flags: EncryptFlags,
    pub update_flag: u8,
    pub key_policy: KeyPolicy,
    pub isv_svn: u16,
    pub key_id: KeyId,
    pub cpu_svn: CpuSvn,
    pub attribute_mask: Attributes,
    pub gmac: Mac128bit,
}

impl MetadataPlain {
    fn new() -> MetadataPlain {
        MetadataPlain {
            file_id: SGX_FILE_ID,
            major_version: SGX_FILE_MAJOR_VERSION,
            minor_version: SGX_FILE_MINOR_VERSION,
            ..Default::default()
        }
    }
}

pub const MD_USER_DATA_SIZE: usize = NODE_SIZE * 3 / 4; // 3072
pub const FILENAME_MAX_LEN: usize = 260;
pub const PATHNAME_MAX_LEN: usize = 512;
pub const FULLNAME_MAX_LEN: usize = PATHNAME_MAX_LEN + FILENAME_MAX_LEN;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct MetadataEncrypted {
    pub file_name: [u8; FILENAME_MAX_LEN],
    pub size: usize,
    pub mht_key: Key128bit,
    pub mht_gmac: Mac128bit,
    pub data: [u8; MD_USER_DATA_SIZE],
}

const METADATA_PADDING: usize =
    NODE_SIZE - mem::size_of::<MetadataPlain>() - mem::size_of::<MetadataEncrypted>();
const METADATA_ENCRYPTED: usize = mem::size_of::<MetadataEncrypted>();

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Metadata {
    pub plaintext: MetadataPlain,
    pub ciphertext: [u8; METADATA_ENCRYPTED],
    pub padding: [u8; METADATA_PADDING],
}

impl Metadata {
    fn new() -> Metadata {
        Metadata {
            plaintext: MetadataPlain::new(),
            ciphertext: [0_u8; METADATA_ENCRYPTED],
            padding: [0_u8; METADATA_PADDING],
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct MetadataNode {
    pub node_number: u64,
    pub metadata: Metadata,
}

impl MetadataNode {
    fn new() -> MetadataNode {
        MetadataNode {
            node_number: META_DATA_PHY_NUM,
            metadata: Metadata::new(),
        }
    }
}

impl_struct_default! {
    Metadata;
    MetadataEncrypted;
}

impl_asref_slice! {
    MetadataPlain;
    MetadataEncrypted;
    Metadata;
    MetadataNode;
}

impl_asmut_slice! {
    MetadataPlain;
    MetadataEncrypted;
    Metadata;
    MetadataNode;
}

impl_struct_ContiguousMemory! {
    MetadataPlain;
    MetadataEncrypted;
    Metadata;
    MetadataNode;
}

#[derive(Clone, Debug, Default)]
pub struct MetadataInfo {
    pub node: MetadataNode,
    pub encrypted_plain: MetadataEncrypted,
}

impl MetadataInfo {
    pub fn new() -> MetadataInfo {
        MetadataInfo {
            node: MetadataNode::new(),
            encrypted_plain: MetadataEncrypted::default(),
        }
    }

    #[inline]
    pub fn integrity_only(&self) -> bool {
        self.encrypt_flags().is_integrity_only()
    }

    #[inline]
    pub fn encrypt_flags(&self) -> EncryptFlags {
        self.node.metadata.plaintext.encrypt_flags
    }

    #[inline]
    pub fn set_encrypt_flags(&mut self, encrypt_flags: EncryptFlags) {
        self.node.metadata.plaintext.encrypt_flags = encrypt_flags;
    }

    #[inline]
    pub fn key_policy(&self) -> KeyPolicy {
        self.node.metadata.plaintext.key_policy
    }

    #[inline]
    pub fn set_key_policy(&mut self, key_policy: KeyPolicy) {
        self.node.metadata.plaintext.key_policy = key_policy;
    }

    #[inline]
    pub fn update_flag(&self) -> bool {
        self.node.metadata.plaintext.update_flag != 0
    }

    #[inline]
    pub fn set_update_flag(&mut self, flag: u8) {
        self.node.metadata.plaintext.update_flag = flag;
    }

    #[inline]
    pub fn file_name(&self) -> FsResult<&str> {
        let len = self
            .encrypted_plain
            .file_name
            .iter()
            .enumerate()
            .find(|x| *x.1 == 0)
            .map(|x| x.0 + 1)
            .ok_or(SgxStatus::Unexpected)?;
        let name = CStr::from_bytes_with_nul(&self.encrypted_plain.file_name[0..len])
            .map_err(|_| SgxStatus::Unexpected)?
            .to_str()
            .map_err(|_| SgxStatus::Unexpected)?;
        Ok(name)
    }

    pub fn encrypt(&mut self, key: &Key128bit) -> FsResult {
        let mac = if !self.integrity_only() {
            let mut aes = AesGcm::new(key, Nonce::zeroed(), Aad::empty())?;
            aes.encrypt(
                self.encrypted_plain.as_ref(),
                self.node.metadata.ciphertext.as_mut(),
            )?
        } else {
            let mut aes = AesGcm::new(key, Nonce::zeroed(), Aad::from(&self.encrypted_plain))?;
            let mac = aes.mac()?;
            self.node
                .metadata
                .ciphertext
                .as_mut()
                .copy_from_slice(self.encrypted_plain.as_ref());
            mac
        };

        self.node.metadata.plaintext.gmac = mac;

        Ok(())
    }

    pub fn decrypt(&mut self, key: &Key128bit) -> FsResult {
        if !self.integrity_only() {
            let mut aes = AesGcm::new(key, Nonce::zeroed(), Aad::empty())?;
            aes.decrypt(
                self.node.metadata.ciphertext.as_ref(),
                self.encrypted_plain.as_mut(),
                &self.node.metadata.plaintext.gmac,
            )?
        } else {
            let mut aes = AesGcm::new(
                key,
                Nonce::zeroed(),
                Aad::from(&self.node.metadata.ciphertext),
            )?;
            aes.verify_mac(&self.node.metadata.plaintext.gmac)?;
            self.encrypted_plain
                .as_mut()
                .copy_from_slice(self.node.metadata.ciphertext.as_ref());
        };

        Ok(())
    }

    pub fn derive_key(&mut self, derive: &mut dyn DeriveKey) -> FsResult<Key128bit> {
        let (key, key_id) = derive.derive_key(KeyType::Metadata, 0)?;
        match self.encrypt_flags() {
            EncryptFlags::AutoKey => {
                cfg_if! {
                    if #[cfg(feature = "tfs")] {
                        use sgx_tse::EnclaveReport;
                        use sgx_types::types::Report;

                        self.node.metadata.plaintext.key_id = key_id;

                        let report = Report::get_self();
                        self.node.metadata.plaintext.cpu_svn = report.body.cpu_svn;
                        self.node.metadata.plaintext.isv_svn = report.body.isv_svn;
                    } else {
                        use sgx_types::error::errno::ENOTSUP;
                        bail!(eos!(ENOTSUP));
                    }
                }
            }
            EncryptFlags::UserKey => {
                self.node.metadata.plaintext.key_id = key_id;
            }
            EncryptFlags::IntegrityOnly => {}
        }
        Ok(key)
    }

    pub fn restore_key(&self, restore: &dyn RestoreKey) -> FsResult<Key128bit> {
        match self.encrypt_flags() {
            EncryptFlags::AutoKey => {
                cfg_if! {
                    if #[cfg(feature = "tfs")] {
                        restore.restore_key(
                            KeyType::Metadata,
                            self.node.metadata.plaintext.key_id,
                            Some(self.node.metadata.plaintext.key_policy),
                            Some(self.node.metadata.plaintext.cpu_svn),
                            Some(self.node.metadata.plaintext.isv_svn),
                        )
                    } else {
                        use sgx_types::error::errno::ENOTSUP;
                        bail!(eos!(ENOTSUP));
                    }
                }
            }
            EncryptFlags::UserKey | EncryptFlags::IntegrityOnly => restore.restore_key(
                KeyType::Metadata,
                self.node.metadata.plaintext.key_id,
                None,
                None,
                None,
            ),
        }
    }

    #[inline]
    pub fn read_from_disk(&mut self, file: &mut dyn HostFs) -> FsResult {
        file.read(META_DATA_PHY_NUM, &mut self.node.metadata)
    }

    #[inline]
    pub fn write_to_disk(&mut self, file: &mut dyn HostFs) -> FsResult {
        file.write(META_DATA_PHY_NUM, &self.node.metadata)
    }

    #[inline]
    pub fn write_recovery_file(&self, file: &mut dyn HostFs) -> FsResult {
        file.write(META_DATA_PHY_NUM, &self.node)
    }
}

impl Drop for MetadataInfo {
    fn drop(&mut self) {
        self.encrypted_plain.as_mut().fill(0)
    }
}
