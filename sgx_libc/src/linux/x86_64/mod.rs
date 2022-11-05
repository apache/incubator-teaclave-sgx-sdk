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

//! This mod provides the interface connecting Rust's memory management system
//! to the Intel's SGX SDK's malloc system.
//!
//! It is a c-style interface and self-explained. Currently we don't have much
//! time for documenting it.

pub use sgx_types::time_t;
pub use sgx_types::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint,
    c_ulong, c_ulonglong, c_ushort, c_void, intmax_t, uintmax_t,
};
pub use sgx_types::{int16_t, int32_t, int64_t, int8_t, uint16_t, uint32_t, uint64_t, uint8_t};
pub use sgx_types::{intptr_t, ptrdiff_t, size_t, ssize_t, uintptr_t};

use core::mem;
use core::ptr;
use sgx_types::{
    sgx_thread_cond_attr_t, sgx_thread_cond_t, sgx_thread_mutex_attr_t, sgx_thread_mutex_t,
};

extern "C" {
    pub fn calloc(nobj: size_t, size: size_t) -> *mut c_void;
    pub fn malloc(size: size_t) -> *mut c_void;
    pub fn realloc(p: *mut c_void, size: size_t) -> *mut c_void;
    pub fn free(p: *mut c_void);
    pub fn memalign(align: size_t, size: size_t) -> *mut c_void;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: size_t) -> *mut c_void;
    pub fn consttime_memequal(b1: *const c_void, b2: *const c_void, len: size_t) -> c_int;
}

#[link(name = "sgx_pthread")]
extern "C" {
    pub fn pthread_create(
        native: *mut pthread_t,
        attr: *const pthread_attr_t,
        f: extern "C" fn(*mut c_void) -> *mut c_void,
        value: *mut c_void,
    ) -> c_int;
    pub fn pthread_join(native: pthread_t, value: *mut *mut c_void) -> c_int;
    pub fn pthread_exit(value: *mut c_void);
    pub fn pthread_self() -> pthread_t;
    pub fn pthread_equal(t1: pthread_t, t2: pthread_t) -> c_int;

    pub fn pthread_once(once_control: *mut pthread_once_t, init_routine: extern "C" fn()) -> c_int;

    pub fn pthread_key_create(
        key: *mut pthread_key_t,
        dtor: Option<unsafe extern "C" fn(*mut c_void)>,
    ) -> c_int;
    pub fn pthread_key_delete(key: pthread_key_t) -> c_int;
    pub fn pthread_getspecific(key: pthread_key_t) -> *mut c_void;
    pub fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int;

    pub fn pthread_mutex_init(
        lock: *mut pthread_mutex_t,
        attr: *const pthread_mutexattr_t,
    ) -> c_int;
    pub fn pthread_mutex_destroy(lock: *mut pthread_mutex_t) -> c_int;
    pub fn pthread_mutex_lock(lock: *mut pthread_mutex_t) -> c_int;
    pub fn pthread_mutex_trylock(lock: *mut pthread_mutex_t) -> c_int;
    pub fn pthread_mutex_unlock(lock: *mut pthread_mutex_t) -> c_int;

    pub fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const pthread_condattr_t) -> c_int;
    pub fn pthread_cond_wait(cond: *mut pthread_cond_t, lock: *mut pthread_mutex_t) -> c_int;
    pub fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int;
    pub fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int;
    pub fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int;
}

pub type exit_function_t = extern "C" fn();

#[link(name = "sgx_trts")]
extern "C" {
    pub fn abort() -> !;
    pub fn atexit(fun: exit_function_t) -> c_int;
}

#[link(name = "sgx_tstdc")]
extern "C" {
    //pub fn memchr(s: *const c_void, c: c_int, n: size_t) -> *mut c_void;
    //pub fn memrchr(cx: *const c_void, c: c_int, n: size_t) -> *mut c_void;
    pub fn strlen(s: *const c_char) -> size_t;
    pub fn malloc_usable_size(ptr: *const c_void) -> size_t;
}

#[link(name = "sgx_tstdc")]
extern "C" {
    #[cfg_attr(target_os = "linux", link_name = "__errno_location")]
    fn errno_location() -> *mut c_int;
    fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: size_t) -> c_int;
}

/// Get the last error number.
pub fn errno() -> i32 {
    unsafe { *errno_location() }
}

/// Set the last error number.
pub fn set_errno(e: i32) {
    unsafe { *errno_location() = e as c_int }
}

/// Gets a detailed string description for the given error number.
pub unsafe fn error_string(errno: i32, buf: &mut [i8]) -> i32 {
    let p = buf.as_mut_ptr();
    strerror_r(errno as c_int, p as *mut c_char, buf.len())
}

pub unsafe fn memchr(s: *const u8, c: u8, n: usize) -> *const u8 {
    let mut ret = ptr::null();
    let mut p = s;
    for _ in 0..n {
        if *p == c {
            ret = p;
            break;
        }
        p = p.offset(1);
    }
    ret
}

pub unsafe fn memrchr(s: *const u8, c: u8, n: usize) -> *const u8 {
    if n == 0 {
        return ptr::null();
    }
    let mut ret = ptr::null();
    let mut p: *const u8 = (s as usize + (n - 1)) as *const u8;
    for _ in 0..n {
        if *p == c {
            ret = p;
            break;
        }
        p = p.offset(-1);
    }
    ret
}

// intentionally not public, only used for fd_set
cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        const ULONG_SIZE: usize = 32;
    } else if #[cfg(target_pointer_width = "64")] {
        const ULONG_SIZE: usize = 64;
    } else {
        // Unknown target_pointer_width
    }
}

pub type in_addr_t = u32;
pub type in_port_t = u16;
pub type sa_family_t = u16;
pub type socklen_t = u32;
pub type off64_t = i64;
pub type clockid_t = i32;
pub type suseconds_t = i64;
pub type dev_t = u64;
pub type mode_t = u32;
pub type blkcnt_t = i64;
pub type blkcnt64_t = i64;
pub type blksize_t = i64;
pub type ino_t = u64;
pub type nlink_t = u64;
pub type off_t = i64;
pub type loff_t = i64;
pub type uid_t = u32;
pub type gid_t = u32;
pub type ino64_t = u64;
pub type nfds_t = c_ulong;
pub type pid_t = i32;

pub type sighandler_t = size_t;

pub type pthread_t = *mut c_void;
pub type pthread_attr_t = *mut c_void;
pub type pthread_mutex_t = *mut sgx_thread_mutex_t;
pub type pthread_mutexattr_t = *mut sgx_thread_mutex_attr_t;
pub type pthread_cond_t = *mut sgx_thread_cond_t;
pub type pthread_condattr_t = *mut sgx_thread_cond_attr_t;
pub type pthread_key_t = c_int;

pub const PTHREAD_NEEDS_INIT: c_int = 0;
pub const PTHREAD_DONE_INIT: c_int = 0;
pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = ptr::null_mut();
pub const PTHREAD_COND_INITIALIZER: pthread_cond_t = ptr::null_mut();
pub const PTHREAD_ONCE_INIT: pthread_once_t = pthread_once_t {
    state: PTHREAD_NEEDS_INIT,
    mutex: PTHREAD_MUTEX_INITIALIZER,
};

#[derive(Copy, Clone, Debug)]
pub enum DIR {}

s! {
    pub struct stat {
        pub st_dev: dev_t,
        pub st_ino: ino_t,
        pub st_nlink: nlink_t,
        pub st_mode: mode_t,
        pub st_uid: uid_t,
        pub st_gid: gid_t,
        __pad0: c_int,
        pub st_rdev: dev_t,
        pub st_size: off_t,
        pub st_blksize: blksize_t,
        pub st_blocks: blkcnt_t,
        pub st_atime: time_t,
        pub st_atime_nsec: i64,
        pub st_mtime: time_t,
        pub st_mtime_nsec: i64,
        pub st_ctime: time_t,
        pub st_ctime_nsec: i64,
        __unused: [i64; 3],
    }

    pub struct stat64 {
        pub st_dev: dev_t,
        pub st_ino: ino64_t,
        pub st_nlink: nlink_t,
        pub st_mode: mode_t,
        pub st_uid: uid_t,
        pub st_gid: gid_t,
        __pad0: c_int,
        pub st_rdev: dev_t,
        pub st_size: off_t,
        pub st_blksize: blksize_t,
        pub st_blocks: blkcnt64_t,
        pub st_atime: time_t,
        pub st_atime_nsec: i64,
        pub st_mtime: time_t,
        pub st_mtime_nsec: i64,
        pub st_ctime: time_t,
        pub st_ctime_nsec: i64,
        __reserved: [i64; 3],
    }

    pub struct timeval {
        pub tv_sec: time_t,
        pub tv_usec: suseconds_t,
    }

    pub struct timespec {
        pub tv_sec: time_t,
        pub tv_nsec: c_long,
    }

    pub struct sockaddr {
        pub sa_family: sa_family_t,
        pub sa_data: [c_char; 14],
    }

    pub struct sockaddr_in {
        pub sin_family: sa_family_t,
        pub sin_port: in_port_t,
        pub sin_addr: in_addr,
        pub sin_zero: [u8; 8],
    }

    pub struct sockaddr_in6 {
        pub sin6_family: sa_family_t,
        pub sin6_port: in_port_t,
        pub sin6_flowinfo: u32,
        pub sin6_addr: in6_addr,
        pub sin6_scope_id: u32,
    }

    pub struct sockaddr_un {
        pub sun_family: sa_family_t,
        pub sun_path: [c_char; 108],
    }

    pub struct sockaddr_storage {
        pub ss_family: sa_family_t,
        __ss_align: size_t,
        #[cfg(target_pointer_width = "32")]
        __ss_pad2: [u8; 128 - 2 * 4],
        #[cfg(target_pointer_width = "64")]
        __ss_pad2: [u8; 128 - 2 * 8],
    }

    pub struct addrinfo {
        pub ai_flags: c_int,
        pub ai_family: c_int,
        pub ai_socktype: c_int,
        pub ai_protocol: c_int,
        pub ai_addrlen: socklen_t,

        #[cfg(any(target_os = "linux",
                  target_os = "emscripten",
                  target_os = "fuchsia"))]
        pub ai_addr: *mut sockaddr,

        pub ai_canonname: *mut c_char,

        #[cfg(target_os = "android")]
        pub ai_addr: *mut sockaddr,

        pub ai_next: *mut addrinfo,
    }

    pub struct sockaddr_nl {
        pub nl_family: sa_family_t,
        nl_pad: c_ushort,
        pub nl_pid: u32,
        pub nl_groups: u32,
    }

    pub struct sockaddr_ll {
        pub sll_family: c_ushort,
        pub sll_protocol: c_ushort,
        pub sll_ifindex: c_int,
        pub sll_hatype: c_ushort,
        pub sll_pkttype: c_uchar,
        pub sll_halen: c_uchar,
        pub sll_addr: [c_uchar; 8],
    }

    pub struct fd_set {
        fds_bits: [c_ulong; FD_SETSIZE / ULONG_SIZE],
    }

    pub struct tm {
        pub tm_sec: c_int,
        pub tm_min: c_int,
        pub tm_hour: c_int,
        pub tm_mday: c_int,
        pub tm_mon: c_int,
        pub tm_year: c_int,
        pub tm_wday: c_int,
        pub tm_yday: c_int,
        pub tm_isdst: c_int,
        pub tm_gmtoff: c_long,
        pub tm_zone: *const c_char,
    }

    #[cfg_attr(any(all(target_arch = "x86",
                       not(target_env = "musl"),
                       not(target_os = "android")),
                   target_arch = "x86_64"),
               repr(packed))]
    pub struct epoll_event {
        pub events: uint32_t,
        pub u64: uint64_t,
    }

    #[cfg_attr(target_os = "netbsd", repr(packed))]
    pub struct in_addr {
        pub s_addr: in_addr_t,
    }

    #[cfg_attr(feature = "align", repr(align(4)))]
    pub struct in6_addr {
        pub s6_addr: [u8; 16],
        #[cfg(not(feature = "align"))]
        __align: [u32; 0],
    }

    pub struct ip_mreq {
        pub imr_multiaddr: in_addr,
        pub imr_interface: in_addr,
    }

    pub struct ipv6_mreq {
        pub ipv6mr_multiaddr: in6_addr,
        #[cfg(target_os = "android")]
        pub ipv6mr_interface: c_int,
        #[cfg(not(target_os = "android"))]
        pub ipv6mr_interface: c_uint,
    }

    pub struct hostent {
        pub h_name: *mut c_char,
        pub h_aliases: *mut *mut c_char,
        pub h_addrtype: c_int,
        pub h_length: c_int,
        pub h_addr_list: *mut *mut c_char,
    }

    pub struct iovec {
        pub iov_base: *mut c_void,
        pub iov_len: size_t,
    }

    pub struct pollfd {
        pub fd: c_int,
        pub events: c_short,
        pub revents: c_short,
    }

    pub struct winsize {
        pub ws_row: c_ushort,
        pub ws_col: c_ushort,
        pub ws_xpixel: c_ushort,
        pub ws_ypixel: c_ushort,
    }

    pub struct linger {
        pub l_onoff: c_int,
        pub l_linger: c_int,
    }

    pub struct sigval {
        // Actually a union of an int and a void*
        pub sival_ptr: *mut c_void
    }

    pub struct msghdr {
        pub msg_name: *mut c_void,
        pub msg_namelen: socklen_t,
        pub msg_iov: *mut iovec,
        pub msg_iovlen: size_t,
        pub msg_control: *mut c_void,
        pub msg_controllen: size_t,
        pub msg_flags: c_int,
    }

    pub struct cmsghdr {
        pub cmsg_len: size_t,
        pub cmsg_level: c_int,
        pub cmsg_type: c_int,
    }

    pub struct dirent {
        pub d_ino: ino_t,
        pub d_off: off_t,
        pub d_reclen: c_ushort,
        pub d_type: c_uchar,
        pub d_name: [c_char; 256],
    }

    pub struct dirent64 {
        pub d_ino: ino64_t,
        pub d_off: off64_t,
        pub d_reclen: c_ushort,
        pub d_type: c_uchar,
        pub d_name: [c_char; 256],
    }

    pub struct passwd {
        pub pw_name: *mut c_char,
        pub pw_passwd: *mut c_char,
        pub pw_uid: uid_t,
        pub pw_gid: gid_t,
        pub pw_gecos: *mut c_char,
        pub pw_dir: *mut c_char,
        pub pw_shell: *mut c_char,
    }

    pub struct cpu_set_t {
        #[cfg(all(target_pointer_width = "32",
                  not(target_arch = "x86_64")))]
        bits: [u32; 32],
        #[cfg(not(all(target_pointer_width = "32",
                      not(target_arch = "x86_64"))))]
        bits: [u64; 16],
    }

    pub struct ucred {
        pub pid: pid_t,
        pub uid: uid_t,
        pub gid: gid_t,
    }

    pub struct pthread_once_t {
        pub state: c_int,
        pub mutex: pthread_mutex_t,
    }

    pub struct sigset_t {
        __val: [u64; 16],
    }

    pub struct sigaction {
        pub sa_sigaction: sighandler_t,
        pub sa_mask: sigset_t,
        pub sa_flags: c_int,
        pub sa_restorer: Option<extern fn()>,
    }
    pub struct siginfo_t {
        pub si_signo: c_int,
        pub si_errno: c_int,
        pub si_code: c_int,
        pub _pad: [c_int; 29],
        _align: [usize; 0],
    }
}

pub const SPLICE_F_MOVE: c_uint = 0x01;
pub const SPLICE_F_NONBLOCK: c_uint = 0x02;
pub const SPLICE_F_MORE: c_uint = 0x04;
pub const SPLICE_F_GIFT: c_uint = 0x08;

pub const AT_FDCWD: c_int = -100;
pub const AT_SYMLINK_NOFOLLOW: c_int = 0x100;
pub const AT_REMOVEDIR: c_int = 0x200;
pub const AT_SYMLINK_FOLLOW: c_int = 0x400;

pub const UTIME_OMIT: c_long = 1073741822;
pub const UTIME_NOW: c_long = 1073741823;

pub const CLOCK_REALTIME: clockid_t = 0;
pub const CLOCK_MONOTONIC: clockid_t = 1;
pub const CLOCK_PROCESS_CPUTIME_ID: clockid_t = 2;
pub const CLOCK_THREAD_CPUTIME_ID: clockid_t = 3;
pub const CLOCK_MONOTONIC_RAW: clockid_t = 4;
pub const CLOCK_REALTIME_COARSE: clockid_t = 5;
pub const CLOCK_MONOTONIC_COARSE: clockid_t = 6;
pub const CLOCK_BOOTTIME: clockid_t = 7;
pub const CLOCK_REALTIME_ALARM: clockid_t = 8;
pub const CLOCK_BOOTTIME_ALARM: clockid_t = 9;
pub const DT_UNKNOWN: u8 = 0;
pub const DT_FIFO: u8 = 1;
pub const DT_CHR: u8 = 2;
pub const DT_DIR: u8 = 4;
pub const DT_BLK: u8 = 6;
pub const DT_REG: u8 = 8;
pub const DT_LNK: u8 = 10;
pub const DT_SOCK: u8 = 12;

pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;

pub const O_APPEND: c_int = 1024;
pub const O_CREAT: c_int = 64;
pub const O_EXCL: c_int = 128;
pub const O_NOCTTY: c_int = 256;
pub const O_NONBLOCK: c_int = 2048;
pub const O_SYNC: c_int = 1_052_672;
pub const O_RSYNC: c_int = 1_052_672;
pub const O_DSYNC: c_int = 4096;
pub const O_FSYNC: c_int = 0x0010_1000;
pub const O_NOATIME: c_int = 0o1_000_000;
pub const O_PATH: c_int = 0o10_000_000;

pub const O_ASYNC: c_int = 0x2000;
pub const O_NDELAY: c_int = 0x800;

pub const O_DIRECT: c_int = 0x4000;
pub const O_DIRECTORY: c_int = 0x10000;
pub const O_NOFOLLOW: c_int = 0x20000;

pub const EFD_NONBLOCK: c_int = 0x800;

pub const F_GETLK: c_int = 5;
pub const F_GETOWN: c_int = 9;
pub const F_SETOWN: c_int = 8;
pub const F_SETLK: c_int = 6;
pub const F_SETLKW: c_int = 7;

pub const SFD_NONBLOCK: c_int = 0x0800;

pub const SFD_CLOEXEC: c_int = 0x0008_0000;

pub const NCCS: usize = 32;

pub const O_TRUNC: c_int = 512;

pub const O_CLOEXEC: c_int = 0x80000;

pub const O_RDONLY: c_int = 0;
pub const O_WRONLY: c_int = 1;
pub const O_RDWR: c_int = 2;
pub const O_ACCMODE: c_int = 3;

pub const SOCK_CLOEXEC: c_int = O_CLOEXEC;

pub const S_IFIFO: mode_t = 4096;
pub const S_IFCHR: mode_t = 8192;
pub const S_IFBLK: mode_t = 24576;
pub const S_IFDIR: mode_t = 16384;
pub const S_IFREG: mode_t = 32768;
pub const S_IFLNK: mode_t = 40960;
pub const S_IFSOCK: mode_t = 49152;
pub const S_IFMT: mode_t = 61440;
pub const S_IRWXU: mode_t = 448;
pub const S_IXUSR: mode_t = 64;
pub const S_IWUSR: mode_t = 128;
pub const S_IRUSR: mode_t = 256;
pub const S_IRWXG: mode_t = 56;
pub const S_IXGRP: mode_t = 8;
pub const S_IWGRP: mode_t = 16;
pub const S_IRGRP: mode_t = 32;
pub const S_IRWXO: mode_t = 7;
pub const S_IXOTH: mode_t = 1;
pub const S_IWOTH: mode_t = 2;
pub const S_IROTH: mode_t = 4;
pub const F_OK: c_int = 0;
pub const R_OK: c_int = 4;
pub const W_OK: c_int = 2;
pub const X_OK: c_int = 1;
pub const SIGHUP: c_int = 1;
pub const SIGINT: c_int = 2;
pub const SIGQUIT: c_int = 3;
pub const SIGILL: c_int = 4;
pub const SIGABRT: c_int = 6;
pub const SIGFPE: c_int = 8;
pub const SIGKILL: c_int = 9;
pub const SIGSEGV: c_int = 11;
pub const SIGPIPE: c_int = 13;
pub const SIGALRM: c_int = 14;
pub const SIGTERM: c_int = 15;

pub const PROT_NONE: c_int = 0;
pub const PROT_READ: c_int = 1;
pub const PROT_WRITE: c_int = 2;
pub const PROT_EXEC: c_int = 4;

pub const SCM_RIGHTS: c_int = 0x01;
pub const SCM_CREDENTIALS: c_int = 0x02;

pub const SOL_SOCKET: c_int = 1;

pub const SO_REUSEADDR: c_int = 2;
pub const SO_TYPE: c_int = 3;
pub const SO_ERROR: c_int = 4;
pub const SO_DONTROUTE: c_int = 5;
pub const SO_BROADCAST: c_int = 6;
pub const SO_SNDBUF: c_int = 7;
pub const SO_RCVBUF: c_int = 8;
pub const SO_SNDBUFFORCE: c_int = 32;
pub const SO_RCVBUFFORCE: c_int = 33;
pub const SO_KEEPALIVE: c_int = 9;
pub const SO_OOBINLINE: c_int = 10;
pub const SO_NO_CHECK: c_int = 11;
pub const SO_PRIORITY: c_int = 12;
pub const SO_LINGER: c_int = 13;
pub const SO_BSDCOMPAT: c_int = 14;
pub const SO_REUSEPORT: c_int = 15;
pub const SO_PASSCRED: c_int = 16;
pub const SO_PEERCRED: c_int = 17;
pub const SO_RCVLOWAT: c_int = 18;
pub const SO_SNDLOWAT: c_int = 19;
pub const SO_RCVTIMEO: c_int = 20;
pub const SO_SNDTIMEO: c_int = 21;
pub const SO_SECURITY_AUTHENTICATION: c_int = 22;
pub const SO_SECURITY_ENCRYPTION_TRANSPORT: c_int = 23;
pub const SO_SECURITY_ENCRYPTION_NETWORK: c_int = 24;
pub const SO_BINDTODEVICE: c_int = 25;
pub const SO_ATTACH_FILTER: c_int = 26;
pub const SO_DETACH_FILTER: c_int = 27;
pub const SO_GET_FILTER: c_int = SO_ATTACH_FILTER;
pub const SO_PEERNAME: c_int = 28;
pub const SO_TIMESTAMP: c_int = 29;
pub const SO_ACCEPTCONN: c_int = 30;
pub const SO_PEERSEC: c_int = 31;
pub const SO_PASSSEC: c_int = 34;
pub const SO_TIMESTAMPNS: c_int = 35;
pub const SCM_TIMESTAMPNS: c_int = SO_TIMESTAMPNS;
pub const SO_MARK: c_int = 36;
pub const SO_TIMESTAMPING: c_int = 37;
pub const SCM_TIMESTAMPING: c_int = SO_TIMESTAMPING;
pub const SO_PROTOCOL: c_int = 38;
pub const SO_DOMAIN: c_int = 39;
pub const SO_RXQ_OVFL: c_int = 40;
pub const SO_WIFI_STATUS: c_int = 41;
pub const SCM_WIFI_STATUS: c_int = SO_WIFI_STATUS;
pub const SO_PEEK_OFF: c_int = 42;
pub const SO_NOFCS: c_int = 43;
pub const SO_LOCK_FILTER: c_int = 44;
pub const SO_SELECT_ERR_QUEUE: c_int = 45;
pub const SO_BUSY_POLL: c_int = 46;
pub const SO_MAX_PACING_RATE: c_int = 47;
pub const SO_BPF_EXTENSIONS: c_int = 48;
pub const SO_INCOMING_CPU: c_int = 49;
pub const SO_ATTACH_BPF: c_int = 50;
pub const SO_DETACH_BPF: c_int = SO_DETACH_FILTER;

pub const SOCK_NONBLOCK: c_int = O_NONBLOCK;

pub const MAP_ANON: c_int = 0x0020;
pub const MAP_ANONYMOUS: c_int = 0x0020;
pub const MAP_DENYWRITE: c_int = 0x0800;
pub const MAP_EXECUTABLE: c_int = 0x01000;
pub const MAP_POPULATE: c_int = 0x08000;
pub const MAP_NONBLOCK: c_int = 0x0001_0000;
pub const MAP_STACK: c_int = 0x0002_0000;

pub const MAP_FILE: c_int = 0x0000;
pub const MAP_SHARED: c_int = 0x0001;
pub const MAP_PRIVATE: c_int = 0x0002;
pub const MAP_FIXED: c_int = 0x0010;
pub const MAP_FAILED: *mut c_void = !0 as *mut c_void;

// MS_ flags for msync(2)
pub const MS_ASYNC: c_int = 0x0001;
pub const MS_INVALIDATE: c_int = 0x0002;
pub const MS_SYNC: c_int = 0x0004;

pub const SOCK_STREAM: c_int = 1;
pub const SOCK_DGRAM: c_int = 2;
pub const SOCK_SEQPACKET: c_int = 5;
pub const SOCK_DCCP: c_int = 6;
pub const SOCK_PACKET: c_int = 10;

pub const TCP_COOKIE_TRANSACTIONS: c_int = 15;
pub const TCP_THIN_LINEAR_TIMEOUTS: c_int = 16;
pub const TCP_THIN_DUPACK: c_int = 17;
pub const TCP_USER_TIMEOUT: c_int = 18;
pub const TCP_REPAIR: c_int = 19;
pub const TCP_REPAIR_QUEUE: c_int = 20;
pub const TCP_QUEUE_SEQ: c_int = 21;
pub const TCP_REPAIR_OPTIONS: c_int = 22;
pub const TCP_FASTOPEN: c_int = 23;
pub const TCP_TIMESTAMP: c_int = 24;

/* DCCP socket options */
pub const DCCP_SOCKOPT_PACKET_SIZE: c_int = 1;
pub const DCCP_SOCKOPT_SERVICE: c_int = 2;
pub const DCCP_SOCKOPT_CHANGE_L: c_int = 3;
pub const DCCP_SOCKOPT_CHANGE_R: c_int = 4;
pub const DCCP_SOCKOPT_GET_CUR_MPS: c_int = 5;
pub const DCCP_SOCKOPT_SERVER_TIMEWAIT: c_int = 6;
pub const DCCP_SOCKOPT_SEND_CSCOV: c_int = 10;
pub const DCCP_SOCKOPT_RECV_CSCOV: c_int = 11;
pub const DCCP_SOCKOPT_AVAILABLE_CCIDS: c_int = 12;
pub const DCCP_SOCKOPT_CCID: c_int = 13;
pub const DCCP_SOCKOPT_TX_CCID: c_int = 14;
pub const DCCP_SOCKOPT_RX_CCID: c_int = 15;
pub const DCCP_SOCKOPT_QPOLICY_ID: c_int = 16;
pub const DCCP_SOCKOPT_QPOLICY_TXQLEN: c_int = 17;
pub const DCCP_SOCKOPT_CCID_RX_INFO: c_int = 128;
pub const DCCP_SOCKOPT_CCID_TX_INFO: c_int = 192;

/// maximum number of services provided on the same listening port
pub const DCCP_SERVICE_LIST_MAX_LEN: c_int = 32;

pub const FIOCLEX: c_int = 0x5451;
pub const FIONBIO: c_int = 0x5421;

pub const F_DUPFD: c_int = 0;
pub const F_GETFD: c_int = 1;
pub const F_SETFD: c_int = 2;
pub const F_GETFL: c_int = 3;
pub const F_SETFL: c_int = 4;

// Linux-specific fcntls
pub const F_SETLEASE: c_int = 1024;
pub const F_GETLEASE: c_int = 1025;
pub const F_NOTIFY: c_int = 1026;
pub const F_CANCELLK: c_int = 1029;
pub const F_DUPFD_CLOEXEC: c_int = 1030;
pub const F_SETPIPE_SZ: c_int = 1031;
pub const F_GETPIPE_SZ: c_int = 1032;
pub const F_ADD_SEALS: c_int = 1033;
pub const F_GET_SEALS: c_int = 1034;

pub const F_SEAL_SEAL: c_int = 0x0001;
pub const F_SEAL_SHRINK: c_int = 0x0002;
pub const F_SEAL_GROW: c_int = 0x0004;

pub const EXIT_FAILURE: c_int = 1;
pub const EXIT_SUCCESS: c_int = 0;
pub const EOF: c_int = -1;
pub const SEEK_SET: c_int = 0;
pub const SEEK_CUR: c_int = 1;
pub const SEEK_END: c_int = 2;
pub const _IOFBF: c_int = 0;
pub const _IONBF: c_int = 2;
pub const _IOLBF: c_int = 1;

pub const FILENAME_MAX: c_uint = 4096;
pub const FOPEN_MAX: c_uint = 16;

pub const IFF_UP: c_int = 0x1;
pub const IFF_BROADCAST: c_int = 0x2;
pub const IFF_DEBUG: c_int = 0x4;
pub const IFF_LOOPBACK: c_int = 0x8;
pub const IFF_POINTOPOINT: c_int = 0x10;
pub const IFF_NOTRAILERS: c_int = 0x20;
pub const IFF_RUNNING: c_int = 0x40;
pub const IFF_NOARP: c_int = 0x80;
pub const IFF_PROMISC: c_int = 0x100;
pub const IFF_ALLMULTI: c_int = 0x200;
pub const IFF_MASTER: c_int = 0x400;
pub const IFF_SLAVE: c_int = 0x800;
pub const IFF_MULTICAST: c_int = 0x1000;
pub const IFF_PORTSEL: c_int = 0x2000;
pub const IFF_AUTOMEDIA: c_int = 0x4000;
pub const IFF_DYNAMIC: c_int = 0x8000;

pub const SOL_IP: c_int = 0;
pub const SOL_TCP: c_int = 6;
pub const SOL_UDP: c_int = 17;
pub const SOL_IPV6: c_int = 41;
pub const SOL_ICMPV6: c_int = 58;
pub const SOL_RAW: c_int = 255;
pub const SOL_DECNET: c_int = 261;
pub const SOL_X25: c_int = 262;
pub const SOL_PACKET: c_int = 263;
pub const SOL_ATM: c_int = 264;
pub const SOL_AAL: c_int = 265;
pub const SOL_IRDA: c_int = 266;
pub const SOL_NETBEUI: c_int = 267;
pub const SOL_LLC: c_int = 268;
pub const SOL_DCCP: c_int = 269;
pub const SOL_NETLINK: c_int = 270;
pub const SOL_TIPC: c_int = 271;

pub const AF_UNSPEC: c_int = 0;
pub const AF_UNIX: c_int = 1;
pub const AF_LOCAL: c_int = 1;
pub const AF_INET: c_int = 2;
pub const AF_AX25: c_int = 3;
pub const AF_IPX: c_int = 4;
pub const AF_APPLETALK: c_int = 5;
pub const AF_NETROM: c_int = 6;
pub const AF_BRIDGE: c_int = 7;
pub const AF_ATMPVC: c_int = 8;
pub const AF_X25: c_int = 9;
pub const AF_INET6: c_int = 10;
pub const AF_ROSE: c_int = 11;
pub const AF_DECnet: c_int = 12;
pub const AF_NETBEUI: c_int = 13;
pub const AF_SECURITY: c_int = 14;
pub const AF_KEY: c_int = 15;
pub const AF_NETLINK: c_int = 16;
pub const AF_ROUTE: c_int = AF_NETLINK;
pub const AF_PACKET: c_int = 17;
pub const AF_ASH: c_int = 18;
pub const AF_ECONET: c_int = 19;
pub const AF_ATMSVC: c_int = 20;
pub const AF_RDS: c_int = 21;
pub const AF_SNA: c_int = 22;
pub const AF_IRDA: c_int = 23;
pub const AF_PPPOX: c_int = 24;
pub const AF_WANPIPE: c_int = 25;
pub const AF_LLC: c_int = 26;
pub const AF_CAN: c_int = 29;
pub const AF_TIPC: c_int = 30;
pub const AF_BLUETOOTH: c_int = 31;
pub const AF_IUCV: c_int = 32;
pub const AF_RXRPC: c_int = 33;
pub const AF_ISDN: c_int = 34;
pub const AF_PHONET: c_int = 35;
pub const AF_IEEE802154: c_int = 36;
pub const AF_CAIF: c_int = 37;
pub const AF_ALG: c_int = 38;

pub const PF_UNSPEC: c_int = AF_UNSPEC;
pub const PF_UNIX: c_int = AF_UNIX;
pub const PF_LOCAL: c_int = AF_LOCAL;
pub const PF_INET: c_int = AF_INET;
pub const PF_AX25: c_int = AF_AX25;
pub const PF_IPX: c_int = AF_IPX;
pub const PF_APPLETALK: c_int = AF_APPLETALK;
pub const PF_NETROM: c_int = AF_NETROM;
pub const PF_BRIDGE: c_int = AF_BRIDGE;
pub const PF_ATMPVC: c_int = AF_ATMPVC;
pub const PF_X25: c_int = AF_X25;
pub const PF_INET6: c_int = AF_INET6;
pub const PF_ROSE: c_int = AF_ROSE;
pub const PF_DECnet: c_int = AF_DECnet;
pub const PF_NETBEUI: c_int = AF_NETBEUI;
pub const PF_SECURITY: c_int = AF_SECURITY;
pub const PF_KEY: c_int = AF_KEY;
pub const PF_NETLINK: c_int = AF_NETLINK;
pub const PF_ROUTE: c_int = AF_ROUTE;
pub const PF_PACKET: c_int = AF_PACKET;
pub const PF_ASH: c_int = AF_ASH;
pub const PF_ECONET: c_int = AF_ECONET;
pub const PF_ATMSVC: c_int = AF_ATMSVC;
pub const PF_RDS: c_int = AF_RDS;
pub const PF_SNA: c_int = AF_SNA;
pub const PF_IRDA: c_int = AF_IRDA;
pub const PF_PPPOX: c_int = AF_PPPOX;
pub const PF_WANPIPE: c_int = AF_WANPIPE;
pub const PF_LLC: c_int = AF_LLC;
pub const PF_CAN: c_int = AF_CAN;
pub const PF_TIPC: c_int = AF_TIPC;
pub const PF_BLUETOOTH: c_int = AF_BLUETOOTH;
pub const PF_IUCV: c_int = AF_IUCV;
pub const PF_RXRPC: c_int = AF_RXRPC;
pub const PF_ISDN: c_int = AF_ISDN;
pub const PF_PHONET: c_int = AF_PHONET;
pub const PF_IEEE802154: c_int = AF_IEEE802154;
pub const PF_CAIF: c_int = AF_CAIF;
pub const PF_ALG: c_int = AF_ALG;

pub const SOMAXCONN: c_int = 128;

pub const MSG_OOB: c_int = 1;
pub const MSG_PEEK: c_int = 2;
pub const MSG_DONTROUTE: c_int = 4;
pub const MSG_CTRUNC: c_int = 8;
pub const MSG_TRUNC: c_int = 0x20;
pub const MSG_DONTWAIT: c_int = 0x40;
pub const MSG_EOR: c_int = 0x80;
pub const MSG_WAITALL: c_int = 0x100;
pub const MSG_FIN: c_int = 0x200;
pub const MSG_SYN: c_int = 0x400;
pub const MSG_CONFIRM: c_int = 0x800;
pub const MSG_RST: c_int = 0x1000;
pub const MSG_ERRQUEUE: c_int = 0x2000;
pub const MSG_NOSIGNAL: c_int = 0x4000;
pub const MSG_MORE: c_int = 0x8000;
pub const MSG_WAITFORONE: c_int = 0x10000;
pub const MSG_FASTOPEN: c_int = 0x2000_0000;
pub const MSG_CMSG_CLOEXEC: c_int = 0x4000_0000;

pub const SCM_TIMESTAMP: c_int = SO_TIMESTAMP;

pub const SOCK_RAW: c_int = 3;
pub const SOCK_RDM: c_int = 4;
pub const IP_MULTICAST_IF: c_int = 32;
pub const IP_MULTICAST_TTL: c_int = 33;
pub const IP_MULTICAST_LOOP: c_int = 34;
pub const IP_TTL: c_int = 2;
pub const IP_HDRINCL: c_int = 3;
pub const IP_PKTINFO: c_int = 8;
pub const IP_ADD_MEMBERSHIP: c_int = 35;
pub const IP_DROP_MEMBERSHIP: c_int = 36;
pub const IP_TRANSPARENT: c_int = 19;
pub const IPV6_UNICAST_HOPS: c_int = 16;
pub const IPV6_MULTICAST_IF: c_int = 17;
pub const IPV6_MULTICAST_HOPS: c_int = 18;
pub const IPV6_MULTICAST_LOOP: c_int = 19;
pub const IPV6_ADD_MEMBERSHIP: c_int = 20;
pub const IPV6_DROP_MEMBERSHIP: c_int = 21;
pub const IPV6_V6ONLY: c_int = 26;
pub const IPV6_RECVPKTINFO: c_int = 49;
pub const IPV6_PKTINFO: c_int = 50;

pub const TCP_NODELAY: c_int = 1;
pub const TCP_MAXSEG: c_int = 2;
pub const TCP_CORK: c_int = 3;
pub const TCP_KEEPIDLE: c_int = 4;
pub const TCP_KEEPINTVL: c_int = 5;
pub const TCP_KEEPCNT: c_int = 6;
pub const TCP_SYNCNT: c_int = 7;
pub const TCP_LINGER2: c_int = 8;
pub const TCP_DEFER_ACCEPT: c_int = 9;
pub const TCP_WINDOW_CLAMP: c_int = 10;
pub const TCP_INFO: c_int = 11;
pub const TCP_QUICKACK: c_int = 12;
pub const TCP_CONGESTION: c_int = 13;

pub const SO_DEBUG: c_int = 1;

pub const SHUT_RD: c_int = 0;
pub const SHUT_WR: c_int = 1;
pub const SHUT_RDWR: c_int = 2;

pub const LOCK_SH: c_int = 1;
pub const LOCK_EX: c_int = 2;
pub const LOCK_NB: c_int = 4;
pub const LOCK_UN: c_int = 8;

pub const SS_ONSTACK: c_int = 1;
pub const SS_DISABLE: c_int = 2;

pub const PATH_MAX: c_int = 4096;

pub const UIO_MAXIOV: c_int = 1024;

pub const FD_SETSIZE: usize = 1024;
pub const FD_CLOEXEC: c_int = 0x1;

pub const EPOLLIN: c_int = 0x1;
pub const EPOLLPRI: c_int = 0x2;
pub const EPOLLOUT: c_int = 0x4;
pub const EPOLLRDNORM: c_int = 0x40;
pub const EPOLLRDBAND: c_int = 0x80;
pub const EPOLLWRNORM: c_int = 0x100;
pub const EPOLLWRBAND: c_int = 0x200;
pub const EPOLLMSG: c_int = 0x400;
pub const EPOLLERR: c_int = 0x8;
pub const EPOLLHUP: c_int = 0x10;
pub const EPOLLET: c_int = 0x8000_0000;
pub const EPOLLRDHUP: c_int = 0x2000;
pub const EPOLLEXCLUSIVE: c_int = 0x1000_0000;
pub const EPOLLWAKEUP: c_int = 0x2000_0000;
pub const EPOLLONESHOT: c_int = 0x4000_0000;
pub const EPOLL_CLOEXEC: c_int = 0x80000;

pub const EFD_CLOEXEC: c_int = 0x80000;

pub const EPOLL_CTL_ADD: c_int = 1;
pub const EPOLL_CTL_MOD: c_int = 3;
pub const EPOLL_CTL_DEL: c_int = 2;

pub const POLLIN: c_short = 0x1;
pub const POLLPRI: c_short = 0x2;
pub const POLLOUT: c_short = 0x4;
pub const POLLERR: c_short = 0x8;
pub const POLLHUP: c_short = 0x10;
pub const POLLNVAL: c_short = 0x20;
pub const POLLRDNORM: c_short = 0x040;
pub const POLLRDBAND: c_short = 0x080;

pub const AI_PASSIVE: c_int = 0x0001;
pub const AI_CANONNAME: c_int = 0x0002;
pub const AI_NUMERICHOST: c_int = 0x0004;
pub const AI_V4MAPPED: c_int = 0x0008;
pub const AI_ALL: c_int = 0x0010;
pub const AI_ADDRCONFIG: c_int = 0x0020;
pub const AI_NUMERICSERV: c_int = 0x0400;

pub const EAI_BADFLAGS: c_int = -1;
pub const EAI_NONAME: c_int = -2;
pub const EAI_AGAIN: c_int = -3;
pub const EAI_FAIL: c_int = -4;
pub const EAI_FAMILY: c_int = -6;
pub const EAI_SOCKTYPE: c_int = -7;
pub const EAI_SERVICE: c_int = -8;
pub const EAI_MEMORY: c_int = -10;
pub const EAI_OVERFLOW: c_int = -12;

pub const NI_NUMERICHOST: c_int = 1;
pub const NI_NUMERICSERV: c_int = 2;
pub const NI_NOFQDN: c_int = 4;
pub const NI_NAMEREQD: c_int = 8;
pub const NI_DGRAM: c_int = 16;

pub const SYNC_FILE_RANGE_WAIT_BEFORE: c_uint = 1;
pub const SYNC_FILE_RANGE_WRITE: c_uint = 2;
pub const SYNC_FILE_RANGE_WAIT_AFTER: c_uint = 4;

pub const EAI_SYSTEM: c_int = -11;

pub const AIO_CANCELED: c_int = 0;
pub const AIO_NOTCANCELED: c_int = 1;
pub const AIO_ALLDONE: c_int = 2;
pub const LIO_READ: c_int = 0;
pub const LIO_WRITE: c_int = 1;
pub const LIO_NOP: c_int = 2;
pub const LIO_WAIT: c_int = 0;
pub const LIO_NOWAIT: c_int = 1;

// netinet/in.h
// NOTE: These are in addition to the constants defined in src/unix/mod.rs

pub const IPPROTO_ICMP: c_int = 1;
pub const IPPROTO_ICMPV6: c_int = 58;
pub const IPPROTO_TCP: c_int = 6;
pub const IPPROTO_UDP: c_int = 17;
pub const IPPROTO_IP: c_int = 0;
pub const IPPROTO_IPV6: c_int = 41;

// IPPROTO_IP defined in src/unix/mod.rs
/// Hop-by-hop option header
pub const IPPROTO_HOPOPTS: c_int = 0;
// IPPROTO_ICMP defined in src/unix/mod.rs
/// group mgmt protocol
pub const IPPROTO_IGMP: c_int = 2;
/// for compatibility
pub const IPPROTO_IPIP: c_int = 4;
// IPPROTO_TCP defined in src/unix/mod.rs
/// exterior gateway protocol
pub const IPPROTO_EGP: c_int = 8;
/// pup
pub const IPPROTO_PUP: c_int = 12;
// IPPROTO_UDP defined in src/unix/mod.rs
/// xns idp
pub const IPPROTO_IDP: c_int = 22;
/// tp-4 w/ class negotiation
pub const IPPROTO_TP: c_int = 29;
/// DCCP
pub const IPPROTO_DCCP: c_int = 33;
// IPPROTO_IPV6 defined in src/unix/mod.rs
/// IP6 routing header
pub const IPPROTO_ROUTING: c_int = 43;
/// IP6 fragmentation header
pub const IPPROTO_FRAGMENT: c_int = 44;
/// resource reservation
pub const IPPROTO_RSVP: c_int = 46;
/// General Routing Encap.
pub const IPPROTO_GRE: c_int = 47;
/// IP6 Encap Sec. Payload
pub const IPPROTO_ESP: c_int = 50;
/// IP6 Auth Header
pub const IPPROTO_AH: c_int = 51;
// IPPROTO_ICMPV6 defined in src/unix/mod.rs
/// IP6 no next header
pub const IPPROTO_NONE: c_int = 59;
/// IP6 destination option
pub const IPPROTO_DSTOPTS: c_int = 60;
pub const IPPROTO_MTP: c_int = 92;
pub const IPPROTO_BEETPH: c_int = 94;
/// encapsulation header
pub const IPPROTO_ENCAP: c_int = 98;
/// Protocol indep. multicast
pub const IPPROTO_PIM: c_int = 103;
/// IP Payload Comp. Protocol
pub const IPPROTO_COMP: c_int = 108;
/// SCTP
pub const IPPROTO_SCTP: c_int = 132;
pub const IPPROTO_MH: c_int = 135;
pub const IPPROTO_UDPLITE: c_int = 136;
pub const IPPROTO_MPLS: c_int = 137;
/// raw IP packet
pub const IPPROTO_RAW: c_int = 255;
pub const IPPROTO_MAX: c_int = 256;

pub const L_tmpnam: c_uint = 20;
pub const _PC_LINK_MAX: c_int = 0;
pub const _PC_MAX_CANON: c_int = 1;
pub const _PC_MAX_INPUT: c_int = 2;
pub const _PC_NAME_MAX: c_int = 3;
pub const _PC_PATH_MAX: c_int = 4;
pub const _PC_PIPE_BUF: c_int = 5;
pub const _PC_CHOWN_RESTRICTED: c_int = 6;
pub const _PC_NO_TRUNC: c_int = 7;
pub const _PC_VDISABLE: c_int = 8;
pub const _PC_SYNC_IO: c_int = 9;
pub const _PC_ASYNC_IO: c_int = 10;
pub const _PC_PRIO_IO: c_int = 11;
pub const _PC_SOCK_MAXBUF: c_int = 12;
pub const _PC_FILESIZEBITS: c_int = 13;
pub const _PC_REC_INCR_XFER_SIZE: c_int = 14;
pub const _PC_REC_MAX_XFER_SIZE: c_int = 15;
pub const _PC_REC_MIN_XFER_SIZE: c_int = 16;
pub const _PC_REC_XFER_ALIGN: c_int = 17;
pub const _PC_ALLOC_SIZE_MIN: c_int = 18;
pub const _PC_SYMLINK_MAX: c_int = 19;
pub const _PC_2_SYMLINKS: c_int = 20;

pub const _SC_ARG_MAX: c_int = 0;
pub const _SC_CHILD_MAX: c_int = 1;
pub const _SC_CLK_TCK: c_int = 2;
pub const _SC_NGROUPS_MAX: c_int = 3;
pub const _SC_OPEN_MAX: c_int = 4;
pub const _SC_STREAM_MAX: c_int = 5;
pub const _SC_TZNAME_MAX: c_int = 6;
pub const _SC_JOB_CONTROL: c_int = 7;
pub const _SC_SAVED_IDS: c_int = 8;
pub const _SC_REALTIME_SIGNALS: c_int = 9;
pub const _SC_PRIORITY_SCHEDULING: c_int = 10;
pub const _SC_TIMERS: c_int = 11;
pub const _SC_ASYNCHRONOUS_IO: c_int = 12;
pub const _SC_PRIORITIZED_IO: c_int = 13;
pub const _SC_SYNCHRONIZED_IO: c_int = 14;
pub const _SC_FSYNC: c_int = 15;
pub const _SC_MAPPED_FILES: c_int = 16;
pub const _SC_MEMLOCK: c_int = 17;
pub const _SC_MEMLOCK_RANGE: c_int = 18;
pub const _SC_MEMORY_PROTECTION: c_int = 19;
pub const _SC_MESSAGE_PASSING: c_int = 20;
pub const _SC_SEMAPHORES: c_int = 21;
pub const _SC_SHARED_MEMORY_OBJECTS: c_int = 22;
pub const _SC_AIO_LISTIO_MAX: c_int = 23;
pub const _SC_AIO_MAX: c_int = 24;
pub const _SC_AIO_PRIO_DELTA_MAX: c_int = 25;
pub const _SC_DELAYTIMER_MAX: c_int = 26;
pub const _SC_MQ_OPEN_MAX: c_int = 27;
pub const _SC_MQ_PRIO_MAX: c_int = 28;
pub const _SC_VERSION: c_int = 29;
pub const _SC_PAGESIZE: c_int = 30;
pub const _SC_PAGE_SIZE: c_int = _SC_PAGESIZE;
pub const _SC_RTSIG_MAX: c_int = 31;
pub const _SC_SEM_NSEMS_MAX: c_int = 32;
pub const _SC_SEM_VALUE_MAX: c_int = 33;
pub const _SC_SIGQUEUE_MAX: c_int = 34;
pub const _SC_TIMER_MAX: c_int = 35;
pub const _SC_BC_BASE_MAX: c_int = 36;
pub const _SC_BC_DIM_MAX: c_int = 37;
pub const _SC_BC_SCALE_MAX: c_int = 38;
pub const _SC_BC_STRING_MAX: c_int = 39;
pub const _SC_COLL_WEIGHTS_MAX: c_int = 40;
pub const _SC_EXPR_NEST_MAX: c_int = 42;
pub const _SC_LINE_MAX: c_int = 43;
pub const _SC_RE_DUP_MAX: c_int = 44;
pub const _SC_2_VERSION: c_int = 46;
pub const _SC_2_C_BIND: c_int = 47;
pub const _SC_2_C_DEV: c_int = 48;
pub const _SC_2_FORT_DEV: c_int = 49;
pub const _SC_2_FORT_RUN: c_int = 50;
pub const _SC_2_SW_DEV: c_int = 51;
pub const _SC_2_LOCALEDEF: c_int = 52;
pub const _SC_UIO_MAXIOV: c_int = 60;
pub const _SC_IOV_MAX: c_int = 60;
pub const _SC_THREADS: c_int = 67;
pub const _SC_THREAD_SAFE_FUNCTIONS: c_int = 68;
pub const _SC_GETGR_R_SIZE_MAX: c_int = 69;
pub const _SC_GETPW_R_SIZE_MAX: c_int = 70;
pub const _SC_LOGIN_NAME_MAX: c_int = 71;
pub const _SC_TTY_NAME_MAX: c_int = 72;
pub const _SC_THREAD_DESTRUCTOR_ITERATIONS: c_int = 73;
pub const _SC_THREAD_KEYS_MAX: c_int = 74;
pub const _SC_THREAD_STACK_MIN: c_int = 75;
pub const _SC_THREAD_THREADS_MAX: c_int = 76;
pub const _SC_THREAD_ATTR_STACKADDR: c_int = 77;
pub const _SC_THREAD_ATTR_STACKSIZE: c_int = 78;
pub const _SC_THREAD_PRIORITY_SCHEDULING: c_int = 79;
pub const _SC_THREAD_PRIO_INHERIT: c_int = 80;
pub const _SC_THREAD_PRIO_PROTECT: c_int = 81;
pub const _SC_THREAD_PROCESS_SHARED: c_int = 82;
pub const _SC_NPROCESSORS_CONF: c_int = 83;
pub const _SC_NPROCESSORS_ONLN: c_int = 84;
pub const _SC_PHYS_PAGES: c_int = 85;
pub const _SC_AVPHYS_PAGES: c_int = 86;
pub const _SC_ATEXIT_MAX: c_int = 87;
pub const _SC_PASS_MAX: c_int = 88;
pub const _SC_XOPEN_VERSION: c_int = 89;
pub const _SC_XOPEN_XCU_VERSION: c_int = 90;
pub const _SC_XOPEN_UNIX: c_int = 91;
pub const _SC_XOPEN_CRYPT: c_int = 92;
pub const _SC_XOPEN_ENH_I18N: c_int = 93;
pub const _SC_XOPEN_SHM: c_int = 94;
pub const _SC_2_CHAR_TERM: c_int = 95;
pub const _SC_2_UPE: c_int = 97;
pub const _SC_XOPEN_XPG2: c_int = 98;
pub const _SC_XOPEN_XPG3: c_int = 99;
pub const _SC_XOPEN_XPG4: c_int = 100;
pub const _SC_NZERO: c_int = 109;
pub const _SC_XBS5_ILP32_OFF32: c_int = 125;
pub const _SC_XBS5_ILP32_OFFBIG: c_int = 126;
pub const _SC_XBS5_LP64_OFF64: c_int = 127;
pub const _SC_XBS5_LPBIG_OFFBIG: c_int = 128;
pub const _SC_XOPEN_LEGACY: c_int = 129;
pub const _SC_XOPEN_REALTIME: c_int = 130;
pub const _SC_XOPEN_REALTIME_THREADS: c_int = 131;
pub const _SC_ADVISORY_INFO: c_int = 132;
pub const _SC_BARRIERS: c_int = 133;
pub const _SC_CLOCK_SELECTION: c_int = 137;
pub const _SC_CPUTIME: c_int = 138;
pub const _SC_THREAD_CPUTIME: c_int = 139;
pub const _SC_MONOTONIC_CLOCK: c_int = 149;
pub const _SC_READER_WRITER_LOCKS: c_int = 153;
pub const _SC_SPIN_LOCKS: c_int = 154;
pub const _SC_REGEXP: c_int = 155;
pub const _SC_SHELL: c_int = 157;
pub const _SC_SPAWN: c_int = 159;
pub const _SC_SPORADIC_SERVER: c_int = 160;
pub const _SC_THREAD_SPORADIC_SERVER: c_int = 161;
pub const _SC_TIMEOUTS: c_int = 164;
pub const _SC_TYPED_MEMORY_OBJECTS: c_int = 165;
pub const _SC_2_PBS: c_int = 168;
pub const _SC_2_PBS_ACCOUNTING: c_int = 169;
pub const _SC_2_PBS_LOCATE: c_int = 170;
pub const _SC_2_PBS_MESSAGE: c_int = 171;
pub const _SC_2_PBS_TRACK: c_int = 172;
pub const _SC_SYMLOOP_MAX: c_int = 173;
pub const _SC_STREAMS: c_int = 174;
pub const _SC_2_PBS_CHECKPOINT: c_int = 175;
pub const _SC_V6_ILP32_OFF32: c_int = 176;
pub const _SC_V6_ILP32_OFFBIG: c_int = 177;
pub const _SC_V6_LP64_OFF64: c_int = 178;
pub const _SC_V6_LPBIG_OFFBIG: c_int = 179;
pub const _SC_HOST_NAME_MAX: c_int = 180;
pub const _SC_TRACE: c_int = 181;
pub const _SC_TRACE_EVENT_FILTER: c_int = 182;
pub const _SC_TRACE_INHERIT: c_int = 183;
pub const _SC_TRACE_LOG: c_int = 184;
pub const _SC_IPV6: c_int = 235;
pub const _SC_RAW_SOCKETS: c_int = 236;
pub const _SC_V7_ILP32_OFF32: c_int = 237;
pub const _SC_V7_ILP32_OFFBIG: c_int = 238;
pub const _SC_V7_LP64_OFF64: c_int = 239;
pub const _SC_V7_LPBIG_OFFBIG: c_int = 240;
pub const _SC_SS_REPL_MAX: c_int = 241;
pub const _SC_TRACE_EVENT_NAME_MAX: c_int = 242;
pub const _SC_TRACE_NAME_MAX: c_int = 243;
pub const _SC_TRACE_SYS_MAX: c_int = 244;
pub const _SC_TRACE_USER_EVENT_MAX: c_int = 245;
pub const _SC_XOPEN_STREAMS: c_int = 246;
pub const _SC_THREAD_ROBUST_PRIO_INHERIT: c_int = 247;
pub const _SC_THREAD_ROBUST_PRIO_PROTECT: c_int = 248;

pub const CPU_SETSIZE: c_int = 0x400;

pub const EPERM: int32_t = 1;
pub const ENOENT: int32_t = 2;
pub const ESRCH: int32_t = 3;
pub const EINTR: int32_t = 4;
pub const EIO: int32_t = 5;
pub const ENXIO: int32_t = 6;
pub const E2BIG: int32_t = 7;
pub const ENOEXEC: int32_t = 8;
pub const EBADF: int32_t = 9;
pub const ECHILD: int32_t = 10;
pub const EAGAIN: int32_t = 11;
pub const ENOMEM: int32_t = 12;
pub const EACCES: int32_t = 13;
pub const EFAULT: int32_t = 14;
pub const ENOTBLK: int32_t = 15;
pub const EBUSY: int32_t = 16;
pub const EEXIST: int32_t = 17;
pub const EXDEV: int32_t = 18;
pub const ENODEV: int32_t = 19;
pub const ENOTDIR: int32_t = 20;
pub const EISDIR: int32_t = 21;
pub const EINVAL: int32_t = 22;
pub const ENFILE: int32_t = 23;
pub const EMFILE: int32_t = 24;
pub const ENOTTY: int32_t = 25;
pub const ETXTBSY: int32_t = 26;
pub const EFBIG: int32_t = 27;
pub const ENOSPC: int32_t = 28;
pub const ESPIPE: int32_t = 29;
pub const EROFS: int32_t = 30;
pub const EMLINK: int32_t = 31;
pub const EPIPE: int32_t = 32;
pub const EDOM: int32_t = 33;
pub const ERANGE: int32_t = 34;
pub const EDEADLK: int32_t = 35;
pub const ENAMETOOLONG: int32_t = 36;
pub const ENOLCK: int32_t = 37;
pub const ENOSYS: int32_t = 38;
pub const ENOTEMPTY: int32_t = 39;
pub const ELOOP: int32_t = 40;
pub const EWOULDBLOCK: int32_t = EAGAIN;
pub const ENOMSG: int32_t = 42;
pub const EIDRM: int32_t = 43;
pub const ECHRNG: int32_t = 44;
pub const EL2NSYNC: int32_t = 45;
pub const EL3HLT: int32_t = 46;
pub const EL3RST: int32_t = 47;
pub const ELNRNG: int32_t = 48;
pub const EUNATCH: int32_t = 49;
pub const ENOCSI: int32_t = 50;
pub const EL2HLT: int32_t = 51;
pub const EBADE: int32_t = 52;
pub const EBADR: int32_t = 53;
pub const EXFULL: int32_t = 54;
pub const ENOANO: int32_t = 55;
pub const EBADRQC: int32_t = 56;
pub const EBADSLT: int32_t = 57;
pub const EDEADLOCK: int32_t = EDEADLK;
pub const EBFONT: int32_t = 59;
pub const ENOSTR: int32_t = 60;
pub const ENODATA: int32_t = 61;
pub const ETIME: int32_t = 62;
pub const ENOSR: int32_t = 63;
pub const ENONET: int32_t = 64;
pub const ENOPKG: int32_t = 65;
pub const EREMOTE: int32_t = 66;
pub const ENOLINK: int32_t = 67;
pub const EADV: int32_t = 68;
pub const ESRMNT: int32_t = 69;
pub const ECOMM: int32_t = 70;
pub const EPROTO: int32_t = 71;
pub const EMULTIHOP: int32_t = 72;
pub const EDOTDOT: int32_t = 73;
pub const EBADMSG: int32_t = 74;
pub const EOVERFLOW: int32_t = 75;
pub const ENOTUNIQ: int32_t = 76;
pub const EBADFD: int32_t = 77;
pub const EREMCHG: int32_t = 78;
pub const ELIBACC: int32_t = 79;
pub const ELIBBAD: int32_t = 80;
pub const ELIBSCN: int32_t = 81;
pub const ELIBMAX: int32_t = 82;
pub const ELIBEXEC: int32_t = 83;
pub const EILSEQ: int32_t = 84;
pub const ERESTART: int32_t = 85;
pub const ESTRPIPE: int32_t = 86;
pub const EUSERS: int32_t = 87;
pub const ENOTSOCK: int32_t = 88;
pub const EDESTADDRREQ: int32_t = 89;
pub const EMSGSIZE: int32_t = 90;
pub const EPROTOTYPE: int32_t = 91;
pub const ENOPROTOOPT: int32_t = 92;
pub const EPROTONOSUPPORT: int32_t = 93;
pub const ESOCKTNOSUPPORT: int32_t = 94;
pub const EOPNOTSUPP: int32_t = 95;
pub const EPFNOSUPPORT: int32_t = 96;
pub const EAFNOSUPPORT: int32_t = 97;
pub const EADDRINUSE: int32_t = 98;
pub const EADDRNOTAVAIL: int32_t = 99;
pub const ENETDOWN: int32_t = 100;
pub const ENETUNREACH: int32_t = 101;
pub const ENETRESET: int32_t = 102;
pub const ECONNABORTED: int32_t = 103;
pub const ECONNRESET: int32_t = 104;
pub const ENOBUFS: int32_t = 105;
pub const EISCONN: int32_t = 106;
pub const ENOTCONN: int32_t = 107;
pub const ESHUTDOWN: int32_t = 108;
pub const ETOOMANYREFS: int32_t = 109;
pub const ETIMEDOUT: int32_t = 110;
pub const ECONNREFUSED: int32_t = 111;
pub const EHOSTDOWN: int32_t = 112;
pub const EHOSTUNREACH: int32_t = 113;
pub const EALREADY: int32_t = 114;
pub const EINPROGRESS: int32_t = 115;
pub const ESTALE: int32_t = 116;
pub const EUCLEAN: int32_t = 117;
pub const ENOTNAM: int32_t = 118;
pub const ENAVAIL: int32_t = 119;
pub const EISNAM: int32_t = 120;
pub const EREMOTEIO: int32_t = 121;
pub const EDQUOT: int32_t = 122;
pub const ENOMEDIUM: int32_t = 123;
pub const EMEDIUMTYPE: int32_t = 124;
pub const ECANCELED: int32_t = 125;
pub const ENOKEY: int32_t = 126;
pub const EKEYEXPIRED: int32_t = 127;
pub const EKEYREVOKED: int32_t = 128;
pub const EKEYREJECTED: int32_t = 129;
pub const EOWNERDEAD: int32_t = 130;
pub const ENOTRECOVERABLE: int32_t = 131;
pub const ERFKILL: int32_t = 132;
pub const EHWPOISON: int32_t = 133;
pub const ENOTSUP: int32_t = EOPNOTSUPP;
pub const ESGX: int32_t = 0x0000_FFFF;

pub const SA_NODEFER: c_int = 0x40000000;
pub const SA_RESETHAND: c_int = 0x80000000;
pub const SA_RESTART: c_int = 0x10000000;
pub const SA_NOCLDSTOP: c_int = 0x00000001;

pub const SA_ONSTACK: c_int = 0x08000000;
pub const SA_SIGINFO: c_int = 0x00000004;
pub const SA_NOCLDWAIT: c_int = 0x00000002;

pub const SIG_DFL: sighandler_t = 0_usize;
pub const SIG_IGN: sighandler_t = 1_usize;
pub const SIG_ERR: sighandler_t = !0_usize;

pub const SIGTRAP: c_int = 5;
pub const SIGCHLD: c_int = 17;
pub const SIGBUS: c_int = 7;
pub const SIGTTIN: c_int = 21;
pub const SIGTTOU: c_int = 22;
pub const SIGXCPU: c_int = 24;
pub const SIGXFSZ: c_int = 25;
pub const SIGVTALRM: c_int = 26;
pub const SIGPROF: c_int = 27;
pub const SIGWINCH: c_int = 28;
pub const SIGUSR1: c_int = 10;
pub const SIGUSR2: c_int = 12;
pub const SIGCONT: c_int = 18;
pub const SIGSTOP: c_int = 19;
pub const SIGTSTP: c_int = 20;
pub const SIGURG: c_int = 23;
pub const SIGIO: c_int = 29;
pub const SIGSYS: c_int = 31;
pub const SIGSTKFLT: c_int = 16;
pub const SIGPOLL: c_int = 29;
pub const SIGPWR: c_int = 30;
pub const SIG_SETMASK: c_int = 2;
pub const SIG_BLOCK: c_int = 0x000000;
pub const SIG_UNBLOCK: c_int = 0x01;

pub const SIGRTMIN: c_int = 32;
pub const SIGRTMAX: c_int = 64;
pub const NSIG: c_int = SIGRTMAX + 1;

#[inline]
pub unsafe fn FD_CLR(fd: c_int, set: *mut fd_set) {
    let fd = fd as usize;
    let size = mem::size_of_val(&(*set).fds_bits[0]) * 8;
    (*set).fds_bits[fd / size] &= !(1 << (fd % size));
}

#[inline]
pub unsafe fn FD_ISSET(fd: c_int, set: *mut fd_set) -> bool {
    let fd = fd as usize;
    let size = mem::size_of_val(&(*set).fds_bits[0]) * 8;
    ((*set).fds_bits[fd / size] & (1 << (fd % size))) != 0
}

#[inline]
pub unsafe fn FD_SET(fd: c_int, set: *mut fd_set) {
    let fd = fd as usize;
    let size = mem::size_of_val(&(*set).fds_bits[0]) * 8;
    (*set).fds_bits[fd / size] |= 1 << (fd % size);
}

#[inline]
pub unsafe fn FD_ZERO(set: *mut fd_set) {
    for slot in (*set).fds_bits.iter_mut() {
        *slot = 0;
    }
}

#[inline]
pub unsafe fn CPU_ALLOC_SIZE(count: c_int) -> size_t {
    let _dummy: cpu_set_t = mem::zeroed();
    let size_in_bits = 8 * mem::size_of_val(&_dummy.bits[0]);
    ((count as size_t + size_in_bits - 1) / 8) as size_t
}

#[inline]
pub unsafe fn CPU_ZERO(cpuset: &mut cpu_set_t) {
    for slot in cpuset.bits.iter_mut() {
        *slot = 0;
    }
}

#[inline]
pub unsafe fn CPU_SET(cpu: usize, cpuset: &mut cpu_set_t) {
    let size_in_bits = 8 * mem::size_of_val(&cpuset.bits[0]); // 32, 64 etc
    let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
    cpuset.bits[idx] |= 1 << offset;
}

#[inline]
pub unsafe fn CPU_CLR(cpu: usize, cpuset: &mut cpu_set_t) {
    let size_in_bits = 8 * mem::size_of_val(&cpuset.bits[0]); // 32, 64 etc
    let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
    cpuset.bits[idx] &= !(1 << offset);
}

#[inline]
pub unsafe fn CPU_ISSET(cpu: usize, cpuset: &cpu_set_t) -> bool {
    let size_in_bits = 8 * mem::size_of_val(&cpuset.bits[0]);
    let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
    0 != (cpuset.bits[idx] & (1 << offset))
}

#[inline]
pub unsafe fn CPU_COUNT_S(size: usize, cpuset: &cpu_set_t) -> c_int {
    let mut s: u32 = 0;
    let size_of_mask = mem::size_of_val(&cpuset.bits[0]);
    for i in cpuset.bits[..(size / size_of_mask)].iter() {
        s += i.count_ones();
    }
    s as c_int
}

#[inline]
pub unsafe fn CPU_COUNT(cpuset: &cpu_set_t) -> c_int {
    CPU_COUNT_S(mem::size_of::<cpu_set_t>(), cpuset)
}

#[inline]
pub unsafe fn CPU_EQUAL(set1: &cpu_set_t, set2: &cpu_set_t) -> bool {
    set1.bits == set2.bits
}

#[inline]
unsafe fn __sigmask(sig: c_int) -> u64 {
    (1_u64) << (((sig) - 1) % (8 * mem::size_of::<u64>()) as i32)
}

#[inline]
unsafe fn __sigword(sig: c_int) -> u64 {
    ((sig - 1) / ((8 * mem::size_of::<u64>()) as i32)) as u64
}

#[inline]
unsafe fn __sigaddset(set: *mut sigset_t, sig: c_int) {
    let mask: u64 = __sigmask(sig);
    let word: u64 = __sigword(sig);
    (*set).__val[word as usize] |= mask;
}

#[inline]
unsafe fn __sigdelset(set: *mut sigset_t, sig: c_int) {
    let mask: u64 = __sigmask(sig);
    let word: u64 = __sigword(sig);
    (*set).__val[word as usize] &= !mask;
}

#[inline]
unsafe fn __sigismember(set: *const sigset_t, sig: c_int) -> c_int {
    let mask: u64 = __sigmask(sig);
    let word: u64 = __sigword(sig);
    let val = u64::from(mask != 0);
    ((*set).__val[word as usize] & val) as c_int
}

pub unsafe fn sigemptyset(set: *mut sigset_t) -> c_int {
    if set.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    ptr::write_bytes(set as *mut sigset_t, 0, 1);
    0
}

pub unsafe fn sigaddset(set: *mut sigset_t, signum: c_int) -> c_int {
    if set.is_null() || signum <= 0 || signum >= NSIG {
        set_errno(EINVAL);
        return -1;
    }
    __sigaddset(set, signum);
    0
}

pub unsafe fn sigfillset(set: *mut sigset_t) -> c_int {
    if set.is_null() {
        set_errno(EINVAL);
        return -1;
    }
    ptr::write_bytes(set, 0xff, 1);
    0
}

pub unsafe fn sigdelset(set: *mut sigset_t, signum: c_int) -> c_int {
    if set.is_null() || signum <= 0 || signum >= NSIG {
        set_errno(EINVAL);
        return -1;
    }
    __sigdelset(set, signum);
    0
}

pub unsafe fn sigismember(set: *const sigset_t, signum: c_int) -> c_int {
    if set.is_null() || signum <= 0 || signum >= NSIG {
        set_errno(EINVAL);
        return -1;
    }
    __sigismember(set, signum)
}

#[inline]
pub const fn CMSG_ALIGN(len: usize) -> usize {
    (len + mem::size_of::<usize>() - 1) & !(mem::size_of::<usize>() - 1)
}

#[inline]
pub unsafe fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
    if (*mhdr).msg_controllen >= mem::size_of::<cmsghdr>() {
        (*mhdr).msg_control as *mut cmsghdr
    } else {
        ptr::null_mut::<cmsghdr>()
    }
}

#[inline]
pub unsafe fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut c_uchar {
    cmsg.offset(1) as *mut c_uchar
}

#[inline]
pub const unsafe fn CMSG_SPACE(length: c_uint) -> c_uint {
    (CMSG_ALIGN(length as usize) + CMSG_ALIGN(mem::size_of::<cmsghdr>())) as c_uint
}

#[inline]
pub unsafe fn CMSG_LEN(length: c_uint) -> c_uint {
    CMSG_ALIGN(mem::size_of::<cmsghdr>()) as c_uint + length
}

#[inline]
pub unsafe fn CMSG_NXTHDR(mhdr: *const msghdr, cmsg: *const cmsghdr) -> *mut cmsghdr {
    if ((*cmsg).cmsg_len) < mem::size_of::<cmsghdr>() {
        return ptr::null_mut::<cmsghdr>();
    };
    let next = (cmsg as usize + CMSG_ALIGN((*cmsg).cmsg_len)) as *mut cmsghdr;
    let max = (*mhdr).msg_control as usize + (*mhdr).msg_controllen;
    if (next.offset(1)) as usize > max || next as usize + CMSG_ALIGN((*next).cmsg_len) > max {
        ptr::null_mut::<cmsghdr>()
    } else {
        next as *mut cmsghdr
    }
}

#[inline]
pub unsafe fn major(dev: dev_t) -> c_uint {
    let mut major = 0;
    major |= (dev & 0x00000000000fff00) >> 8;
    major |= (dev & 0xfffff00000000000) >> 32;
    major as c_uint
}

#[inline]
pub unsafe fn minor(dev: dev_t) -> c_uint {
    let mut minor = 0;
    minor |= dev & 0x00000000000000ff;
    minor |= (dev & 0x00000ffffff00000) >> 12;
    minor as c_uint
}

#[inline]
pub unsafe fn makedev(major: c_uint, minor: c_uint) -> dev_t {
    let major = major as dev_t;
    let minor = minor as dev_t;
    let mut dev = 0;
    dev |= (major & 0x00000fff) << 8;
    dev |= (major & 0xfffff000) << 32;
    dev |= minor & 0x000000ff;
    dev |= (minor & 0xffffff00) << 12;
    dev
}

pub mod ocall;
