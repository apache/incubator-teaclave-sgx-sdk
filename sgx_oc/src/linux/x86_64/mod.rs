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

pub use sgx_types::error::errno::*;
pub use sgx_types::types::time_t;
pub use sgx_types::types::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint,
    c_ulong, c_ulonglong, c_ushort, c_void, intmax_t, timespec, uintmax_t,
};
pub use sgx_types::types::{
    int16_t, int32_t, int64_t, int8_t, uint16_t, uint32_t, uint64_t, uint8_t,
};
pub use sgx_types::types::{intptr_t, ptrdiff_t, size_t, ssize_t, uintptr_t};

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
pub type nfds_t = u64;
pub type pid_t = i32;
pub type clock_t = i64;

pub type sighandler_t = size_t;

#[derive(Clone, Copy, Debug)]
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

    // pub struct timespec {
    //     pub tv_sec: time_t,
    //     pub tv_nsec: c_long,
    // }

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
        pub fds_bits: [c_ulong; FD_SETSIZE / ULONG_SIZE],
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
        pub bits: [u32; 32],
        #[cfg(not(all(target_pointer_width = "32",
                      not(target_arch = "x86_64"))))]
        pub bits: [u64; 16],
    }

    pub struct ucred {
        pub pid: pid_t,
        pub uid: uid_t,
        pub gid: gid_t,
    }

    pub struct sigset_t {
        pub __val: [u64; 16],
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

s_no_extra_traits! {
    #[cfg_attr(
        any(
            all(
                target_arch = "x86",
                not(target_env = "musl"),
                not(target_os = "android")),
            target_arch = "x86_64"),
        repr(packed))]
    pub struct epoll_event {
        pub events: u32,
        pub u64: u64,
    }

    pub struct sockaddr_un {
        pub sun_family: sa_family_t,
        pub sun_path: [c_char; 108]
    }

    pub struct sockaddr_storage {
        pub ss_family: sa_family_t,
        __ss_align: size_t,
        #[cfg(target_pointer_width = "32")]
        __ss_pad2: [u8; 128 - 2 * 4],
        #[cfg(target_pointer_width = "64")]
        __ss_pad2: [u8; 128 - 2 * 8],
    }

    pub struct utsname {
        pub sysname: [c_char; 65],
        pub nodename: [c_char; 65],
        pub release: [c_char; 65],
        pub version: [c_char; 65],
        pub machine: [c_char; 65],
        pub domainname: [c_char; 65]
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for epoll_event {
            fn eq(&self, other: &epoll_event) -> bool {
                self.events == other.events
                    && self.u64 == other.u64
            }
        }
        impl Eq for epoll_event {}
        impl ::fmt::Debug for epoll_event {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                let events = self.events;
                let u64 = self.u64;
                f.debug_struct("epoll_event")
                    .field("events", &events)
                    .field("u64", &u64)
                    .finish()
            }
        }
        impl ::hash::Hash for epoll_event {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                let events = self.events;
                let u64 = self.u64;
                events.hash(state);
                u64.hash(state);
            }
        }

        impl PartialEq for sockaddr_un {
            fn eq(&self, other: &sockaddr_un) -> bool {
                self.sun_family == other.sun_family
                    && self
                    .sun_path
                    .iter()
                    .zip(other.sun_path.iter())
                    .all(|(a, b)| a == b)
            }
        }
        impl Eq for sockaddr_un {}
        impl ::fmt::Debug for sockaddr_un {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("sockaddr_un")
                    .field("sun_family", &self.sun_family)
                // FIXME: .field("sun_path", &self.sun_path)
                    .finish()
            }
        }
        impl ::hash::Hash for sockaddr_un {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.sun_family.hash(state);
                self.sun_path.hash(state);
            }
        }

        impl PartialEq for sockaddr_storage {
            fn eq(&self, other: &sockaddr_storage) -> bool {
                self.ss_family == other.ss_family
                    && self
                    .__ss_pad2
                    .iter()
                    .zip(other.__ss_pad2.iter())
                    .all(|(a, b)| a == b)
            }
        }

        impl Eq for sockaddr_storage {}

        impl ::fmt::Debug for sockaddr_storage {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("sockaddr_storage")
                    .field("ss_family", &self.ss_family)
                    .field("__ss_align", &self.__ss_align)
                // FIXME: .field("__ss_pad2", &self.__ss_pad2)
                    .finish()
            }
        }

        impl ::hash::Hash for sockaddr_storage {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.ss_family.hash(state);
                self.__ss_pad2.hash(state);
            }
        }
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

pub const IPTOS_PREC_NETCONTROL: u8 = 0xe0;
pub const IPTOS_PREC_INTERNETCONTROL: u8 = 0xc0;
pub const IPTOS_PREC_CRITIC_ECP: u8 = 0xa0;
pub const IPTOS_PREC_FLASHOVERRIDE: u8 = 0x80;
pub const IPTOS_PREC_FLASH: u8 = 0x60;
pub const IPTOS_PREC_IMMEDIATE: u8 = 0x40;
pub const IPTOS_PREC_PRIORITY: u8 = 0x20;
pub const IPTOS_PREC_ROUTINE: u8 = 0x00;

pub const IPTOS_ECN_MASK: u8 = 0x03;
pub const IPTOS_ECN_ECT1: u8 = 0x01;
pub const IPTOS_ECN_ECT0: u8 = 0x02;
pub const IPTOS_ECN_CE: u8 = 0x03;

pub const IPOPT_COPY: u8 = 0x80;
pub const IPOPT_CLASS_MASK: u8 = 0x60;
pub const IPOPT_NUMBER_MASK: u8 = 0x1f;

pub const IPOPT_CONTROL: u8 = 0x00;
pub const IPOPT_RESERVED1: u8 = 0x20;
pub const IPOPT_MEASUREMENT: u8 = 0x40;
pub const IPOPT_RESERVED2: u8 = 0x60;
pub const IPOPT_END: u8 = IPOPT_CONTROL;
pub const IPOPT_NOOP: u8 = 1 | IPOPT_CONTROL;
pub const IPOPT_SEC: u8 = 2 | IPOPT_CONTROL | IPOPT_COPY;
pub const IPOPT_LSRR: u8 = 3 | IPOPT_CONTROL | IPOPT_COPY;
pub const IPOPT_TIMESTAMP: u8 = 4 | IPOPT_MEASUREMENT;
pub const IPOPT_RR: u8 = 7 | IPOPT_CONTROL;
pub const IPOPT_SID: u8 = 8 | IPOPT_CONTROL | IPOPT_COPY;
pub const IPOPT_SSRR: u8 = 9 | IPOPT_CONTROL | IPOPT_COPY;
pub const IPOPT_RA: u8 = 20 | IPOPT_CONTROL | IPOPT_COPY;
pub const IPVERSION: u8 = 4;
pub const MAXTTL: u8 = 255;
pub const IPDEFTTL: u8 = 64;
pub const IPOPT_OPTVAL: u8 = 0;
pub const IPOPT_OLEN: u8 = 1;
pub const IPOPT_OFFSET: u8 = 2;
pub const IPOPT_MINOFF: u8 = 4;
pub const MAX_IPOPTLEN: u8 = 40;
pub const IPOPT_NOP: u8 = IPOPT_NOOP;
pub const IPOPT_EOL: u8 = IPOPT_END;
pub const IPOPT_TS: u8 = IPOPT_TIMESTAMP;
pub const IPOPT_TS_TSONLY: u8 = 0;
pub const IPOPT_TS_TSANDADDR: u8 = 1;
pub const IPOPT_TS_PRESPEC: u8 = 3;

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
pub const EAI_SYSTEM: c_int = -11;
pub const EAI_OVERFLOW: c_int = -12;

pub const EAI_NODATA: c_int = -5;
pub const EAI_ADDRFAMILY: c_int = -9;
pub const EAI_INPROGRESS: c_int = -100;
pub const EAI_CANCELED: c_int = -101;
pub const EAI_NOTCANCELED: c_int = -102;
pub const EAI_ALLDONE: c_int = -103;
pub const EAI_INTR: c_int = -104;
pub const EAI_IDN_ENCODE: c_int = -105;

pub const NI_NUMERICHOST: c_int = 1;
pub const NI_NUMERICSERV: c_int = 2;
pub const NI_NOFQDN: c_int = 4;
pub const NI_NAMEREQD: c_int = 8;
pub const NI_DGRAM: c_int = 16;

pub const SYNC_FILE_RANGE_WAIT_BEFORE: c_uint = 1;
pub const SYNC_FILE_RANGE_WRITE: c_uint = 2;
pub const SYNC_FILE_RANGE_WAIT_AFTER: c_uint = 4;

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

pub const SA_NODEFER: c_int = 0x4000_0000;
pub const SA_RESETHAND: c_int = 0x8000_0000;
pub const SA_RESTART: c_int = 0x1000_0000;
pub const SA_NOCLDSTOP: c_int = 0x0000_0001;

pub const SA_ONSTACK: c_int = 0x0800_0000;
pub const SA_SIGINFO: c_int = 0x0000_0004;
pub const SA_NOCLDWAIT: c_int = 0x0000_0002;

pub const SIG_DFL: sighandler_t = 0_usize;
pub const SIG_IGN: sighandler_t = 1_usize;
pub const SIG_ERR: sighandler_t = !0;

pub const SIGHUP: c_int = 1;
pub const SIGINT: c_int = 2;
pub const SIGQUIT: c_int = 3;
pub const SIGILL: c_int = 4;
pub const SIGTRAP: c_int = 5;
pub const SIGABRT: c_int = 6;
pub const SIGBUS: c_int = 7;
pub const SIGFPE: c_int = 8;
pub const SIGKILL: c_int = 9;
pub const SIGUSR1: c_int = 10;
pub const SIGSEGV: c_int = 11;
pub const SIGUSR2: c_int = 12;
pub const SIGPIPE: c_int = 13;
pub const SIGALRM: c_int = 14;
pub const SIGTERM: c_int = 15;
pub const SIGSTKFLT: c_int = 16;
pub const SIGCHLD: c_int = 17;
pub const SIGCONT: c_int = 18;
pub const SIGSTOP: c_int = 19;
pub const SIGTSTP: c_int = 20;
pub const SIGTTIN: c_int = 21;
pub const SIGTTOU: c_int = 22;
pub const SIGURG: c_int = 23;
pub const SIGXCPU: c_int = 24;
pub const SIGXFSZ: c_int = 25;
pub const SIGVTALRM: c_int = 26;
pub const SIGPROF: c_int = 27;
pub const SIGWINCH: c_int = 28;
pub const SIGIO: c_int = 29;
pub const SIGPWR: c_int = 30;
pub const SIGSYS: c_int = 31;
pub const SIGPOLL: c_int = 29;
pub const SIG_BLOCK: c_int = 0x0000_0000;
pub const SIG_UNBLOCK: c_int = 0x01;

pub const SIGSTKSZ: size_t = 0x2000;
pub const MINSIGSTKSZ: size_t = 2048;
pub const SIG_SETMASK: c_int = 2;

pub const NSIG: c_int = 65;

pub const FUTEX_WAIT: c_int = 0;
pub const FUTEX_WAKE: c_int = 1;
pub const FUTEX_FD: c_int = 2;
pub const FUTEX_REQUEUE: c_int = 3;
pub const FUTEX_CMP_REQUEUE: c_int = 4;
pub const FUTEX_WAKE_OP: c_int = 5;
pub const FUTEX_LOCK_PI: c_int = 6;
pub const FUTEX_UNLOCK_PI: c_int = 7;
pub const FUTEX_TRYLOCK_PI: c_int = 8;
pub const FUTEX_WAIT_BITSET: c_int = 9;
pub const FUTEX_WAKE_BITSET: c_int = 10;
pub const FUTEX_WAIT_REQUEUE_PI: c_int = 11;
pub const FUTEX_CMP_REQUEUE_PI: c_int = 12;

pub const FUTEX_PRIVATE_FLAG: c_int = 128;
pub const FUTEX_CLOCK_REALTIME: c_int = 256;
pub const FUTEX_CMD_MASK: c_int = !(FUTEX_PRIVATE_FLAG | FUTEX_CLOCK_REALTIME);

// Syscall table

pub const SYS_read: c_long = 0;
pub const SYS_write: c_long = 1;
pub const SYS_open: c_long = 2;
pub const SYS_close: c_long = 3;
pub const SYS_stat: c_long = 4;
pub const SYS_fstat: c_long = 5;
pub const SYS_lstat: c_long = 6;
pub const SYS_poll: c_long = 7;
pub const SYS_lseek: c_long = 8;
pub const SYS_mmap: c_long = 9;
pub const SYS_mprotect: c_long = 10;
pub const SYS_munmap: c_long = 11;
pub const SYS_brk: c_long = 12;
pub const SYS_rt_sigaction: c_long = 13;
pub const SYS_rt_sigprocmask: c_long = 14;
pub const SYS_rt_sigreturn: c_long = 15;
pub const SYS_ioctl: c_long = 16;
pub const SYS_pread64: c_long = 17;
pub const SYS_pwrite64: c_long = 18;
pub const SYS_readv: c_long = 19;
pub const SYS_writev: c_long = 20;
pub const SYS_access: c_long = 21;
pub const SYS_pipe: c_long = 22;
pub const SYS_select: c_long = 23;
pub const SYS_sched_yield: c_long = 24;
pub const SYS_mremap: c_long = 25;
pub const SYS_msync: c_long = 26;
pub const SYS_mincore: c_long = 27;
pub const SYS_madvise: c_long = 28;
pub const SYS_shmget: c_long = 29;
pub const SYS_shmat: c_long = 30;
pub const SYS_shmctl: c_long = 31;
pub const SYS_dup: c_long = 32;
pub const SYS_dup2: c_long = 33;
pub const SYS_pause: c_long = 34;
pub const SYS_nanosleep: c_long = 35;
pub const SYS_getitimer: c_long = 36;
pub const SYS_alarm: c_long = 37;
pub const SYS_setitimer: c_long = 38;
pub const SYS_getpid: c_long = 39;
pub const SYS_sendfile: c_long = 40;
pub const SYS_socket: c_long = 41;
pub const SYS_connect: c_long = 42;
pub const SYS_accept: c_long = 43;
pub const SYS_sendto: c_long = 44;
pub const SYS_recvfrom: c_long = 45;
pub const SYS_sendmsg: c_long = 46;
pub const SYS_recvmsg: c_long = 47;
pub const SYS_shutdown: c_long = 48;
pub const SYS_bind: c_long = 49;
pub const SYS_listen: c_long = 50;
pub const SYS_getsockname: c_long = 51;
pub const SYS_getpeername: c_long = 52;
pub const SYS_socketpair: c_long = 53;
pub const SYS_setsockopt: c_long = 54;
pub const SYS_getsockopt: c_long = 55;
pub const SYS_clone: c_long = 56;
pub const SYS_fork: c_long = 57;
pub const SYS_vfork: c_long = 58;
pub const SYS_execve: c_long = 59;
pub const SYS_exit: c_long = 60;
pub const SYS_wait4: c_long = 61;
pub const SYS_kill: c_long = 62;
pub const SYS_uname: c_long = 63;
pub const SYS_semget: c_long = 64;
pub const SYS_semop: c_long = 65;
pub const SYS_semctl: c_long = 66;
pub const SYS_shmdt: c_long = 67;
pub const SYS_msgget: c_long = 68;
pub const SYS_msgsnd: c_long = 69;
pub const SYS_msgrcv: c_long = 70;
pub const SYS_msgctl: c_long = 71;
pub const SYS_fcntl: c_long = 72;
pub const SYS_flock: c_long = 73;
pub const SYS_fsync: c_long = 74;
pub const SYS_fdatasync: c_long = 75;
pub const SYS_truncate: c_long = 76;
pub const SYS_ftruncate: c_long = 77;
pub const SYS_getdents: c_long = 78;
pub const SYS_getcwd: c_long = 79;
pub const SYS_chdir: c_long = 80;
pub const SYS_fchdir: c_long = 81;
pub const SYS_rename: c_long = 82;
pub const SYS_mkdir: c_long = 83;
pub const SYS_rmdir: c_long = 84;
pub const SYS_creat: c_long = 85;
pub const SYS_link: c_long = 86;
pub const SYS_unlink: c_long = 87;
pub const SYS_symlink: c_long = 88;
pub const SYS_readlink: c_long = 89;
pub const SYS_chmod: c_long = 90;
pub const SYS_fchmod: c_long = 91;
pub const SYS_chown: c_long = 92;
pub const SYS_fchown: c_long = 93;
pub const SYS_lchown: c_long = 94;
pub const SYS_umask: c_long = 95;
pub const SYS_gettimeofday: c_long = 96;
pub const SYS_getrlimit: c_long = 97;
pub const SYS_getrusage: c_long = 98;
pub const SYS_sysinfo: c_long = 99;
pub const SYS_times: c_long = 100;
pub const SYS_ptrace: c_long = 101;
pub const SYS_getuid: c_long = 102;
pub const SYS_syslog: c_long = 103;
pub const SYS_getgid: c_long = 104;
pub const SYS_setuid: c_long = 105;
pub const SYS_setgid: c_long = 106;
pub const SYS_geteuid: c_long = 107;
pub const SYS_getegid: c_long = 108;
pub const SYS_setpgid: c_long = 109;
pub const SYS_getppid: c_long = 110;
pub const SYS_getpgrp: c_long = 111;
pub const SYS_setsid: c_long = 112;
pub const SYS_setreuid: c_long = 113;
pub const SYS_setregid: c_long = 114;
pub const SYS_getgroups: c_long = 115;
pub const SYS_setgroups: c_long = 116;
pub const SYS_setresuid: c_long = 117;
pub const SYS_getresuid: c_long = 118;
pub const SYS_setresgid: c_long = 119;
pub const SYS_getresgid: c_long = 120;
pub const SYS_getpgid: c_long = 121;
pub const SYS_setfsuid: c_long = 122;
pub const SYS_setfsgid: c_long = 123;
pub const SYS_getsid: c_long = 124;
pub const SYS_capget: c_long = 125;
pub const SYS_capset: c_long = 126;
pub const SYS_rt_sigpending: c_long = 127;
pub const SYS_rt_sigtimedwait: c_long = 128;
pub const SYS_rt_sigqueueinfo: c_long = 129;
pub const SYS_rt_sigsuspend: c_long = 130;
pub const SYS_sigaltstack: c_long = 131;
pub const SYS_utime: c_long = 132;
pub const SYS_mknod: c_long = 133;
pub const SYS_uselib: c_long = 134;
pub const SYS_personality: c_long = 135;
pub const SYS_ustat: c_long = 136;
pub const SYS_statfs: c_long = 137;
pub const SYS_fstatfs: c_long = 138;
pub const SYS_sysfs: c_long = 139;
pub const SYS_getpriority: c_long = 140;
pub const SYS_setpriority: c_long = 141;
pub const SYS_sched_setparam: c_long = 142;
pub const SYS_sched_getparam: c_long = 143;
pub const SYS_sched_setscheduler: c_long = 144;
pub const SYS_sched_getscheduler: c_long = 145;
pub const SYS_sched_get_priority_max: c_long = 146;
pub const SYS_sched_get_priority_min: c_long = 147;
pub const SYS_sched_rr_get_interval: c_long = 148;
pub const SYS_mlock: c_long = 149;
pub const SYS_munlock: c_long = 150;
pub const SYS_mlockall: c_long = 151;
pub const SYS_munlockall: c_long = 152;
pub const SYS_vhangup: c_long = 153;
pub const SYS_modify_ldt: c_long = 154;
pub const SYS_pivot_root: c_long = 155;
pub const SYS__sysctl: c_long = 156;
pub const SYS_prctl: c_long = 157;
pub const SYS_arch_prctl: c_long = 158;
pub const SYS_adjtimex: c_long = 159;
pub const SYS_setrlimit: c_long = 160;
pub const SYS_chroot: c_long = 161;
pub const SYS_sync: c_long = 162;
pub const SYS_acct: c_long = 163;
pub const SYS_settimeofday: c_long = 164;
pub const SYS_mount: c_long = 165;
pub const SYS_umount2: c_long = 166;
pub const SYS_swapon: c_long = 167;
pub const SYS_swapoff: c_long = 168;
pub const SYS_reboot: c_long = 169;
pub const SYS_sethostname: c_long = 170;
pub const SYS_setdomainname: c_long = 171;
pub const SYS_iopl: c_long = 172;
pub const SYS_ioperm: c_long = 173;
pub const SYS_create_module: c_long = 174;
pub const SYS_init_module: c_long = 175;
pub const SYS_delete_module: c_long = 176;
pub const SYS_get_kernel_syms: c_long = 177;
pub const SYS_query_module: c_long = 178;
pub const SYS_quotactl: c_long = 179;
pub const SYS_nfsservctl: c_long = 180;
pub const SYS_getpmsg: c_long = 181;
pub const SYS_putpmsg: c_long = 182;
pub const SYS_afs_syscall: c_long = 183;
pub const SYS_tuxcall: c_long = 184;
pub const SYS_security: c_long = 185;
pub const SYS_gettid: c_long = 186;
pub const SYS_readahead: c_long = 187;
pub const SYS_setxattr: c_long = 188;
pub const SYS_lsetxattr: c_long = 189;
pub const SYS_fsetxattr: c_long = 190;
pub const SYS_getxattr: c_long = 191;
pub const SYS_lgetxattr: c_long = 192;
pub const SYS_fgetxattr: c_long = 193;
pub const SYS_listxattr: c_long = 194;
pub const SYS_llistxattr: c_long = 195;
pub const SYS_flistxattr: c_long = 196;
pub const SYS_removexattr: c_long = 197;
pub const SYS_lremovexattr: c_long = 198;
pub const SYS_fremovexattr: c_long = 199;
pub const SYS_tkill: c_long = 200;
pub const SYS_time: c_long = 201;
pub const SYS_futex: c_long = 202;
pub const SYS_sched_setaffinity: c_long = 203;
pub const SYS_sched_getaffinity: c_long = 204;
pub const SYS_set_thread_area: c_long = 205;
pub const SYS_io_setup: c_long = 206;
pub const SYS_io_destroy: c_long = 207;
pub const SYS_io_getevents: c_long = 208;
pub const SYS_io_submit: c_long = 209;
pub const SYS_io_cancel: c_long = 210;
pub const SYS_get_thread_area: c_long = 211;
pub const SYS_lookup_dcookie: c_long = 212;
pub const SYS_epoll_create: c_long = 213;
pub const SYS_epoll_ctl_old: c_long = 214;
pub const SYS_epoll_wait_old: c_long = 215;
pub const SYS_remap_file_pages: c_long = 216;
pub const SYS_getdents64: c_long = 217;
pub const SYS_set_tid_address: c_long = 218;
pub const SYS_restart_syscall: c_long = 219;
pub const SYS_semtimedop: c_long = 220;
pub const SYS_fadvise64: c_long = 221;
pub const SYS_timer_create: c_long = 222;
pub const SYS_timer_settime: c_long = 223;
pub const SYS_timer_gettime: c_long = 224;
pub const SYS_timer_getoverrun: c_long = 225;
pub const SYS_timer_delete: c_long = 226;
pub const SYS_clock_settime: c_long = 227;
pub const SYS_clock_gettime: c_long = 228;
pub const SYS_clock_getres: c_long = 229;
pub const SYS_clock_nanosleep: c_long = 230;
pub const SYS_exit_group: c_long = 231;
pub const SYS_epoll_wait: c_long = 232;
pub const SYS_epoll_ctl: c_long = 233;
pub const SYS_tgkill: c_long = 234;
pub const SYS_utimes: c_long = 235;
pub const SYS_vserver: c_long = 236;
pub const SYS_mbind: c_long = 237;
pub const SYS_set_mempolicy: c_long = 238;
pub const SYS_get_mempolicy: c_long = 239;
pub const SYS_mq_open: c_long = 240;
pub const SYS_mq_unlink: c_long = 241;
pub const SYS_mq_timedsend: c_long = 242;
pub const SYS_mq_timedreceive: c_long = 243;
pub const SYS_mq_notify: c_long = 244;
pub const SYS_mq_getsetattr: c_long = 245;
pub const SYS_kexec_load: c_long = 246;
pub const SYS_waitid: c_long = 247;
pub const SYS_add_key: c_long = 248;
pub const SYS_request_key: c_long = 249;
pub const SYS_keyctl: c_long = 250;
pub const SYS_ioprio_set: c_long = 251;
pub const SYS_ioprio_get: c_long = 252;
pub const SYS_inotify_init: c_long = 253;
pub const SYS_inotify_add_watch: c_long = 254;
pub const SYS_inotify_rm_watch: c_long = 255;
pub const SYS_migrate_pages: c_long = 256;
pub const SYS_openat: c_long = 257;
pub const SYS_mkdirat: c_long = 258;
pub const SYS_mknodat: c_long = 259;
pub const SYS_fchownat: c_long = 260;
pub const SYS_futimesat: c_long = 261;
pub const SYS_newfstatat: c_long = 262;
pub const SYS_unlinkat: c_long = 263;
pub const SYS_renameat: c_long = 264;
pub const SYS_linkat: c_long = 265;
pub const SYS_symlinkat: c_long = 266;
pub const SYS_readlinkat: c_long = 267;
pub const SYS_fchmodat: c_long = 268;
pub const SYS_faccessat: c_long = 269;
pub const SYS_pselect6: c_long = 270;
pub const SYS_ppoll: c_long = 271;
pub const SYS_unshare: c_long = 272;
pub const SYS_set_robust_list: c_long = 273;
pub const SYS_get_robust_list: c_long = 274;
pub const SYS_splice: c_long = 275;
pub const SYS_tee: c_long = 276;
pub const SYS_sync_file_range: c_long = 277;
pub const SYS_vmsplice: c_long = 278;
pub const SYS_move_pages: c_long = 279;
pub const SYS_utimensat: c_long = 280;
pub const SYS_epoll_pwait: c_long = 281;
pub const SYS_signalfd: c_long = 282;
pub const SYS_timerfd_create: c_long = 283;
pub const SYS_eventfd: c_long = 284;
pub const SYS_fallocate: c_long = 285;
pub const SYS_timerfd_settime: c_long = 286;
pub const SYS_timerfd_gettime: c_long = 287;
pub const SYS_accept4: c_long = 288;
pub const SYS_signalfd4: c_long = 289;
pub const SYS_eventfd2: c_long = 290;
pub const SYS_epoll_create1: c_long = 291;
pub const SYS_dup3: c_long = 292;
pub const SYS_pipe2: c_long = 293;
pub const SYS_inotify_init1: c_long = 294;
pub const SYS_preadv: c_long = 295;
pub const SYS_pwritev: c_long = 296;
pub const SYS_rt_tgsigqueueinfo: c_long = 297;
pub const SYS_perf_event_open: c_long = 298;
pub const SYS_recvmmsg: c_long = 299;
pub const SYS_fanotify_init: c_long = 300;
pub const SYS_fanotify_mark: c_long = 301;
pub const SYS_prlimit64: c_long = 302;
pub const SYS_name_to_handle_at: c_long = 303;
pub const SYS_open_by_handle_at: c_long = 304;
pub const SYS_clock_adjtime: c_long = 305;
pub const SYS_syncfs: c_long = 306;
pub const SYS_sendmmsg: c_long = 307;
pub const SYS_setns: c_long = 308;
pub const SYS_getcpu: c_long = 309;
pub const SYS_process_vm_readv: c_long = 310;
pub const SYS_process_vm_writev: c_long = 311;
pub const SYS_kcmp: c_long = 312;
pub const SYS_finit_module: c_long = 313;
pub const SYS_sched_setattr: c_long = 314;
pub const SYS_sched_getattr: c_long = 315;
pub const SYS_renameat2: c_long = 316;
pub const SYS_seccomp: c_long = 317;
pub const SYS_getrandom: c_long = 318;
pub const SYS_memfd_create: c_long = 319;
pub const SYS_kexec_file_load: c_long = 320;
pub const SYS_bpf: c_long = 321;
pub const SYS_execveat: c_long = 322;
pub const SYS_userfaultfd: c_long = 323;
pub const SYS_membarrier: c_long = 324;
pub const SYS_mlock2: c_long = 325;
pub const SYS_copy_file_range: c_long = 326;
pub const SYS_preadv2: c_long = 327;
pub const SYS_pwritev2: c_long = 328;
pub const SYS_pkey_mprotect: c_long = 329;
pub const SYS_pkey_alloc: c_long = 330;
pub const SYS_pkey_free: c_long = 331;
pub const SYS_statx: c_long = 332;
pub const SYS_pidfd_send_signal: c_long = 424;
pub const SYS_io_uring_setup: c_long = 425;
pub const SYS_io_uring_enter: c_long = 426;
pub const SYS_io_uring_register: c_long = 427;
pub const SYS_open_tree: c_long = 428;
pub const SYS_move_mount: c_long = 429;
pub const SYS_fsopen: c_long = 430;
pub const SYS_fsconfig: c_long = 431;
pub const SYS_fsmount: c_long = 432;
pub const SYS_fspick: c_long = 433;
pub const SYS_pidfd_open: c_long = 434;
pub const SYS_clone3: c_long = 435;
pub const SYS_close_range: c_long = 436;
pub const SYS_openat2: c_long = 437;
pub const SYS_pidfd_getfd: c_long = 438;
pub const SYS_faccessat2: c_long = 439;
pub const SYS_process_madvise: c_long = 440;
pub const SYS_epoll_pwait2: c_long = 441;
pub const SYS_mount_setattr: c_long = 442;
