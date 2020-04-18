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

use std::io::Error;
use std::collections::HashMap;
use std::sync::{Mutex, Once};
use std::mem;
use libc::{self, c_int, c_void, sigset_t, siginfo_t};
use libc::{signal, sigprocmask, sigaction, sigemptyset, sigfillset, raise};
use libc::{SIG_SETMASK, SA_SIGINFO, SIG_ERR, SIG_DFL};
use crate::sgx_types::{sgx_enclave_id_t, sgx_status_t};

static DISPATCHER_INIT: Once = Once::new();
static mut GLOBAL_DATA: Option<GlobalData> = None;

pub const SIGRTMIN: c_int = 32;
pub const SIGRTMAX: c_int = 64;
pub const NSIG: c_int = SIGRTMAX + 1;

extern "C" {
    fn t_signal_handler_ecall(eid: sgx_enclave_id_t,
                              retval: *mut c_int,
                              info: *const siginfo_t) -> sgx_status_t;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SigNum(i32);

impl SigNum {
    // add code here
    pub fn from_raw(signo: i32) -> Option<SigNum> {
        if signo <= 0 || signo >= NSIG {
            None
        } else {
            Some(SigNum(signo))
        }
    }
    pub unsafe fn from_raw_uncheck(signo: i32) -> SigNum {
        SigNum(signo)
    }
    pub fn raw(&self) -> i32 { self.0 }
}

#[derive(Copy, Clone)]
pub struct SigSet(sigset_t);

impl SigSet {
    // add code here
    pub fn new() -> SigSet {
        let set = unsafe {
            let mut set: sigset_t = mem::zeroed();
            sigemptyset(&mut set as *mut sigset_t);
            set
        };
        SigSet(set)
    }
    pub fn fill(&mut self) {
        unsafe {
            sigfillset(&mut self.0 as *mut sigset_t);
        }
    }
    pub unsafe fn from_raw(set: sigset_t) -> SigSet { SigSet(set) }
    pub fn raw(&self) -> sigset_t { self.0 }
}

struct GlobalData {
    signal_dispatcher: SignalDispatcher,
}

impl GlobalData {
     // add code here
    fn get() -> &'static GlobalData {
        unsafe { GLOBAL_DATA.as_ref().unwrap() }
    }
    fn ensure() -> &'static GlobalData {
        DISPATCHER_INIT.call_once(|| unsafe {
            GLOBAL_DATA = Some(GlobalData {
                signal_dispatcher: SignalDispatcher::new(),
            });
        });
        Self::get()
    }
}

struct SignalDispatcher {
    signal_set: Mutex<HashMap<SigNum, sgx_enclave_id_t>>,
}

impl SignalDispatcher {
    // add code here
    pub fn new () -> SignalDispatcher {
        SignalDispatcher {
            signal_set: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_signal(&self, signo: SigNum, enclave_id: sgx_enclave_id_t) -> Option<sgx_enclave_id_t> {
         // Block all signals when registering a signal handler to avoid deadlock.
        let mut mask = SigSet::new();
        let oldmask = SigSet::new();
        mask.fill();

        unsafe { sigprocmask(SIG_SETMASK, &mask.raw(), &mut oldmask.raw() as *mut sigset_t); }
        let old = self.signal_set
            .lock()
            .unwrap()
            .insert(signo, enclave_id);
        unsafe { sigprocmask(SIG_SETMASK, &oldmask.raw(), 0 as *mut sigset_t); }
        old
    }

    pub fn get_eid_for_signal(&self, signo: SigNum) -> Option<sgx_enclave_id_t> {
        self.signal_set
            .lock()
            .unwrap()
            .get(&signo)
            .copied()
    }

    pub fn deregister_all_signals_for_eid(&self, eid: sgx_enclave_id_t) {
        let mut mask = SigSet::new();
        let oldmask = SigSet::new();
        mask.fill();

        unsafe { sigprocmask(SIG_SETMASK, &mask.raw(), &mut oldmask.raw() as *mut sigset_t); }
        // If this enclave has registered any signals, deregister them and set the
        // signal handler to the default one.
        self.signal_set
            .lock()
            .unwrap()
            .retain(|&signum, &mut v| {
                if v == eid {
                    unsafe {
                        if signal(signum.raw(), SIG_DFL) == SIG_ERR {
                        }
                    }
                }
                v != eid
            });
        unsafe { sigprocmask(SIG_SETMASK, &oldmask.raw(), 0 as *mut sigset_t); }
    }

    unsafe fn handle_signal(
        &self,
        signo: SigNum,
        info: &siginfo_t,
        _context: *const c_void,
    ) -> c_int {
        let mut retval: c_int = 0;
        let eid = match self.get_eid_for_signal(signo) {
            Some(eid) => eid,
            None => return -1,
        };

        let result = t_signal_handler_ecall(
            eid,
            &mut retval as *mut c_int,
            info as *const siginfo_t,
        );
        if result != sgx_status_t::SGX_SUCCESS {
            return -1;
        }
        retval
    }
}

pub fn deregister_all_signals_for_eid(enclave_id: sgx_enclave_id_t) {
    let global = GlobalData::ensure();
    global.signal_dispatcher.deregister_all_signals_for_eid(enclave_id);
}

extern "C" fn handle_signal_entry(signum: c_int, info: *const siginfo_t, ucontext: *const c_void) {
    let signo = SigNum::from_raw(signum);
    if info.is_null() || signo.is_none() {
        return;
    }

    unsafe {
        GlobalData::get().signal_dispatcher
            .handle_signal(signo.unwrap(), &(*info), ucontext);
    }
}

#[no_mangle]
pub extern "C" fn u_sigaction_ocall(
    error: * mut c_int,
    signum: c_int,
    act: *const sigaction,
    oldact: *mut sigaction,
    enclave_id: sgx_enclave_id_t,
) -> c_int {
    let mut errno = 0;
    let signo = SigNum::from_raw(signum);
    if signo.is_none() || act.is_null() {
        if  !error.is_null() {
            unsafe {  *error = libc::EINVAL; }
        }
        return -1;
    }

    let e_act = unsafe { &*act };
    let ret = if e_act.sa_sigaction == 0 {
        let global = GlobalData::ensure();
        global.signal_dispatcher.register_signal(signo.unwrap(), enclave_id);

        type FnSaSigaction = extern "C" fn(c_int, *const siginfo_t, *const c_void);
        let new_act = sigaction {
            sa_sigaction: unsafe { mem::transmute::<FnSaSigaction, usize>(handle_signal_entry) },
            // Set the flag so that sa_sigaction is registered as the signal handler
            // instead of sa_handler.
            sa_flags: e_act.sa_flags | SA_SIGINFO,
            sa_mask: e_act.sa_mask,
            sa_restorer: None,
        };
        let mut old_act: sigaction = unsafe { mem::zeroed() };
        unsafe { sigaction(signum, &new_act, &mut old_act as *mut sigaction) }
    } else {
       unsafe { sigaction(signum, act as *const sigaction, oldact) }
    };

    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_sigprocmask_ocall(
    error: *mut c_int,
    signum: c_int,
    set: *const sigset_t,
    oldset: *mut sigset_t,
) -> c_int {
    let mut errno = 0;
    let ret = unsafe { sigprocmask(signum, set, oldset) };
    if ret < 0 {
        errno = Error::last_os_error().raw_os_error().unwrap_or(0);
    }
    if !error.is_null() {
        unsafe { *error = errno; }
    }
    ret
}

#[no_mangle]
pub extern "C" fn u_raise_ocall(signum: c_int) -> c_int {
    unsafe { raise(signum) }
}

#[no_mangle]
pub extern "C" fn u_signal_clear_ocall(eid: sgx_enclave_id_t) {
    deregister_all_signals_for_eid(eid);
}
