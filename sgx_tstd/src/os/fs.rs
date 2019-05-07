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
use fs::Metadata;
#[cfg(not(feature = "untrusted_fs"))]
use untrusted::fs::Metadata;
use sys_common::AsInner;

use os::raw;

/// OS-specific extension methods for `fs::Metadata`
pub trait MetadataExt {
    /// Gain a reference to the underlying `stat` structure which contains
    /// the raw information returned by the OS.
    ///
    /// The contents of the returned [`stat`] are **not** consistent across
    /// Unix platforms. The `os::unix::fs::MetadataExt` trait contains the
    /// cross-Unix abstractions contained within the raw stat.
    ///
    fn as_raw_stat(&self) -> &raw::stat;

    /// Returns the device ID on which this file resides.
    ///
    fn st_dev(&self) -> u64;
    /// Returns the inode number.
    ///
    fn st_ino(&self) -> u64;
    /// Returns the file type and mode.
    ///
    fn st_mode(&self) -> u32;
    /// Returns the number of hard links to file.
    ///
    fn st_nlink(&self) -> u64;
    /// Returns the user ID of the file owner.
    ///
    fn st_uid(&self) -> u32;
    /// Returns the group ID of the file owner.
    ///
    fn st_gid(&self) -> u32;
    /// Returns the device ID that this file represents. Only relevant for special file.
    ///
    fn st_rdev(&self) -> u64;
    /// Returns the size of the file (if it is a regular file or a symbolic link) in bytes.
    ///
    /// The size of a symbolic link is the length of the pathname it contains,
    /// without a terminating null byte.
    ///
    fn st_size(&self) -> u64;
    /// Returns the last access time.
    ///
    fn st_atime(&self) -> i64;
    /// Returns the last access time, nano seconds part.
    ///
    fn st_atime_nsec(&self) -> i64;
    /// Returns the last modification time.
    ///
    fn st_mtime(&self) -> i64;
    /// Returns the last modification time, nano seconds part.
    ///
    fn st_mtime_nsec(&self) -> i64;
    /// Returns the last status change time.
    ///
    fn st_ctime(&self) -> i64;
    /// Returns the last status change time, nano seconds part.
    ///
    fn st_ctime_nsec(&self) -> i64;
    /// Returns the "preferred" blocksize for efficient filesystem I/O.
    ///
    fn st_blksize(&self) -> u64;
    /// Returns the number of blocks allocated to the file, 512-byte units.
    ///
    fn st_blocks(&self) -> u64;
}

impl MetadataExt for Metadata {
    fn as_raw_stat(&self) -> &raw::stat {
        unsafe {
            &*(self.as_inner().as_inner() as *const libc::stat64
                                          as *const raw::stat)
        }
    }
    fn st_dev(&self) -> u64 {
        self.as_inner().as_inner().st_dev as u64
    }
    fn st_ino(&self) -> u64 {
        self.as_inner().as_inner().st_ino as u64
    }
    fn st_mode(&self) -> u32 {
        self.as_inner().as_inner().st_mode as u32
    }
    fn st_nlink(&self) -> u64 {
        self.as_inner().as_inner().st_nlink as u64
    }
    fn st_uid(&self) -> u32 {
        self.as_inner().as_inner().st_uid as u32
    }
    fn st_gid(&self) -> u32 {
        self.as_inner().as_inner().st_gid as u32
    }
    fn st_rdev(&self) -> u64 {
        self.as_inner().as_inner().st_rdev as u64
    }
    fn st_size(&self) -> u64 {
        self.as_inner().as_inner().st_size as u64
    }
    fn st_atime(&self) -> i64 {
        self.as_inner().as_inner().st_atime as i64
    }
    fn st_atime_nsec(&self) -> i64 {
        self.as_inner().as_inner().st_atime_nsec as i64
    }
    fn st_mtime(&self) -> i64 {
        self.as_inner().as_inner().st_mtime as i64
    }
    fn st_mtime_nsec(&self) -> i64 {
        self.as_inner().as_inner().st_mtime_nsec as i64
    }
    fn st_ctime(&self) -> i64 {
        self.as_inner().as_inner().st_ctime as i64
    }
    fn st_ctime_nsec(&self) -> i64 {
        self.as_inner().as_inner().st_ctime_nsec as i64
    }
    fn st_blksize(&self) -> u64 {
        self.as_inner().as_inner().st_blksize as u64
    }
    fn st_blocks(&self) -> u64 {
        self.as_inner().as_inner().st_blocks as u64
    }
}