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

use crate::sys::error::FsError;
use crate::sys::file::{self as file_imp, ProtectedFile};
#[cfg(feature = "tfs")]
use sgx_types::types::KeyPolicy;
use sgx_types::types::{Key128bit, Mac128bit};
use std::boxed::Box;
use std::io::{Result, SeekFrom};
use std::mem::ManuallyDrop;
use std::path::Path;

pub use file::DEFAULT_CACHE_SIZE;

#[macro_use]
pub(crate) mod error;
#[macro_use]
mod node;

mod cache;
mod file;
mod host;
mod keys;
mod metadata;

#[derive(Clone, Debug)]
pub struct OpenOptions(file_imp::OpenOptions);

#[derive(Clone, Debug)]
pub enum EncryptMode {
    #[cfg(feature = "tfs")]
    EncryptAutoKey(KeyPolicy),
    EncryptUserKey(Key128bit),
    IntegrityOnly,
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        Self(file_imp::OpenOptions::new())
    }
    #[inline]
    pub fn read(&mut self, read: bool) {
        self.0.read = read;
    }
    #[inline]
    pub fn write(&mut self, write: bool) {
        self.0.write = write;
    }
    #[inline]
    pub fn append(&mut self, append: bool) {
        self.0.append = append;
    }
    #[inline]
    pub fn update(&mut self, update: bool) {
        self.0.update = update;
    }
    #[inline]
    pub fn binary(&mut self, binary: bool) {
        self.0.binary = binary;
    }

    #[allow(dead_code)]
    pub fn check(&self) -> Result<()> {
        self.0.check().map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }
}

impl Default for OpenOptions {
    fn default() -> OpenOptions {
        Self::new()
    }
}

#[derive(Debug)]
pub struct SgxFile {
    file: Box<ProtectedFile>,
}

impl SgxFile {
    pub fn open<P: AsRef<Path>>(
        path: P,
        opts: &OpenOptions,
        encrypt_mode: &EncryptMode,
        cache_size: Option<usize>,
    ) -> Result<SgxFile> {
        ProtectedFile::open(path, &opts.0, &encrypt_mode.into(), cache_size)
            .map_err(|e| {
                e.set_errno();
                e.to_io_error()
            })
            .map(|f| SgxFile { file: Box::new(f) })
    }

    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        self.file.read(buf).map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        self.file.read_at(buf, offset).map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf).map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        self.file.write_at(buf, offset).map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn tell(&self) -> Result<u64> {
        self.file.tell().map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn seek(&self, pos: SeekFrom) -> Result<u64> {
        self.file.seek(pos).map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn set_len(&self, size: u64) -> Result<()> {
        self.file.set_len(size).map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn flush(&self) -> Result<()> {
        self.file.flush().map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn file_size(&self) -> Result<u64> {
        self.file.file_size().map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.file.get_eof()
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_error(&self) -> FsError {
        self.file.get_error()
    }

    #[inline]
    pub fn clear_cache(&self) -> Result<()> {
        self.file.clear_cache().map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn clear_error(&self) -> Result<()> {
        self.file.clear_error().map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn get_mac(&self) -> Result<Mac128bit> {
        self.file.get_metadata_mac().map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }

    #[inline]
    pub fn rename<P: AsRef<str>, Q: AsRef<str>>(&self, old_name: P, new_name: Q) -> Result<()> {
        self.file.rename(old_name, new_name).map_err(|e| {
            e.set_errno();
            e.to_io_error()
        })
    }
}

#[allow(dead_code)]
pub type RawProtectedFile = *const std::ffi::c_void;

#[allow(dead_code)]
impl SgxFile {
    pub fn into_raw(self) -> RawProtectedFile {
        let file = ManuallyDrop::new(self);
        file.file.as_ref() as *const _ as RawProtectedFile
    }

    /// # Safety
    pub unsafe fn from_raw(raw: RawProtectedFile) -> Self {
        let file = Box::from_raw(raw as *mut ProtectedFile);
        Self { file }
    }
}

impl Drop for SgxFile {
    fn drop(&mut self) {
        let _ = self.file.close();
    }
}

#[inline]
pub fn remove<P: AsRef<Path>>(path: P) -> Result<()> {
    ProtectedFile::remove(path).map_err(|e| {
        e.set_errno();
        e.to_io_error()
    })
}

#[cfg(feature = "tfs")]
#[inline]
pub fn export_key<P: AsRef<Path>>(path: P) -> Result<Key128bit> {
    ProtectedFile::export_key(path).map_err(|e| {
        e.set_errno();
        e.to_io_error()
    })
}

#[cfg(feature = "tfs")]
#[inline]
pub fn import_key<P: AsRef<Path>>(
    path: P,
    key: Key128bit,
    key_policy: Option<KeyPolicy>,
) -> Result<()> {
    ProtectedFile::import_key(path, key, key_policy).map_err(|e| {
        e.set_errno();
        e.to_io_error()
    })
}
