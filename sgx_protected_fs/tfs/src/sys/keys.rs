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
use sgx_crypto::mac::AesCMac;
use sgx_rand::{RdRand, Rng};
#[cfg(feature = "tfs")]
use sgx_tse::{EnclaveKey, EnclaveReport};
use sgx_types::error::errno::*;
use sgx_types::marker::ContiguousMemory;
#[cfg(feature = "tfs")]
use sgx_types::types::{
    Attributes, AttributesFlags, KeyName, KeyRequest, Report, TSEAL_DEFAULT_MISCMASK,
};
use sgx_types::types::{CpuSvn, Key128bit, KeyId, KeyPolicy};
#[cfg(feature = "tfs")]
use std::boxed::Box;

pub trait DeriveKey {
    fn derive_key(&mut self, key_type: KeyType, node_number: u64) -> FsResult<(Key128bit, KeyId)>;
}

pub trait RestoreKey {
    fn restore_key(
        &self,
        key_type: KeyType,
        key_id: KeyId,
        key_policy: Option<KeyPolicy>,
        cpu_svn: Option<CpuSvn>,
        isv_svn: Option<u16>,
    ) -> FsResult<Key128bit>;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeyType {
    Metadata,
    Master,
    Random,
}

#[derive(Clone, Debug, Default)]
pub struct MasterKey {
    key: Key128bit,
    key_id: KeyId,
    count: u32,
}

impl MasterKey {
    fn new() -> FsResult<MasterKey> {
        let (key, key_id) = KdfInput::derive_key(&Key128bit::default(), KeyType::Master, 0)?;
        Ok(MasterKey {
            key,
            key_id,
            count: 0,
        })
    }

    fn update(&mut self) -> FsResult<(Key128bit, KeyId)> {
        const MAX_USAGES: u32 = 65536;

        if self.count >= MAX_USAGES {
            *self = Self::new()?;
        } else {
            self.count += 1;
        }
        Ok((self.key, self.key_id))
    }
}

impl DeriveKey for MasterKey {
    fn derive_key(&mut self, key_type: KeyType, node_number: u64) -> FsResult<(Key128bit, KeyId)> {
        match key_type {
            KeyType::Master => self.update(),
            KeyType::Random => {
                let (key, _) = self.update()?;
                KdfInput::derive_key(&key, KeyType::Random, node_number)
            }
            _ => Err(eos!(ENOTSUP)),
        }
    }
}

impl RestoreKey for MasterKey {
    fn restore_key(
        &self,
        _key_type: KeyType,
        _key_id: KeyId,
        _key_policy: Option<KeyPolicy>,
        _cpu_svn: Option<CpuSvn>,
        _isv_svn: Option<u16>,
    ) -> FsResult<Key128bit> {
        Err(eos!(ENOTSUP))
    }
}

impl Drop for MasterKey {
    fn drop(&mut self) {
        self.count = 0;
        self.key.fill(0)
    }
}

#[derive(Clone, Debug)]
pub enum MetadataKey {
    UserKey(Key128bit),
    #[cfg(feature = "tfs")]
    AutoKey {
        report: Box<Report>,
        key_policy: Option<KeyPolicy>,
    },
}

impl MetadataKey {
    #[allow(unused_variables)]
    fn new(user_key: Option<Key128bit>, key_policy: Option<KeyPolicy>) -> FsResult<MetadataKey> {
        if let Some(user_key) = user_key {
            Ok(Self::UserKey(user_key))
        } else {
            cfg_if! {
                if #[cfg(feature = "tfs")] {
                    Ok(Self::AutoKey {
                        report: Box::new(*Report::get_self()),
                        key_policy,
                    })
                } else {
                    Err(eos!(ENOTSUP))
                }
            }
        }
    }
}

impl DeriveKey for MetadataKey {
    fn derive_key(&mut self, key_type: KeyType, _node_number: u64) -> FsResult<(Key128bit, KeyId)> {
        ensure!(key_type == KeyType::Metadata, eos!(EINVAL));

        match self {
            Self::UserKey(ref user_key) => KdfInput::derive_key(user_key, KeyType::Metadata, 0),
            #[cfg(feature = "tfs")]
            Self::AutoKey {
                ref report,
                ref key_policy,
            } => {
                let mut rng = RdRand::new().map_err(|_| ENOTSUP)?;
                let mut key_id = KeyId::default();
                rng.fill_bytes(key_id.as_mut());

                let key_request = KeyRequest {
                    key_name: KeyName::Seal,
                    key_policy: key_policy.unwrap_or(KeyPolicy::MRSIGNER),
                    isv_svn: report.body.isv_svn,
                    cpu_svn: report.body.cpu_svn,
                    attribute_mask: Attributes {
                        flags: AttributesFlags::DEFAULT_MASK,
                        xfrm: 0,
                    },
                    key_id,
                    misc_mask: TSEAL_DEFAULT_MISCMASK,
                    ..Default::default()
                };
                let key = key_request.get_key()?;
                Ok((key, key_id))
            }
        }
    }
}

impl RestoreKey for MetadataKey {
    #[allow(unused_variables)]
    fn restore_key(
        &self,
        key_type: KeyType,
        key_id: KeyId,
        key_policy: Option<KeyPolicy>,
        cpu_svn: Option<CpuSvn>,
        isv_svn: Option<u16>,
    ) -> FsResult<Key128bit> {
        ensure!(key_type == KeyType::Metadata, eos!(EINVAL));

        match self {
            Self::UserKey(ref user_key) => {
                KdfInput::restore_key(user_key, KeyType::Metadata, 0, key_id)
            }
            #[cfg(feature = "tfs")]
            Self::AutoKey { .. } => {
                let cpu_svn = cpu_svn.ok_or(EINVAL)?;
                let isv_svn = isv_svn.ok_or(EINVAL)?;

                let key_request = KeyRequest {
                    key_name: KeyName::Seal,
                    key_policy: key_policy.unwrap_or(KeyPolicy::MRSIGNER),
                    isv_svn,
                    cpu_svn,
                    attribute_mask: Attributes {
                        flags: AttributesFlags::DEFAULT_MASK,
                        xfrm: 0,
                    },
                    key_id,
                    misc_mask: TSEAL_DEFAULT_MISCMASK,
                    ..Default::default()
                };
                let key = key_request.get_key()?;
                Ok(key)
            }
        }
    }
}

impl Drop for MetadataKey {
    fn drop(&mut self) {
        match self {
            Self::UserKey(ref mut key) => key.fill(0),
            #[cfg(feature = "tfs")]
            Self::AutoKey { .. } => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct KdfInput {
    index: u32,
    label: [u8; 64],
    _pad1: u32,
    node_number: u64,
    nonce: KeyId,
    output_len: u32,
    _pad2: u32,
}

impl_struct_default! {
    KdfInput;
}

unsafe impl ContiguousMemory for KdfInput {}

impl KdfInput {
    const MASTER_KEY_NAME: &'static str = "SGX-PROTECTED-FS-MASTER-KEY";
    const RANDOM_KEY_NAME: &'static str = "SGX-PROTECTED-FS-RANDOM-KEY";
    const METADATA_KEY_NAME: &'static str = "SGX-PROTECTED-FS-METADATA-KEY";

    fn derive_key(
        key: &Key128bit,
        key_type: KeyType,
        node_number: u64,
    ) -> FsResult<(Key128bit, KeyId)> {
        let mut rng = RdRand::new().map_err(|_| ENOTSUP)?;
        let label = match key_type {
            KeyType::Metadata => Self::METADATA_KEY_NAME,
            KeyType::Master => Self::MASTER_KEY_NAME,
            KeyType::Random => Self::RANDOM_KEY_NAME,
        };

        let mut kdf = KdfInput {
            index: 0x01,
            output_len: 0x80,
            node_number,
            ..Default::default()
        };
        kdf.label[0..label.len()].copy_from_slice(label.as_bytes());
        rng.fill_bytes(kdf.nonce.as_mut());

        let key = AesCMac::cmac(key, &kdf)?;
        Ok((key, kdf.nonce))
    }

    fn restore_key(
        key: &Key128bit,
        key_type: KeyType,
        node_number: u64,
        key_id: KeyId,
    ) -> FsResult<Key128bit> {
        let label = match key_type {
            KeyType::Metadata => Self::METADATA_KEY_NAME,
            KeyType::Master => Self::MASTER_KEY_NAME,
            KeyType::Random => Self::RANDOM_KEY_NAME,
        };

        let mut kdf = KdfInput {
            index: 0x01,
            output_len: 0x80,
            node_number,
            nonce: key_id,
            ..Default::default()
        };
        kdf.label[0..label.len()].copy_from_slice(label.as_bytes());

        let key = AesCMac::cmac(key, &kdf)?;
        Ok(key)
    }
}

#[derive(Clone, Debug)]
pub enum FsKeyGen {
    EncryptWithIntegrity(MetadataKey, MasterKey),
    IntegrityOnly,
    Import(MetadataKey),
    Export(MetadataKey),
}

impl FsKeyGen {
    pub fn new(mode: &OpenMode) -> FsResult<FsKeyGen> {
        match mode {
            OpenMode::AutoKey(key_policy) => Ok(Self::EncryptWithIntegrity(
                MetadataKey::new(None, Some(*key_policy))?,
                MasterKey::new()?,
            )),
            OpenMode::UserKey(user_key) => Ok(Self::EncryptWithIntegrity(
                MetadataKey::new(Some(*user_key), None)?,
                MasterKey::new()?,
            )),
            OpenMode::IntegrityOnly => Ok(Self::IntegrityOnly),
            OpenMode::ImportKey((_, key_policy)) => {
                Ok(Self::Import(MetadataKey::new(None, Some(*key_policy))?))
            }
            OpenMode::ExportKey => Ok(Self::Export(MetadataKey::new(None, None)?)),
        }
    }
}

impl DeriveKey for FsKeyGen {
    fn derive_key(&mut self, key_type: KeyType, node_number: u64) -> FsResult<(Key128bit, KeyId)> {
        match self {
            Self::EncryptWithIntegrity(metadata_key, master_key) => match key_type {
                KeyType::Metadata => metadata_key.derive_key(KeyType::Metadata, 0),
                KeyType::Master => master_key.derive_key(KeyType::Master, 0),
                KeyType::Random => master_key.derive_key(KeyType::Random, node_number),
            },
            Self::IntegrityOnly => Ok((Key128bit::default(), KeyId::default())),
            Self::Import(metadata_key) => {
                ensure!(key_type == KeyType::Metadata, eos!(EINVAL));
                metadata_key.derive_key(KeyType::Metadata, 0)
            }
            Self::Export(_) => Err(eos!(EINVAL)),
        }
    }
}

impl RestoreKey for FsKeyGen {
    fn restore_key(
        &self,
        key_type: KeyType,
        key_id: KeyId,
        key_policy: Option<KeyPolicy>,
        cpu_svn: Option<CpuSvn>,
        isv_svn: Option<u16>,
    ) -> FsResult<Key128bit> {
        match self {
            Self::EncryptWithIntegrity(metadata_key, _) => match key_type {
                KeyType::Metadata => {
                    metadata_key.restore_key(key_type, key_id, key_policy, cpu_svn, isv_svn)
                }
                KeyType::Master | KeyType::Random => Err(eos!(EINVAL)),
            },
            Self::IntegrityOnly => Ok(Key128bit::default()),
            Self::Import(_) => Err(eos!(EINVAL)),
            Self::Export(metadata_key) => match key_type {
                KeyType::Metadata => {
                    metadata_key.restore_key(key_type, key_id, key_policy, cpu_svn, isv_svn)
                }
                KeyType::Master | KeyType::Random => Err(eos!(EINVAL)),
            },
        }
    }
}
