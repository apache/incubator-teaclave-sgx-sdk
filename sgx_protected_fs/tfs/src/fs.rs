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

use crate::sys as fs_imp;
use sgx_types::types::{Key128bit, Mac128bit};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::string::String;
use std::vec::Vec;

cfg_if! {
    if #[cfg(feature = "tfs")] {
        use sgx_rsrvmm::map::Map;
        use sgx_types::error::errno::ESGX;
        use sgx_types::error::OsResult;
        use sgx_types::types::KeyPolicy;
    }
}

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a SgxFile is opened and
/// what operations are permitted on the open file. The SgxFile::open and
/// SgxFile::create methods are aliases for commonly used options using this
/// builder.
///
#[derive(Clone, Debug)]
pub struct OpenOptions(fs_imp::OpenOptions);

#[derive(Clone, Debug)]
pub struct EncryptMode(fs_imp::EncryptMode);

/// A reference to an open Sgxfile on the filesystem.
///
/// An instance of a `SgxFile` can be read and/or written depending on what options
/// it was opened with. SgxFiles also implement [`Seek`] to alter the logical cursor
/// that the file contains internally.
///
/// SgxFiles are automatically closed when they go out of scope.
pub struct SgxFile {
    inner: fs_imp::SgxFile,
}

/// Read the entire contents of a file into a bytes vector.
#[cfg(feature = "tfs")]
pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = SgxFile::open(path)?;
    let mut bytes = Vec::with_capacity(buffer_capacity_required(&file));
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Read the entire contents of a file into a string.
#[cfg(feature = "tfs")]
pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = SgxFile::open(path)?;
    let mut string = String::with_capacity(buffer_capacity_required(&file));
    file.read_to_string(&mut string)?;
    Ok(string)
}

/// Write a slice as the entire contents of a file.
#[cfg(feature = "tfs")]
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    SgxFile::create(path)?.write_all(contents.as_ref())
}

pub fn read_with_key<P: AsRef<Path>>(path: P, key: Key128bit) -> io::Result<Vec<u8>> {
    let mut file = SgxFile::open_with_key(path, key)?;
    let mut bytes = Vec::with_capacity(buffer_capacity_required(&file));
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn read_to_string_with_key<P: AsRef<Path>>(path: P, key: Key128bit) -> io::Result<String> {
    let mut file = SgxFile::open_with_key(path, key)?;
    let mut string = String::with_capacity(buffer_capacity_required(&file));
    file.read_to_string(&mut string)?;
    Ok(string)
}

pub fn write_with_key<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    key: Key128bit,
    contents: C,
) -> io::Result<()> {
    SgxFile::create_with_key(path, key)?.write_all(contents.as_ref())
}

pub fn read_integrity_only<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = SgxFile::open_integrity_only(path)?;
    let mut bytes = Vec::with_capacity(buffer_capacity_required(&file));
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn read_to_string_integrity_only<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = SgxFile::open_integrity_only(path)?;
    let mut string = String::with_capacity(buffer_capacity_required(&file));
    file.read_to_string(&mut string)?;
    Ok(string)
}

pub fn write_integrity_only<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
) -> io::Result<()> {
    SgxFile::create_integrity_only(path)?.write_all(contents.as_ref())
}

impl SgxFile {
    #[cfg(feature = "tfs")]
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<SgxFile> {
        OpenOptions::new().read(true).open(path.as_ref())
    }

    #[cfg(feature = "tfs")]
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<SgxFile> {
        OpenOptions::new().write(true).open(path.as_ref())
    }

    #[cfg(feature = "tfs")]
    pub fn append<P: AsRef<Path>>(path: P) -> io::Result<SgxFile> {
        OpenOptions::new().append(true).open(path.as_ref())
    }

    pub fn open_with_key<P: AsRef<Path>>(path: P, key: Key128bit) -> io::Result<SgxFile> {
        OpenOptions::new()
            .read(true)
            .open_with_key(path.as_ref(), key)
    }

    pub fn create_with_key<P: AsRef<Path>>(path: P, key: Key128bit) -> io::Result<SgxFile> {
        OpenOptions::new()
            .write(true)
            .open_with_key(path.as_ref(), key)
    }

    pub fn append_with_key<P: AsRef<Path>>(path: P, key: Key128bit) -> io::Result<SgxFile> {
        OpenOptions::new()
            .append(true)
            .open_with_key(path.as_ref(), key)
    }

    pub fn open_integrity_only<P: AsRef<Path>>(path: P) -> io::Result<SgxFile> {
        OpenOptions::new()
            .read(true)
            .open_integrity_only(path.as_ref())
    }

    pub fn create_integrity_only<P: AsRef<Path>>(path: P) -> io::Result<SgxFile> {
        OpenOptions::new()
            .write(true)
            .open_integrity_only(path.as_ref())
    }

    pub fn append_integrity_only<P: AsRef<Path>>(path: P) -> io::Result<SgxFile> {
        OpenOptions::new()
            .append(true)
            .open_integrity_only(path.as_ref())
    }

    pub fn open_with<P: AsRef<Path>>(
        path: P,
        encrypt_mode: EncryptMode,
        cache_size: Option<usize>,
    ) -> io::Result<SgxFile> {
        OpenOptions::new()
            .read(true)
            .open_with(path.as_ref(), encrypt_mode, cache_size)
    }

    pub fn create_with<P: AsRef<Path>>(
        path: P,
        encrypt_mode: EncryptMode,
        cache_size: Option<usize>,
    ) -> io::Result<SgxFile> {
        OpenOptions::new()
            .write(true)
            .open_with(path.as_ref(), encrypt_mode, cache_size)
    }

    pub fn append_with<P: AsRef<Path>>(
        path: P,
        encrypt_mode: EncryptMode,
        cache_size: Option<usize>,
    ) -> io::Result<SgxFile> {
        OpenOptions::new()
            .append(true)
            .open_with(path.as_ref(), encrypt_mode, cache_size)
    }

    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.inner.set_len(size)
    }

    pub fn tell(&self) -> io::Result<u64> {
        self.inner.tell()
    }

    pub fn file_size(&self) -> io::Result<u64> {
        self.inner.file_size()
    }

    pub fn is_eof(&self) -> bool {
        self.inner.is_eof()
    }

    pub fn clear_error(&self) -> io::Result<()> {
        self.inner.clear_error()
    }

    pub fn clear_cache(&self) -> io::Result<()> {
        self.inner.clear_cache()
    }

    pub fn get_mac(&self) -> io::Result<Mac128bit> {
        self.inner.get_mac()
    }

    pub fn rename<P: AsRef<str>>(&mut self, old_name: P, new_name: P) -> io::Result<()> {
        self.inner.rename(old_name.as_ref(), new_name.as_ref())
    }
}

/// Indicates how much extra capacity is needed to read the rest of the file.
fn buffer_capacity_required(file: &SgxFile) -> usize {
    let size = file.file_size().unwrap_or(0);
    let pos = file.tell().unwrap_or(0);
    // Don't worry about `usize` overflow because reading will fail regardless
    // in that case.
    size.saturating_sub(pos) as usize
}

impl Read for SgxFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for SgxFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for SgxFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl Read for &SgxFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for &SgxFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for &SgxFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl FileExt for SgxFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        self.inner.read_at(buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        self.inner.write_at(buf, offset)
    }
}

#[cfg(feature = "tfs")]
impl Map for SgxFile {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> OsResult<usize> {
        self.inner
            .read_at(buf, offset as u64)
            .map_err(|e| e.raw_os_error().unwrap_or(ESGX))
    }
    fn write_at(&self, buf: &[u8], offset: usize) -> OsResult<usize> {
        self.inner
            .write_at(buf, offset as u64)
            .map_err(|e| e.raw_os_error().unwrap_or(ESGX))
    }
    #[inline]
    fn flush(&self) -> OsResult {
        self.inner
            .flush()
            .map_err(|e| e.raw_os_error().unwrap_or(ESGX))
    }
}

pub fn remove<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::remove(path.as_ref())
}

#[cfg(feature = "tfs")]
pub fn export_key<P: AsRef<Path>>(path: P) -> io::Result<Key128bit> {
    fs_imp::export_key(path.as_ref())
}

#[cfg(feature = "tfs")]
pub fn import_key<P: AsRef<Path>>(
    path: P,
    key: Key128bit,
    key_policy: Option<KeyPolicy>,
) -> io::Result<()> {
    fs_imp::import_key(path.as_ref(), key, key_policy)
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    pub fn new() -> OpenOptions {
        OpenOptions(fs_imp::OpenOptions::new())
    }

    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.0.read(read);
        self
    }

    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.0.write(write);
        self
    }

    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.0.append(append);
        self
    }

    /// Sets the option for update a previous file.
    pub fn update(&mut self, update: bool) -> &mut OpenOptions {
        self.0.update(update);
        self
    }

    /// Sets the option for binary a file.
    pub fn binary(&mut self, binary: bool) -> &mut OpenOptions {
        self.0.binary(binary);
        self
    }

    /// Opens a file at `path` with the options specified by `self`.
    #[cfg(feature = "tfs")]
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<SgxFile> {
        self.open_with(path, EncryptMode::auto_key(None), None)
    }

    pub fn open_with_key<P: AsRef<Path>>(&self, path: P, key: Key128bit) -> io::Result<SgxFile> {
        self.open_with(path, EncryptMode::user_key(key), None)
    }

    pub fn open_integrity_only<P: AsRef<Path>>(&self, path: P) -> io::Result<SgxFile> {
        self.open_with(path, EncryptMode::integrity_only(), None)
    }

    pub fn open_with<P: AsRef<Path>>(
        &self,
        path: P,
        encrypt_mode: EncryptMode,
        cache_size: Option<usize>,
    ) -> io::Result<SgxFile> {
        let inner = fs_imp::SgxFile::open(path, &self.0, &encrypt_mode.0, cache_size)?;
        Ok(SgxFile { inner })
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptMode {
    #[cfg(feature = "tfs")]
    #[inline]
    pub fn auto_key(key_policy: Option<KeyPolicy>) -> EncryptMode {
        EncryptMode(fs_imp::EncryptMode::EncryptAutoKey(
            key_policy.unwrap_or(KeyPolicy::MRSIGNER),
        ))
    }

    #[inline]
    pub fn user_key(key: Key128bit) -> EncryptMode {
        EncryptMode(fs_imp::EncryptMode::EncryptUserKey(key))
    }

    #[inline]
    pub fn integrity_only() -> EncryptMode {
        EncryptMode(fs_imp::EncryptMode::IntegrityOnly)
    }
}
