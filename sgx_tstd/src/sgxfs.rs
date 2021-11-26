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

//! Filesystem manipulation operations.

use crate::io::{self, SeekFrom, Seek, Read, Initializer, Write};
use crate::path::Path;
use crate::sys::sgxfs as fs_imp;
use crate::sys_common::{AsInner, AsInnerMut, FromInner, IntoInner};
use sgx_types::{sgx_key_128bit_t, sgx_align_key_128bit_t};

/// A reference to an open file on the filesystem.
///
/// An instance of a `File` can be read and/or written depending on what options
/// it was opened with. Files also implement [`Seek`] to alter the logical cursor
/// that the file contains internally.
///
/// Files are automatically closed when they go out of scope.
pub struct SgxFile {
    inner: fs_imp::SgxFile,
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

/// Read the entire contents of a file into a bytes vector.
///
/// This is a convenience function for using SgxFile::open and read_to_end
/// with fewer imports and without an intermediate variable.
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to OpenOptions::open.
///
/// It will also return an error if it encounters while reading an error
/// of a kind other than ErrorKind::Interrupted.
///
pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    SgxFile::open(path)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Read the entire contents of a file into a string.
///
/// This is a convenience function for using SgxFile::open and read_to_string
/// with fewer imports and without an intermediate variable.
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to OpenOptions::open.
///
/// It will also return an error if it encounters while reading an error
/// of a kind other than ErrorKind::Interrupted,
/// or if the contents of the file are not valid UTF-8.
///
pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut string = String::new();
    SgxFile::open(path)?.read_to_string(&mut string)?;
    Ok(string)
}

/// Write a slice as the entire contents of a file.
///
/// This function will create a file if it does not exist,
/// and will entirely replace its contents if it does.
///
/// This is a convenience function for using SgxFile::create and write_all
/// with fewer imports.
///
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    SgxFile::create(path)?.write_all(contents.as_ref())
}

impl SgxFile {
    /// Attempts to open a file in read-only mode.
    ///
    /// See the [`OpenOptions::open`] method for more details.
    ///
    /// # Errors
    ///
    /// This function will return an error if `path` does not already exist.
    /// Other errors may also be returned according to [`OpenOptions::open`].
    ///
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<SgxFile> {
        OpenOptions::new().read(true).open(path.as_ref())
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist,
    /// and will truncate it if it does.
    ///
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<SgxFile> {
        OpenOptions::new().write(true).open(path.as_ref())
    }

    pub fn open_ex<P: AsRef<Path>>(path: P, key: &sgx_key_128bit_t) -> io::Result<SgxFile> {
        OpenOptions::new().read(true).open_ex(path.as_ref(), key)
    }

    pub fn create_ex<P: AsRef<Path>>(path: P, key: &sgx_key_128bit_t) -> io::Result<SgxFile> {
        OpenOptions::new().write(true).open_ex(path.as_ref(), key)
    }

    pub fn is_eof(&self) -> bool {
        self.inner.is_eof()
    }

    pub fn clearerr(&self) {
        self.inner.clearerr()
    }

    pub fn clear_cache(&self) -> io::Result<()> {
        self.inner.clear_cache()
    }
}

impl AsInner<fs_imp::SgxFile> for SgxFile {
    fn as_inner(&self) -> &fs_imp::SgxFile { &self.inner }
}
impl FromInner<fs_imp::SgxFile> for SgxFile {
    fn from_inner(f: fs_imp::SgxFile) -> SgxFile {
        SgxFile { inner: f }
    }
}
impl IntoInner<fs_imp::SgxFile> for SgxFile {
    fn into_inner(self) -> fs_imp::SgxFile {
        self.inner
    }
}

impl Read for SgxFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl Write for SgxFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}

impl Seek for SgxFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<'a> Read for &'a SgxFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl<'a> Write for &'a SgxFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}

impl<'a> Seek for &'a SgxFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to `false`.
    ///
    pub fn new() -> OpenOptions {
        OpenOptions(fs_imp::OpenOptions::new())
    }

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `read`-able if opened.
    ///
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.0.read(read); self
    }

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `write`-able if opened.
    ///
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.0.write(write); self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous contents.
    /// Note that setting `.write(true).append(true)` has the same effect as
    /// setting only `.append(true)`.
    ///
    /// For most filesystems, the operating system guarantees that all writes are
    /// atomic: no writes get mangled because another process writes at the same
    /// time.
    ///
    /// One maybe obvious note when using append-mode: make sure that all data
    /// that belongs together is written to the file in one operation. This
    /// can be done by concatenating strings before passing them to `write()`,
    /// or using a buffered writer (with a buffer of adequate size),
    /// and calling `flush()` when the message is complete.
    ///
    /// If a file is opened with both read and append access, beware that after
    /// opening, and after every write, the position for reading may be set at the
    /// end of the file. So, before writing, save the current position (using
    /// `seek(SeekFrom::Current(0))`, and restore it before the next read.
    ///
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.0.append(append); self
    }

    /// Sets the option for update a previous file.
    pub fn update(&mut self, update: bool) -> &mut OpenOptions {
        self.0.update(update); self
    }

    /// Sets the option for binary a file.
    pub fn binary(&mut self, binary: bool) -> &mut OpenOptions {
        self.0.binary(binary); self
    }

    /// Opens a file at `path` with the options specified by `self`.
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<SgxFile> {
        self._open(path.as_ref())
    }

    pub fn open_ex<P: AsRef<Path>>(&self, path: P, key: &sgx_key_128bit_t) -> io::Result<SgxFile> {
        self._open_ex(path.as_ref(), key)
    }

    fn _open(&self, path: &Path) -> io::Result<SgxFile> {
        let inner = fs_imp::SgxFile::open(path, &self.0)?;
        Ok(SgxFile { inner })
    }

    fn _open_ex(&self, path: &Path, key: &sgx_key_128bit_t) -> io::Result<SgxFile> {
        let inner = fs_imp::SgxFile::open_ex(path, &self.0, key)?;
        Ok(SgxFile { inner })
    }
}

impl AsInnerMut<fs_imp::OpenOptions> for OpenOptions {
    fn as_inner_mut(&mut self) -> &mut fs_imp::OpenOptions { &mut self.0 }
}

pub fn remove<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::remove(path.as_ref())
}

pub fn export_auto_key<P: AsRef<Path>>(path: P) -> io::Result<sgx_key_128bit_t> {
    fs_imp::export_auto_key(path.as_ref())
}

pub fn export_align_auto_key<P: AsRef<Path>>(path: P) -> io::Result<sgx_align_key_128bit_t> {
    fs_imp::export_align_auto_key(path.as_ref())
}

pub fn import_auto_key<P: AsRef<Path>>(path: P, key: &sgx_key_128bit_t) -> io::Result<()> {
    fs_imp::import_auto_key(path.as_ref(), key)
}

/// Copies the contents of one file to another.
/// This function will **overwrite** the contents of `to`.
///
/// Note that if `from` and `to` both point to the same file, then the file
/// will likely get truncated by this operation.
///
/// On success, the total number of bytes copied is returned.
///
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    fs_imp::copy(from.as_ref(), to.as_ref())
}
