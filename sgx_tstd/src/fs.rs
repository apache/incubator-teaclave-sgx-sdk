// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! Filesystem manipulation operations.

use core::fmt;
use io::{self, SeekFrom, Seek, Read, Initializer, Write};
use path::{Path, PathBuf};
use sys::fs as fs_imp;
use sys_common::{AsInnerMut, FromInner, AsInner, IntoInner};
use time::SystemTime;

/// A reference to an open file on the filesystem.
///
/// An instance of a `File` can be read and/or written depending on what options
/// it was opened with. Files also implement [`Seek`] to alter the logical cursor
/// that the file contains internally.
///
/// Files are automatically closed when they go out of scope.
///
pub struct File {
    inner: fs_imp::File,
}

/// Metadata information about a file.
///
/// This structure is returned from the [`metadata`] or
/// [`symlink_metadata`] function or method and represents known
/// metadata about a file such as its permissions, size, modification
/// times, etc.
///
#[derive(Clone)]
pub struct Metadata(fs_imp::FileAttr);

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a [`File`] is opened and
/// what operations are permitted on the open file. The [`File::open`] and
/// [`File::create`] methods are aliases for commonly used options using this
/// builder.
///
/// [`File`]: struct.File.html
/// [`File::open`]: struct.File.html#method.open
/// [`File::create`]: struct.File.html#method.create
///
/// Generally speaking, when using `OpenOptions`, you'll first call [`new`],
/// then chain calls to methods to set each option, then call [`open`],
/// passing the path of the file you're trying to open. This will give you a
/// [`io::Result`][result] with a [`File`][file] inside that you can further
/// operate on.
///
/// [`new`]: struct.OpenOptions.html#method.new
/// [`open`]: struct.OpenOptions.html#method.open
/// [result]: ../io/type.Result.html
/// [file]: struct.File.html
///
#[derive(Clone, Debug)]
pub struct OpenOptions(fs_imp::OpenOptions);

/// Representation of the various permissions on a file.
///
/// This module only currently provides one bit of information, [`readonly`],
/// which is exposed on all currently supported platforms. Unix-specific
/// functionality, such as mode bits, is available through the
/// `os::unix::PermissionsExt` trait.
///
/// [`readonly`]: struct.Permissions.html#method.readonly
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Permissions(fs_imp::FilePermissions);

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
///
/// [`Metadata::file_type`]: struct.Metadata.html#method.file_type
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType(fs_imp::FileType);

/// How large a buffer to pre-allocate before reading the entire file.
fn initial_buffer_size(file: &File) -> usize {
    // Allocate one extra byte so the buffer doesn't need to grow before the
    // final `read` call at the end of the file.  Don't worry about `usize`
    // overflow because reading will fail regardless in that case.
    file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0)
}

/// Read the entire contents of a file into a bytes vector.
///
/// This is a convenience function for using [`File::open`] and [`read_to_end`]
/// with fewer imports and without an intermediate variable.  It pre-allocates a
/// buffer based on the file size when available, so it is generally faster than
/// reading into a vector created with `Vec::new()`.
///
/// [`File::open`]: struct.File.html#method.open
/// [`read_to_end`]: ../io/trait.Read.html#method.read_to_end
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to [`OpenOptions::open`].
///
/// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
///
/// It will also return an error if it encounters while reading an error
/// of a kind other than [`ErrorKind::Interrupted`].
///
/// [`ErrorKind::Interrupted`]: ../../std/io/enum.ErrorKind.html#variant.Interrupted
///
pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::with_capacity(initial_buffer_size(&file));
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Read the entire contents of a file into a string.
///
/// This is a convenience function for using [`File::open`] and [`read_to_string`]
/// with fewer imports and without an intermediate variable.  It pre-allocates a
/// buffer based on the file size when available, so it is generally faster than
/// reading into a string created with `String::new()`.
///
/// [`File::open`]: struct.File.html#method.open
/// [`read_to_string`]: ../io/trait.Read.html#method.read_to_string
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to [`OpenOptions::open`].
///
/// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
///
/// It will also return an error if it encounters while reading an error
/// of a kind other than [`ErrorKind::Interrupted`],
/// or if the contents of the file are not valid UTF-8.
///
/// [`ErrorKind::Interrupted`]: ../../std/io/enum.ErrorKind.html#variant.Interrupted
///
pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut string = String::with_capacity(initial_buffer_size(&file));
    file.read_to_string(&mut string)?;
    Ok(string)
}

/// Write a slice as the entire contents of a file.
///
/// This function will create a file if it does not exist,
/// and will entirely replace its contents if it does.
///
/// This is a convenience function for using [`File::create`] and [`write_all`]
/// with fewer imports.
///
/// [`File::create`]: struct.File.html#method.create
/// [`write_all`]: ../io/trait.Write.html#method.write_all
///
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    File::create(path)?.write_all(contents.as_ref())
}

impl File {
    /// Attempts to open a file in read-only mode.
    ///
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
        OpenOptions::new().read(true).open(path.as_ref())
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist,
    /// and will truncate it if it does.
    ///
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<File> {
        OpenOptions::new().write(true).create(true).truncate(true).open(path.as_ref())
    }

    /// Attempts to sync all OS-internal metadata to disk.
    ///
    /// This function will attempt to ensure that all in-core data reaches the
    /// filesystem before returning.
    ///
    pub fn sync_all(&self) -> io::Result<()> {
        self.inner.fsync()
    }

    /// This function is similar to [`sync_all`], except that it may not
    /// synchronize file metadata to the filesystem.
    ///
    /// This is intended for use cases that must synchronize content, but don't
    /// need the metadata on disk. The goal of this method is to reduce disk
    /// operations.
    ///
    /// Note that some platforms may simply implement this in terms of
    /// [`sync_all`].
    ///
    /// [`sync_all`]: struct.File.html#method.sync_all
    ///
    pub fn sync_data(&self) -> io::Result<()> {
        self.inner.datasync()
    }

    /// Truncates or extends the underlying file, updating the size of
    /// this file to become `size`.
    ///
    /// If the `size` is less than the current file's size, then the file will
    /// be shrunk. If it is greater than the current file's size, then the file
    /// will be extended to `size` and have all of the intermediate data filled
    /// in with 0s.
    ///
    /// The file's cursor isn't changed. In particular, if the cursor was at the
    /// end and the file is shrunk using this operation, the cursor will now be
    /// past the end.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file is not opened for writing.
    ///
    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.inner.truncate(size)
    }

    /// Queries metadata about the underlying file.
    ///
    pub fn metadata(&self) -> io::Result<Metadata> {
        self.inner.file_attr().map(Metadata)
    }

    /// Create a new `File` instance that shares the same underlying file handle
    /// as the existing `File` instance. Reads, writes, and seeks will affect
    /// both `File` instances simultaneously.
    ///
    pub fn try_clone(&self) -> io::Result<File> {
        Ok(File {
            inner: self.inner.duplicate()?
        })
    }

    /// Changes the permissions on the underlying file.
    ///
    /// # Platform-specific behavior
    ///
    /// This function currently corresponds to the `fchmod` function on Unix and
    /// the `SetFileInformationByHandle` function on Windows. Note that, this
    /// [may change in the future][changes].
    ///
    /// [changes]: ../io/index.html#platform-specific-behavior
    ///
    /// # Errors
    ///
    /// This function will return an error if the user lacks permission change
    /// attributes on the underlying file. It may also return an error in other
    /// os-specific unspecified cases.
    ///
    pub fn set_permissions(&self, perm: Permissions) -> io::Result<()> {
        self.inner.set_permissions(perm.0)
    }
}

impl AsInner<fs_imp::File> for File {
    fn as_inner(&self) -> &fs_imp::File { &self.inner }
}
impl FromInner<fs_imp::File> for File {
    fn from_inner(f: fs_imp::File) -> File {
        File { inner: f }
    }
}
impl IntoInner<fs_imp::File> for File {
    fn into_inner(self) -> fs_imp::File {
        self.inner
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<'a> Read for &'a File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        Initializer::nop()
    }
}

impl<'a> Write for &'a File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}

impl<'a> Seek for &'a File {
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
    /// If the file already exists, any write calls on it will overwrite its
    /// contents, without truncating it.
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
    /// can be done by concatenating strings before passing them to [`write()`],
    /// or using a buffered writer (with a buffer of adequate size),
    /// and calling [`flush()`] when the message is complete.
    ///
    /// If a file is opened with both read and append access, beware that after
    /// opening, and after every write, the position for reading may be set at the
    /// end of the file. So, before writing, save the current position (using
    /// [`seek`]`(`[`SeekFrom`]`::`[`Current`]`(0))`, and restore it before the next read.
    ///
    /// ## Note
    ///
    /// This function doesn't create the file if it doesn't exist. Use the [`create`]
    /// method to do so.
    ///
    /// [`write()`]: ../../std/fs/struct.File.html#method.write
    /// [`flush()`]: ../../std/fs/struct.File.html#method.flush
    /// [`seek`]: ../../std/fs/struct.File.html#method.seek
    /// [`SeekFrom`]: ../../std/io/enum.SeekFrom.html
    /// [`Current`]: ../../std/io/enum.SeekFrom.html#variant.Current
    /// [`create`]: #method.create
    ///
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.0.append(append); self
    }

    /// Sets the option for truncating a previous file.
    ///
    /// If a file is successfully opened with this option set it will truncate
    /// the file to 0 length if it already exists.
    ///
    /// The file must be opened with write access for truncate to work.
    ///
    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        self.0.truncate(truncate); self
    }

    /// Sets the option for creating a new file.
    ///
    /// This option indicates whether a new file will be created if the file
    /// does not yet already exist.
    ///
    /// In order for the file to be created, [`write`] or [`append`] access must
    /// be used.
    ///
    /// [`write`]: #method.write
    /// [`append`]: #method.append
    ///
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.0.create(create); self
    }

    /// Sets the option to always create a new file.
    ///
    /// This option indicates whether a new file will be created.
    /// No file is allowed to exist at the target location, also no (dangling)
    /// symlink.
    ///
    /// This option is useful because it is atomic. Otherwise between checking
    /// whether a file exists and creating a new one, the file may have been
    /// created by another process (a TOCTOU race condition / attack).
    ///
    /// If `.create_new(true)` is set, [`.create()`] and [`.truncate()`] are
    /// ignored.
    ///
    /// The file must be opened with write or append access in order to create
    /// a new file.
    ///
    /// [`.create()`]: #method.create
    /// [`.truncate()`]: #method.truncate
    ///
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.0.create_new(create_new); self
    }

    /// Opens a file at `path` with the options specified by `self`.
    ///
    /// # Errors
    ///
    /// This function will return an error under a number of different
    /// circumstances. Some of these error conditions are listed here, together
    /// with their [`ErrorKind`]. The mapping to [`ErrorKind`]s is not part of
    /// the compatibility contract of the function, especially the `Other` kind
    /// might change to more specific kinds in the future.
    ///
    /// * [`NotFound`]: The specified file does not exist and neither `create`
    ///   or `create_new` is set.
    /// * [`NotFound`]: One of the directory components of the file path does
    ///   not exist.
    /// * [`PermissionDenied`]: The user lacks permission to get the specified
    ///   access rights for the file.
    /// * [`PermissionDenied`]: The user lacks permission to open one of the
    ///   directory components of the specified path.
    /// * [`AlreadyExists`]: `create_new` was specified and the file already
    ///   exists.
    /// * [`InvalidInput`]: Invalid combinations of open options (truncate
    ///   without write access, no access mode set, etc.).
    /// * [`Other`]: One of the directory components of the specified file path
    ///   was not, in fact, a directory.
    /// * [`Other`]: Filesystem-level errors: full disk, write permission
    ///   requested on a read-only file system, exceeded disk quota, too many
    ///   open files, too long filename, too many symbolic links in the
    ///   specified path (Unix-like systems only), etc.
    ///
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        self._open(path.as_ref())
    }

    fn _open(&self, path: &Path) -> io::Result<File> {
        let inner = fs_imp::File::open(path, &self.0)?;
        Ok(File { inner: inner })
    }
}

impl AsInnerMut<fs_imp::OpenOptions> for OpenOptions {
    fn as_inner_mut(&mut self) -> &mut fs_imp::OpenOptions { &mut self.0 }
}

impl Metadata {
    /// Returns the file type for this metadata.
    ///
    pub fn file_type(&self) -> FileType {
        FileType(self.0.file_type())
    }

    /// Returns whether this metadata is for a directory. The
    /// result is mutually exclusive to the result of
    /// [`is_file`], and will be false for symlink metadata
    /// obtained from [`symlink_metadata`].
    ///
    pub fn is_dir(&self) -> bool { self.file_type().is_dir() }

    /// Returns whether this metadata is for a regular file. The
    /// result is mutually exclusive to the result of
    /// [`is_dir`], and will be false for symlink metadata
    /// obtained from [`symlink_metadata`].
    ///
    pub fn is_file(&self) -> bool { self.file_type().is_file() }

    /// Returns the size of the file, in bytes, this metadata is for.
    ///
    pub fn len(&self) -> u64 { self.0.size() }

    /// Returns the permissions of the file this metadata is for.
    ///
    pub fn permissions(&self) -> Permissions {
        Permissions(self.0.perm())
    }

    /// Returns the last modification time listed in this metadata.
    ///
    /// The returned value corresponds to the `mtime` field of `stat` on Unix
    /// platforms and the `ftLastWriteTime` field on Windows platforms.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    pub fn modified(&self) -> io::Result<SystemTime> {
        self.0.modified().map(FromInner::from_inner)
    }

    /// Returns the last access time of this metadata.
    ///
    /// The returned value corresponds to the `atime` field of `stat` on Unix
    /// platforms and the `ftLastAccessTime` field on Windows platforms.
    ///
    /// Note that not all platforms will keep this field update in a file's
    /// metadata, for example Windows has an option to disable updating this
    /// time when files are accessed and Linux similarly has `noatime`.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    pub fn accessed(&self) -> io::Result<SystemTime> {
        self.0.accessed().map(FromInner::from_inner)
    }

    /// Returns the creation time listed in the this metadata.
    ///
    /// The returned value corresponds to the `birthtime` field of `stat` on
    /// Unix platforms and the `ftCreationTime` field on Windows platforms.
    ///
    /// # Errors
    ///
    /// This field may not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    pub fn created(&self) -> io::Result<SystemTime> {
        self.0.created().map(FromInner::from_inner)
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Metadata")
            .field("file_type", &self.file_type())
            .field("is_dir", &self.is_dir())
            .field("is_file", &self.is_file())
            .field("permissions", &self.permissions())
            .field("modified", &self.modified())
            .field("accessed", &self.accessed())
            .field("created", &self.created())
            .finish()
    }
}

impl AsInner<fs_imp::FileAttr> for Metadata {
    fn as_inner(&self) -> &fs_imp::FileAttr { &self.0 }
}

impl Permissions {
    /// Returns whether these permissions describe a readonly (unwritable) file.
    ///
    pub fn readonly(&self) -> bool { self.0.readonly() }

    /// Modifies the readonly flag for this set of permissions. If the
    /// `readonly` argument is `true`, using the resulting `Permission` will
    /// update file permissions to forbid writing. Conversely, if it's `false`,
    /// using the resulting `Permission` will update file permissions to allow
    /// writing.
    ///
    /// This operation does **not** modify the filesystem. To modify the
    /// filesystem use the `fs::set_permissions` function.
    ///
    pub fn set_readonly(&mut self, readonly: bool) {
        self.0.set_readonly(readonly)
    }
}

impl FileType {
    /// Test whether this file type represents a directory. The
    /// result is mutually exclusive to the results of
    /// [`is_file`] and [`is_symlink`]; only zero or one of these
    /// tests may pass.
    ///
    pub fn is_dir(&self) -> bool { self.0.is_dir() }

    /// Test whether this file type represents a regular file.
    /// The result is  mutually exclusive to the results of
    /// [`is_dir`] and [`is_symlink`]; only zero or one of these
    /// tests may pass.
    ///
    pub fn is_file(&self) -> bool { self.0.is_file() }

    /// Test whether this file type represents a symbolic link.
    /// The result is mutually exclusive to the results of
    /// [`is_dir`] and [`is_file`]; only zero or one of these
    /// tests may pass.
    ///
    pub fn is_symlink(&self) -> bool { self.0.is_symlink() }
}

impl AsInner<fs_imp::FileType> for FileType {
    fn as_inner(&self) -> &fs_imp::FileType { &self.0 }
}

impl FromInner<fs_imp::FilePermissions> for Permissions {
    fn from_inner(f: fs_imp::FilePermissions) -> Permissions {
        Permissions(f)
    }
}

impl AsInner<fs_imp::FilePermissions> for Permissions {
    fn as_inner(&self) -> &fs_imp::FilePermissions { &self.0 }
}

/// Removes a file from the filesystem.
///
/// Note that there is no
/// guarantee that the file is immediately deleted (e.g. depending on
/// platform, other open file descriptors may prevent immediate removal).
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `unlink` function on Unix
/// and the `DeleteFile` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` points to a directory.
/// * The user lacks permissions to remove the file.
///
pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::unlink(path.as_ref())
}

/// Given a path, query the file system to get information about a file,
/// directory, etc.
///
/// This function will traverse symbolic links to query information about the
/// destination file.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `stat` function on Unix
/// and the `GetFileAttributesEx` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to perform `metadata` call on `path`.
/// * `path` does not exist.
///
pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    fs_imp::stat(path.as_ref()).map(Metadata)
}

/// Query the metadata about a file without following symlinks.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `lstat` function on Unix
/// and the `GetFileAttributesEx` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to perform `metadata` call on `path`.
/// * `path` does not exist.
///
pub fn symlink_metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    fs_imp::lstat(path.as_ref()).map(Metadata)
}

/// Rename a file or directory to a new name, replacing the original file if
/// `to` already exists.
///
/// This will not work if the new name is on a different mount point.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `rename` function on Unix
/// and the `MoveFileEx` function with the `MOVEFILE_REPLACE_EXISTING` flag on Windows.
///
/// Because of this, the behavior when both `from` and `to` exist differs. On
/// Unix, if `from` is a directory, `to` must also be an (empty) directory. If
/// `from` is not a directory, `to` must also be not a directory. In contrast,
/// on Windows, `from` can be anything, but `to` must *not* be a directory.
///
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `from` does not exist.
/// * The user lacks permissions to view contents.
/// * `from` and `to` are on separate filesystems.
///
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    fs_imp::rename(from.as_ref(), to.as_ref())
}

/// Copies the contents of one file to another. This function will also
/// copy the permission bits of the original file to the destination file.
///
/// This function will **overwrite** the contents of `to`.
///
/// Note that if `from` and `to` both point to the same file, then the file
/// will likely get truncated by this operation.
///
/// On success, the total number of bytes copied is returned and it is equal to
/// the length of the `to` file as reported by `metadata`.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `open` function in Unix
/// with `O_RDONLY` for `from` and `O_WRONLY`, `O_CREAT`, and `O_TRUNC` for `to`.
/// `O_CLOEXEC` is set for returned file descriptors.
/// On Windows, this function currently corresponds to `CopyFileEx`. Alternate
/// NTFS streams are copied but only the size of the main stream is returned by
/// this function.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The `from` path is not a file.
/// * The `from` file does not exist.
/// * The current process does not have the permission rights to access
///   `from` or write `to`.
///
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    fs_imp::copy(from.as_ref(), to.as_ref())
}

/// Creates a new hard link on the filesystem.
///
/// The `dst` path will be a link pointing to the `src` path. Note that systems
/// often require these two paths to both be located on the same filesystem.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `link` function on Unix
/// and the `CreateHardLink` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The `src` path is not a file or doesn't exist.
///
pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    fs_imp::link(src.as_ref(), dst.as_ref())
}

/// Creates a new symbolic link on the filesystem.
///
/// The `dst` path will be a symbolic link pointing to the `src` path.
/// On Windows, this will be a file symlink, not a directory symlink;
/// for this reason, the platform-specific `std::os::unix::fs::symlink`
/// and `std::os::windows::fs::{symlink_file, symlink_dir}` should be
/// used instead to make the intent explicit.
///
pub fn soft_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    fs_imp::symlink(src.as_ref(), dst.as_ref())
}

/// Reads a symbolic link, returning the file that the link points to.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `readlink` function on Unix
/// and the `CreateFile` function with `FILE_FLAG_OPEN_REPARSE_POINT` and
/// `FILE_FLAG_BACKUP_SEMANTICS` flags on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` is not a symbolic link.
/// * `path` does not exist.
///
pub fn read_link<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    fs_imp::readlink(path.as_ref())
}

/// Returns the canonical form of a path with all intermediate components
/// normalized and symbolic links resolved.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `realpath` function on Unix
/// and the `CreateFile` and `GetFinalPathNameByHandle` functions on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` does not exist.
/// * A component in path is not a directory.
///
pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    fs_imp::canonicalize(path.as_ref())
}

/// Changes the permissions found on a file or a directory.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `chmod` function on Unix
/// and the `SetFileAttributes` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: ../io/index.html#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` does not exist.
/// * The user lacks the permission to change attributes of the file.
///
pub fn set_permissions<P: AsRef<Path>>(path: P, perm: Permissions)
                                       -> io::Result<()> {
    fs_imp::set_perm(path.as_ref(), perm.0)
}