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

use sgx_trts::libc;
#[cfg(feature = "untrusted_fs")]
use crate::fs::{self, OpenOptions, Permissions};
#[cfg(not(feature = "untrusted_fs"))]
use crate::untrusted::fs::{self, OpenOptions, Permissions};
use crate::io;
use crate::path::Path;
use crate::sys;
use crate::sys_common::{AsInner, AsInnerMut, FromInner};
use crate::os::fs::MetadataExt as UnixMetadataExt;

/// Unix-specific extensions to `File`
pub trait FileExt {
    /// Reads a number of bytes starting from a given offset.
    ///
    /// Returns the number of bytes read.
    ///
    /// The offset is relative to the start of the file and thus independent
    /// from the current cursor.
    ///
    /// The current file cursor is not affected by this function.
    ///
    /// Note that similar to [`File::read`], it is not an error to return with a
    /// short read.
    ///
    /// [`File::read`]: ../../../../std/fs/struct.File.html#method.read
    ///
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;

    /// Reads the exact number of byte required to fill `buf` from the given offset.
    ///
    /// The offset is relative to the start of the file and thus independent
    /// from the current cursor.
    ///
    /// The current file cursor is not affected by this function.
    ///
    /// Similar to [`Read::read_exact`] but uses [`read_at`] instead of `read`.
    ///
    /// [`Read::read_exact`]: ../../../../std/io/trait.Read.html#method.read_exact
    /// [`read_at`]: #tymethod.read_at
    ///
    /// # Errors
    ///
    /// If this function encounters an error of the kind
    /// [`ErrorKind::Interrupted`] then the error is ignored and the operation
    /// will continue.
    ///
    /// If this function encounters an "end of file" before completely filling
    /// the buffer, it returns an error of the kind [`ErrorKind::UnexpectedEof`].
    /// The contents of `buf` are unspecified in this case.
    ///
    /// If any other read error is encountered then this function immediately
    /// returns. The contents of `buf` are unspecified in this case.
    ///
    /// If this function returns an error, it is unspecified how many bytes it
    /// has read, but it will never read more than would be necessary to
    /// completely fill the buffer.
    ///
    /// [`ErrorKind::Interrupted`]: ../../../../std/io/enum.ErrorKind.html#variant.Interrupted
    /// [`ErrorKind::UnexpectedEof`]: ../../../../std/io/enum.ErrorKind.html#variant.UnexpectedEof
    ///
    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match self.read_at(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    offset += n as u64;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
        } else {
            Ok(())
        }
    }

    /// Writes a number of bytes starting from a given offset.
    ///
    /// Returns the number of bytes written.
    ///
    /// The offset is relative to the start of the file and thus independent
    /// from the current cursor.
    ///
    /// The current file cursor is not affected by this function.
    ///
    /// When writing beyond the end of the file, the file is appropriately
    /// extended and the intermediate bytes are initialized with the value 0.
    ///
    /// Note that similar to [`File::write`], it is not an error to return a
    /// short write.
    ///
    /// [`File::write`]: ../../../../std/fs/struct.File.html#write.v
    ///
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize>;

    /// Attempts to write an entire buffer starting from a given offset.
    ///
    /// The offset is relative to the start of the file and thus independent
    /// from the current cursor.
    ///
    /// The current file cursor is not affected by this function.
    ///
    /// This method will continuously call [`write_at`] until there is no more data
    /// to be written or an error of non-[`ErrorKind::Interrupted`] kind is
    /// returned. This method will not return until the entire buffer has been
    /// successfully written or such an error occurs. The first error that is
    /// not of [`ErrorKind::Interrupted`] kind generated from this method will be
    /// returned.
    ///
    /// # Errors
    ///
    /// This function will return the first error of
    /// non-[`ErrorKind::Interrupted`] kind that [`write_at`] returns.
    ///
    /// [`ErrorKind::Interrupted`]: ../../../../std/io/enum.ErrorKind.html#variant.Interrupted
    /// [`write_at`]: #tymethod.write_at
    ///
    fn write_all_at(&self, mut buf: &[u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match self.write_at(buf, offset) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => {
                    buf = &buf[n..];
                    offset += n as u64
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl FileExt for fs::File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        self.as_inner().read_at(buf, offset)
    }
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        self.as_inner().write_at(buf, offset)
    }
}

/// Unix-specific extensions to [`fs::Permissions`].
///
/// [`fs::Permissions`]: ../../../../std/fs/struct.Permissions.html
pub trait PermissionsExt {
    /// Returns the underlying raw `st_mode` bits that contain the standard
    /// Unix permissions for this file.
    ///
    fn mode(&self) -> u32;

    /// Sets the underlying raw bits for this set of permissions.
    ///
    fn set_mode(&mut self, mode: u32);

    /// Creates a new instance of `Permissions` from the given set of Unix
    /// permission bits.
    ///
    fn from_mode(mode: u32) -> Self;
}

impl PermissionsExt for Permissions {
    fn mode(&self) -> u32 {
        self.as_inner().mode()
    }

    fn set_mode(&mut self, mode: u32) {
        *self = Permissions::from_inner(FromInner::from_inner(mode));
    }

    fn from_mode(mode: u32) -> Permissions {
        Permissions::from_inner(FromInner::from_inner(mode))
    }
}

/// Unix-specific extensions to [`fs::OpenOptions`].
///
/// [`fs::OpenOptions`]: ../../../../std/fs/struct.OpenOptions.html
pub trait OpenOptionsExt {
    /// Sets the mode bits that a new file will be created with.
    ///
    /// If a new file is created as part of an `OpenOptions::open` call then this
    /// specified `mode` will be used as the permission bits for the new file.
    /// If no `mode` is set, the default of `0o666` will be used.
    /// The operating system masks out bits with the system's `umask`, to produce
    /// the final permissions.
    ///
    fn mode(&mut self, mode: u32) -> &mut Self;

    /// Pass custom flags to the `flags` argument of `open`.
    ///
    /// The bits that define the access mode are masked out with `O_ACCMODE`, to
    /// ensure they do not interfere with the access mode set by Rusts options.
    ///
    /// Custom flags can only set flags, not remove flags set by Rusts options.
    /// This options overwrites any previously set custom flags.
    ///
    fn custom_flags(&mut self, flags: i32) -> &mut Self;
}

impl OpenOptionsExt for OpenOptions {
    fn mode(&mut self, mode: u32) -> &mut OpenOptions {
        self.as_inner_mut().mode(mode);
        self
    }

    fn custom_flags(&mut self, flags: i32) -> &mut OpenOptions {
        self.as_inner_mut().custom_flags(flags);
        self
    }
}

/// Unix-specific extensions to [`fs::Metadata`].
///
/// [`fs::Metadata`]: ../../../../std/fs/struct.Metadata.html
pub trait MetadataExt {
    /// Returns the ID of the device containing the file.
    ///
    fn dev(&self) -> u64;
    /// Returns the inode number.
    ///
    fn ino(&self) -> u64;
    /// Returns the rights applied to this file.
    ///
    fn mode(&self) -> u32;
    /// Returns the number of hard links pointing to this file.
    ///
    fn nlink(&self) -> u64;
    /// Returns the user ID of the owner of this file.
    ///
    fn uid(&self) -> u32;
    /// Returns the group ID of the owner of this file.
    ///
    fn gid(&self) -> u32;
    /// Returns the device ID of this file (if it is a special one).
    ///
    fn rdev(&self) -> u64;
    /// Returns the total size of this file in bytes.
    ///
    fn size(&self) -> u64;
    /// Returns the last access time of the file, in seconds since Unix Epoch.
    ///
    fn atime(&self) -> i64;
    /// Returns the last access time of the file, in nanoseconds since [`atime`].
    ///
    /// [`atime`]: #tymethod.atime
    ///
    fn atime_nsec(&self) -> i64;
    /// Returns the last modification time of the file, in seconds since Unix Epoch.
    ///
    fn mtime(&self) -> i64;
    /// Returns the last modification time of the file, in nanoseconds since [`mtime`].
    ///
    /// [`mtime`]: #tymethod.mtime
    ///
    fn mtime_nsec(&self) -> i64;
    /// Returns the last status change time of the file, in seconds since Unix Epoch.
    ///
    fn ctime(&self) -> i64;
    /// Returns the last status change time of the file, in nanoseconds since [`ctime`].
    ///
    /// [`ctime`]: #tymethod.ctime
    ///
    fn ctime_nsec(&self) -> i64;
    /// Returns the blocksize for filesystem I/O.
    ///
    fn blksize(&self) -> u64;
    /// Returns the number of blocks allocated to the file, in 512-byte units.
    ///
    /// Please note that this may be smaller than `st_size / 512` when the file has holes.
    ///
    fn blocks(&self) -> u64;
}

impl MetadataExt for fs::Metadata {
    fn dev(&self) -> u64 {
        self.st_dev()
    }
    fn ino(&self) -> u64 {
        self.st_ino()
    }
    fn mode(&self) -> u32 {
        self.st_mode()
    }
    fn nlink(&self) -> u64 {
        self.st_nlink()
    }
    fn uid(&self) -> u32 {
        self.st_uid()
    }
    fn gid(&self) -> u32 {
        self.st_gid()
    }
    fn rdev(&self) -> u64 {
        self.st_rdev()
    }
    fn size(&self) -> u64 {
        self.st_size()
    }
    fn atime(&self) -> i64 {
        self.st_atime()
    }
    fn atime_nsec(&self) -> i64 {
        self.st_atime_nsec()
    }
    fn mtime(&self) -> i64 {
        self.st_mtime()
    }
    fn mtime_nsec(&self) -> i64 {
        self.st_mtime_nsec()
    }
    fn ctime(&self) -> i64 {
        self.st_ctime()
    }
    fn ctime_nsec(&self) -> i64 {
        self.st_ctime_nsec()
    }
    fn blksize(&self) -> u64 {
        self.st_blksize()
    }
    fn blocks(&self) -> u64 {
        self.st_blocks()
    }
}

/// Unix-specific extensions for [`FileType`].
///
/// Adds support for special Unix file types such as block/character devices,
/// pipes, and sockets.
///
/// [`FileType`]: ../../../../std/fs/struct.FileType.html
pub trait FileTypeExt {
    /// Returns `true` if this file type is a block device.
    ///
    fn is_block_device(&self) -> bool;
    /// Returns `true` if this file type is a char device.
    ///
    fn is_char_device(&self) -> bool;
    /// Returns `true` if this file type is a fifo.
    ///
    fn is_fifo(&self) -> bool;
    /// Returns `true` if this file type is a socket.
    ///
    fn is_socket(&self) -> bool;
}

impl FileTypeExt for fs::FileType {
    fn is_block_device(&self) -> bool {
        self.as_inner().is(libc::S_IFBLK)
    }
    fn is_char_device(&self) -> bool {
        self.as_inner().is(libc::S_IFCHR)
    }
    fn is_fifo(&self) -> bool {
        self.as_inner().is(libc::S_IFIFO)
    }
    fn is_socket(&self) -> bool {
        self.as_inner().is(libc::S_IFSOCK)
    }
}
/// Unix-specific extension methods for [`fs::DirEntry`].
///
pub trait DirEntryExt {
    /// Returns the underlying `d_ino` field in the contained `dirent`
    /// structure.
    fn ino(&self) -> u64;
}

impl DirEntryExt for fs::DirEntry {
    fn ino(&self) -> u64 {
        self.as_inner().ino()
    }
}

/// Creates a new symbolic link on the filesystem.
///
/// The `dst` path will be a symbolic link pointing to the `src` path.
///
/// # Note
///
/// On Windows, you must specify whether a symbolic link points to a file
/// or directory. Use `os::windows::fs::symlink_file` to create a
/// symbolic link to a file, or `os::windows::fs::symlink_dir` to create a
/// symbolic link to a directory. Additionally, the process must have
/// `SeCreateSymbolicLinkPrivilege` in order to be able to create a
/// symbolic link.
///
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    sys::fs::symlink(src.as_ref(), dst.as_ref())
}

pub trait DirBuilderExt {
    fn mode(&mut self, mode: u32) -> &mut Self;
}

impl DirBuilderExt for fs::DirBuilder {
    fn mode(&mut self, mode: u32) -> &mut fs::DirBuilder {
        self.as_inner_mut().set_mode(mode);
        self
    }
}