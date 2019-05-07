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

use sgx_trts::libc;
#[cfg(feature = "untrusted_fs")]
use fs::{self, Permissions, OpenOptions};
#[cfg(not(feature = "untrusted_fs"))]
use untrusted::fs::{self, Permissions, OpenOptions};
use io;
use path::Path;
use sys;
use sys_common::{FromInner, AsInner, AsInnerMut};
use os::fs::MetadataExt as UnixMetadataExt;

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
    /// Note that similar to `File::read`, it is not an error to return with a
    /// short read.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;

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
    /// Note that similar to `File::write`, it is not an error to return a
    /// short write.
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize>;
}

impl FileExt for fs::File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        self.as_inner().read_at(buf, offset)
    }
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        self.as_inner().write_at(buf, offset)
    }
}

/// Unix-specific extensions to `Permissions`
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

/// Unix-specific extensions to `OpenOptions`
pub trait OpenOptionsExt {
    /// Sets the mode bits that a new file will be created with.
    ///
    /// If a new file is created as part of a `File::open_opts` call then this
    /// specified `mode` will be used as the permission bits for the new file.
    /// If no `mode` is set, the default of `0o666` will be used.
    /// The operating system masks out bits with the systems `umask`, to produce
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
        self.as_inner_mut().mode(mode); self
    }

    fn custom_flags(&mut self, flags: i32) -> &mut OpenOptions {
        self.as_inner_mut().custom_flags(flags); self
    }
}

// Hm, why are there casts here to the returned type, shouldn't the types always
// be the same? Right you are! Turns out, however, on android at least the types
// in the raw `stat` structure are not the same as the types being returned. Who
// knew!
//
// As a result to make sure this compiles for all platforms we do the manual
// casts and rely on manual lowering to `stat` if the raw type is desired.
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
    /// Returns the time of the last access to the file.
    ///
    fn atime(&self) -> i64;
    /// Returns the time of the last access to the file in nanoseconds.
    ///
    fn atime_nsec(&self) -> i64;
    /// Returns the time of the last modification of the file.
    ///
    fn mtime(&self) -> i64;
    /// Returns the time of the last modification of the file in nanoseconds.
    ///
    fn mtime_nsec(&self) -> i64;
    /// Returns the time of the last status change of the file.
    ///
    fn ctime(&self) -> i64;
    /// Returns the time of the last status change of the file in nanoseconds.
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
    fn dev(&self) -> u64 { self.st_dev() }
    fn ino(&self) -> u64 { self.st_ino() }
    fn mode(&self) -> u32 { self.st_mode() }
    fn nlink(&self) -> u64 { self.st_nlink() }
    fn uid(&self) -> u32 { self.st_uid() }
    fn gid(&self) -> u32 { self.st_gid() }
    fn rdev(&self) -> u64 { self.st_rdev() }
    fn size(&self) -> u64 { self.st_size() }
    fn atime(&self) -> i64 { self.st_atime() }
    fn atime_nsec(&self) -> i64 { self.st_atime_nsec() }
    fn mtime(&self) -> i64 { self.st_mtime() }
    fn mtime_nsec(&self) -> i64 { self.st_mtime_nsec() }
    fn ctime(&self) -> i64 { self.st_ctime() }
    fn ctime_nsec(&self) -> i64 { self.st_ctime_nsec() }
    fn blksize(&self) -> u64 { self.st_blksize() }
    fn blocks(&self) -> u64 { self.st_blocks() }
}

/// Add support for special unix types (block/char device, fifo and socket).
pub trait FileTypeExt {
    /// Returns whether this file type is a block device.
    ///
    fn is_block_device(&self) -> bool;
    /// Returns whether this file type is a char device.
    ///
    fn is_char_device(&self) -> bool;
    /// Returns whether this file type is a fifo.
    ///
    fn is_fifo(&self) -> bool;
    /// Returns whether this file type is a socket.
    ///
    fn is_socket(&self) -> bool;
}

impl FileTypeExt for fs::FileType {
    fn is_block_device(&self) -> bool { self.as_inner().is(libc::S_IFBLK) }
    fn is_char_device(&self) -> bool { self.as_inner().is(libc::S_IFCHR) }
    fn is_fifo(&self) -> bool { self.as_inner().is(libc::S_IFIFO) }
    fn is_socket(&self) -> bool { self.as_inner().is(libc::S_IFSOCK) }
}

/// Creates a new symbolic link on the filesystem.
///
/// The `dst` path will be a symbolic link pointing to the `src` path.
///
/// # Note
///
/// On Windows, you must specify whether a symbolic link points to a file
/// or directory.  Use `os::windows::fs::symlink_file` to create a
/// symbolic link to a file, or `os::windows::fs::symlink_dir` to create a
/// symbolic link to a directory.  Additionally, the process must have
/// `SeCreateSymbolicLinkPrivilege` in order to be able to create a
/// symbolic link.
///
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()>
{
    sys::fs::symlink(src.as_ref(), dst.as_ref())
}