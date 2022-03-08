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

use core::mem;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        pub mod x86_64;
        pub use x86_64::*;
    } else {

    }
}

pub mod edl;
pub mod ocall;

f! {
    pub fn FD_CLR(fd: c_int, set: *mut fd_set) -> () {
        let fd = fd as usize;
        let size = mem::size_of_val(&(*set).fds_bits[0]) * 8;
        (*set).fds_bits[fd / size] &= !(1 << (fd % size));
        ()
    }

    pub fn FD_ISSET(fd: c_int, set: *mut fd_set) -> bool {
        let fd = fd as usize;
        let size = mem::size_of_val(&(*set).fds_bits[0]) * 8;
        ((*set).fds_bits[fd / size] & (1 << (fd % size))) != 0
    }

    pub fn FD_SET(fd: c_int, set: *mut fd_set) -> () {
        let fd = fd as usize;
        let size = mem::size_of_val(&(*set).fds_bits[0]) * 8;
        (*set).fds_bits[fd / size] |= 1 << (fd % size);
        ()
    }

    pub fn FD_ZERO(set: *mut fd_set) -> () {
        for slot in (*set).fds_bits.iter_mut() {
            *slot = 0;
        }
    }

    pub fn CPU_ALLOC_SIZE(count: c_int) -> size_t {
        let _dummy: cpu_set_t = mem::zeroed();
        let size_in_bits = 8 * mem::size_of_val(&_dummy.bits[0]);
        ((count as size_t + size_in_bits - 1) / 8) as size_t
    }

    pub fn CPU_ZERO(cpuset: *mut cpu_set_t) -> () {
        for slot in (*cpuset).bits.iter_mut() {
            *slot = 0;
        }
    }

    pub fn CPU_SET(cpu: usize, cpuset: *mut cpu_set_t) -> () {
        let size_in_bits = 8 * mem::size_of_val(&(*cpuset).bits[0]); // 32, 64 etc
        let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
        (*cpuset).bits[idx] |= 1 << offset;
        ()
    }

    pub fn CPU_CLR(cpu: usize, cpuset: *mut cpu_set_t) -> () {
        let size_in_bits = 8 * mem::size_of_val(&(*cpuset).bits[0]); // 32, 64 etc
        let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
        (*cpuset).bits[idx] &= !(1 << offset);
        ()
    }

    pub fn CPU_ISSET(cpu: usize, cpuset: *const cpu_set_t) -> bool {
        let size_in_bits = 8 * mem::size_of_val(&(*cpuset).bits[0]);
        let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
        0 != ((*cpuset).bits[idx] & (1 << offset))
    }

    pub fn CPU_COUNT_S(size: usize, cpuset: *const cpu_set_t) -> c_int {
        let mut s: u32 = 0;
        let size_of_mask = mem::size_of_val(&(*cpuset).bits[0]);
        for i in (*cpuset).bits[..(size / size_of_mask)].iter() {
            s += i.count_ones();
        };
        s as c_int
    }

    pub fn CPU_COUNT(cpuset: *const cpu_set_t) -> c_int {
        CPU_COUNT_S(mem::size_of::<cpu_set_t>(), cpuset)
    }

    pub fn CPU_EQUAL(set1: *const cpu_set_t, set2: *const cpu_set_t) -> bool {
        (*set1).bits == (*set2).bits
    }

    pub fn CMSG_ALIGN(len: usize) -> usize {
        (len + mem::size_of::<usize>() - 1) & !(mem::size_of::<usize>() - 1)
    }

    pub fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
        if (*mhdr).msg_controllen as usize >= mem::size_of::<cmsghdr>() {
            (*mhdr).msg_control as *mut cmsghdr
        } else {
            core::ptr::null_mut::<cmsghdr>()
        }
    }

    pub fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut c_uchar {
        cmsg.offset(1) as *mut c_uchar
    }

    pub fn CMSG_SPACE(length: c_uint) -> c_uint {
        (CMSG_ALIGN(length as usize) + CMSG_ALIGN(mem::size_of::<cmsghdr>())) as c_uint
    }

    pub fn CMSG_LEN(length: c_uint) -> c_uint {
        CMSG_ALIGN(mem::size_of::<cmsghdr>()) as c_uint + length
    }

    pub fn CMSG_NXTHDR(mhdr: *const msghdr, cmsg: *const cmsghdr) -> *mut cmsghdr {
        if ((*cmsg).cmsg_len as usize) < mem::size_of::<cmsghdr>() {
            return core::ptr::null_mut::<cmsghdr>();
        };
        let next = (cmsg as usize + CMSG_ALIGN((*cmsg).cmsg_len as usize)) as *mut cmsghdr;
        let max = (*mhdr).msg_control as usize + (*mhdr).msg_controllen as usize;
        if (next.offset(1)) as usize > max
            || next as usize + CMSG_ALIGN((*next).cmsg_len as usize) > max
        {
            core::ptr::null_mut::<cmsghdr>()
        } else {
            next as *mut cmsghdr
        }
    }

    pub fn major(dev: dev_t) -> c_uint {
        let mut major = 0;
        major |= (dev & 0x00000000000fff00) >> 8;
        major |= (dev & 0xfffff00000000000) >> 32;
        major as c_uint
    }

    pub fn minor(dev: dev_t) -> c_uint {
        let mut minor = 0;
        minor |= dev & 0x00000000000000ff;
        minor |= (dev & 0x00000ffffff00000) >> 12;
        minor as c_uint
    }

    pub fn makedev(major: c_uint, minor: c_uint) -> dev_t {
        let major = major as dev_t;
        let minor = minor as dev_t;
        let mut dev = 0;
        dev |= (major & 0x00000fff) << 8;
        dev |= (major & 0xfffff000) << 32;
        dev |= minor & 0x000000ff;
        dev |= (minor & 0xffffff00) << 12;
        dev
    }

    pub fn __libc_current_sigrtmax() -> c_int {
        NSIG - 1
    }

    pub fn __libc_current_sigrtmin() -> c_int {
        32
    }
}

#[no_mangle]
pub unsafe extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    if set.is_null() {
        ocall::set_errno(EINVAL);
        return -1;
    };
    core::ptr::write_bytes(set as *mut sigset_t, 0, 1);
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaddset(set: *mut sigset_t, signum: c_int) -> c_int {
    if set.is_null() || signum <= 0 || signum >= NSIG {
        ocall::set_errno(EINVAL);
        return -1;
    }
    __sigaddset(set, signum);
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    if set.is_null() {
        ocall::set_errno(EINVAL);
        return -1;
    }
    core::ptr::write_bytes(set, 0xff, 1);
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigdelset(set: *mut sigset_t, signum: c_int) -> c_int {
    if set.is_null() || signum <= 0 || signum >= NSIG {
        ocall::set_errno(EINVAL);
        return -1;
    }
    __sigdelset(set, signum);
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigismember(set: *const sigset_t, signum: c_int) -> c_int {
    if set.is_null() || signum <= 0 || signum >= NSIG {
        ocall::set_errno(EINVAL);
        return -1;
    }
    __sigismember(set, signum)
}

#[inline]
fn __sigmask(sig: c_int) -> u64 {
    (1_u64) << (((sig) - 1) % (8 * mem::size_of::<u64>()) as i32)
}

#[inline]
fn __sigword(sig: c_int) -> u64 {
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
    let val = if mask != 0 { 1 } else { 0 };
    ((*set).__val[word as usize] & val) as c_int
}

safe_f! {
    pub fn SIGRTMAX() -> c_int {
        unsafe { __libc_current_sigrtmax() }
    }

    pub fn SIGRTMIN() -> c_int {
        unsafe { __libc_current_sigrtmin() }
    }

    pub {const} fn WIFSTOPPED(status: c_int) -> bool {
        (status & 0xff) == 0x7f
    }

    pub {const} fn WSTOPSIG(status: c_int) -> c_int {
        (status >> 8) & 0xff
    }

    pub {const} fn WIFCONTINUED(status: c_int) -> bool {
        status == 0xffff
    }

    pub {const} fn WIFSIGNALED(status: c_int) -> bool {
        ((status & 0x7f) + 1) as i8 >= 2
    }

    pub {const} fn WTERMSIG(status: c_int) -> c_int {
        status & 0x7f
    }

    pub {const} fn WIFEXITED(status: c_int) -> bool {
        (status & 0x7f) == 0
    }

    pub {const} fn WEXITSTATUS(status: c_int) -> c_int {
        (status >> 8) & 0xff
    }

    pub {const} fn WCOREDUMP(status: c_int) -> bool {
        (status & 0x80) != 0
    }

    pub {const} fn W_EXITCODE(ret: c_int, sig: c_int) -> c_int {
        (ret << 8) | sig
    }

    pub {const} fn W_STOPCODE(sig: c_int) -> c_int {
        (sig << 8) | 0x7f
    }

    pub {const} fn QCMD(cmd: c_int, type_: c_int) -> c_int {
        (cmd << 8) | (type_ & 0x00ff)
    }

    pub {const} fn IPOPT_COPIED(o: u8) -> u8 {
        o & IPOPT_COPY
    }

    pub {const} fn IPOPT_CLASS(o: u8) -> u8 {
        o & IPOPT_CLASS_MASK
    }

    pub {const} fn IPOPT_NUMBER(o: u8) -> u8 {
        o & IPOPT_NUMBER_MASK
    }

    pub {const} fn IPTOS_ECN(x: u8) -> u8 {
        x & IPTOS_ECN_MASK
    }
}
