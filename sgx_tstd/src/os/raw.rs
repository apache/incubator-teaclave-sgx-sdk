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

//! Linux-specific raw type definitions.

pub type c_char = i8;
pub type c_schar = i8;
pub type c_uchar = u8;
pub type c_short = i16;
pub type c_ushort = u16;
pub type c_int = i32;
pub type c_uint = u32;
#[cfg(target_pointer_width = "32")]
pub type c_long = i32;
#[cfg(target_pointer_width = "32")]
pub type c_ulong = u32;
#[cfg(target_pointer_width = "64")]
pub type c_long = i64;
#[cfg(target_pointer_width = "64")]
pub type c_ulong = u64;
pub type c_longlong = i64;
pub type c_ulonglong = u64;
pub type c_float = f32;
pub type c_double = f64;

#[doc(no_inline)]
pub use core::ffi::c_void;

pub type dev_t = u64;
pub type mode_t = u32;
pub type pthread_t = *mut c_void;

pub use self::arch::{off_t, ino_t, nlink_t, blksize_t, blkcnt_t, stat, time_t};

mod arch {
    use crate::os::raw::{c_long, c_int};

    pub type blkcnt_t = u64;
    pub type blksize_t = u64;
    pub type ino_t = u64;
    pub type nlink_t = u64;
    pub type off_t = i64;
    pub type time_t = i64;

    #[repr(C)]
    #[derive(Clone)]
    pub struct stat {
        pub st_dev: u64,
        pub st_ino: u64,
        pub st_nlink: u64,
        pub st_mode: u32,
        pub st_uid: u32,
        pub st_gid: u32,
        pub __pad0: c_int,
        pub st_rdev: u64,
        pub st_size: i64,
        pub st_blksize: i64,
        pub st_blocks: i64,
        pub st_atime: i64,
        pub st_atime_nsec: c_long,
        pub st_mtime: i64,
        pub st_mtime_nsec: c_long,
        pub st_ctime: i64,
        pub st_ctime_nsec: c_long,
        pub __unused: [c_long; 3],
    }
}