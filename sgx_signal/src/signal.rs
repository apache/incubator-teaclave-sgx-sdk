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

use crate::manager::{self, ActionId, SigNum, SigSet, SignalManager};
use sgx_libc::ocall::{raise, sigaction, sigprocmask};
use sgx_libc::set_errno;
use sgx_libc::{sigaction, sigemptyset, sighandler_t, siginfo_t, sigset_t};
use sgx_libc::{EINVAL, ESGX};
use sgx_libc::{
    SA_RESETHAND, SIGBUS, SIGFPE, SIGILL, SIGKILL, SIGSEGV, SIGSTOP, SIGTRAP, SIG_BLOCK, SIG_DFL,
    SIG_ERR, SIG_SETMASK, SIG_UNBLOCK,
};
use sgx_types::{c_int, c_void, sgx_enclave_id_t, sgx_status_t, SysResult};
use std::enclave::get_enclave_id;
use std::io::Error;
use std::mem;
use std::ptr;
use std::rt::*;
use std::sync::{Arc, Once, SgxMutex};

pub const FORBIDDEN: &[c_int] = FORBIDDEN_IMPL;
const FORBIDDEN_IMPL: &[c_int] = &[SIGKILL, SIGSTOP, SIGILL, SIGFPE, SIGSEGV, SIGBUS, SIGTRAP];

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SignalId {
    signal: SigNum,
    action: ActionId,
}

struct GlobalData {
    signal_manager: SignalManager,
    signal_action_lock: SgxMutex<()>,
}

static mut GLOBAL_DATA: Option<GlobalData> = None;
static MANAGER_INIT: Once = Once::new();

extern "C" {
    pub fn u_signal_clear_ocall(enclave_id: sgx_enclave_id_t) -> sgx_status_t;
}

impl GlobalData {
    fn get() -> &'static Self {
        unsafe { GLOBAL_DATA.as_ref().unwrap() }
    }

    fn ensure() -> &'static Self {
        MANAGER_INIT.call_once(|| unsafe {
            GLOBAL_DATA = Some(GlobalData {
                signal_manager: SignalManager::new(),
                signal_action_lock: SgxMutex::new(()),
            });

            let _r = at_exit(Self::clear);
        });
        Self::get()
    }

    fn clear() {
        if !Self::get().signal_manager.is_action_empty() {
            unsafe { u_signal_clear_ocall(get_enclave_id()) };
        }
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn t_signal_handler_ecall(info: *const siginfo_t) -> c_int {
    if info.is_null() {
        return -1;
    }

    let si_info = &*(info);
    let global = GlobalData::get();
    let mask = manager::get_block_mask();
    // If the signal is blocked and still passed into the enclave. The signal
    // masks inside the enclave is out of sync with the untrusted signal mask.
    let signo = SigNum::from_raw_uncheck(si_info.si_signo);
    if mask.is_member(signo) {
        -1
    } else {
        global.signal_manager.handler(
            si_info.si_signo,
            info as *const siginfo_t,
            ptr::null::<c_void>(),
        );
        0
    }
}

fn native_sigaction(signo: SigNum, act: &sigaction, oldact: &mut sigaction) -> c_int {
    let global = GlobalData::ensure();
    let mut mask = SigSet::new();
    let old_mask = SigSet::new();

    mask.fill();

    // Guards sigaction calls. This is to ensure that signal handlers are not
    // overwritten between the time sigaction gets |oldact| and sets |act|.
    {
        let _guard = global.signal_action_lock.lock();
        if let Some(t) = global.signal_manager.get_action(signo) {
            *oldact = t.get_act();
        } else {
            oldact.sa_sigaction = SIG_DFL;
        }

        rsgx_sigprocmask(SIG_SETMASK, &mask.raw(), &mut old_mask.raw());
        global.signal_manager.set_action(signo, act);
        rsgx_sigprocmask(SIG_SETMASK, &old_mask.raw(), &mut mask.raw());
    }

    let new_act = sigaction {
        sa_sigaction: 0,
        sa_mask: act.sa_mask,
        sa_flags: act.sa_flags,
        sa_restorer: None,
    };
    if (new_act.sa_flags & SA_RESETHAND) != 0 {
        global.signal_manager.set_reset_on_handle(signo);
    }

    unsafe {
        sigaction(
            signo.raw(),
            &new_act,
            oldact as *mut sigaction,
            get_enclave_id(),
        )
    }
}

fn native_sigaction_impl<F>(
    signo: SigNum,
    act: &sigaction,
    oldact: &mut sigaction,
    f: Arc<F>,
) -> SysResult<ActionId>
where
    F: Fn(&siginfo_t) + Sync + Send + 'static,
{
    let global = GlobalData::ensure();
    let mut mask = SigSet::new();
    let old_mask = SigSet::new();

    mask.fill();

    // Guards sigaction calls. This is to ensure that signal handlers are not
    // overwritten between the time sigaction gets |oldact| and sets |act|.
    let (exist, action_id) = {
        let _guard = global.signal_action_lock.lock();
        let exist = if let Some(t) = global.signal_manager.get_action(signo) {
            *oldact = t.get_act();
            true
        } else {
            oldact.sa_sigaction = SIG_DFL;
            false
        };

        rsgx_sigprocmask(SIG_SETMASK, &mask.raw(), &mut old_mask.raw());
        let action_id = global.signal_manager.set_action_impl(signo, act, f);
        rsgx_sigprocmask(SIG_SETMASK, &old_mask.raw(), &mut mask.raw());
        (exist, action_id)
    };

    if exist {
        return Ok(action_id);
    }

    let new_act = sigaction {
        sa_sigaction: 0,
        sa_mask: act.sa_mask,
        sa_flags: act.sa_flags,
        sa_restorer: None,
    };
    if (new_act.sa_flags & SA_RESETHAND) != 0 {
        global.signal_manager.set_reset_on_handle(signo);
    }

    let result = unsafe {
        sigaction(
            signo.raw(),
            &new_act,
            oldact as *mut sigaction,
            get_enclave_id(),
        )
    };
    if result == 0 {
        Ok(action_id)
    } else {
        Err(result)
    }
}

pub fn rsgx_sigaction(signum: c_int, act: &sigaction, oldact: &mut sigaction) -> c_int {
    if FORBIDDEN.contains(&signum) {
        set_errno(EINVAL);
        return -1;
    }

    let eid = get_enclave_id();
    if eid == 0 {
        set_errno(ESGX);
        return -1;
    }

    let signo = match SigNum::from_raw(signum) {
        Some(signo) => signo,
        None => {
            set_errno(EINVAL);
            return -1;
        }
    };
    native_sigaction(signo, act, oldact)
}

pub fn rsgx_signal(signum: c_int, handler: sighandler_t) -> sighandler_t {
    let mut act: sigaction = unsafe { mem::zeroed() };
    let mut oldact: sigaction = unsafe { mem::zeroed() };
    act.sa_sigaction = handler;
    unsafe { sigemptyset(&mut act.sa_mask as *mut sigset_t) };
    if rsgx_sigaction(signum, &act, &mut oldact) != 0 {
        // Errno is set by sigaction.
        return SIG_ERR;
    }
    oldact.sa_sigaction
}

pub fn rsgx_sigprocmask(how: c_int, set: &sigset_t, oldset: &mut sigset_t) -> c_int {
    if how != SIG_BLOCK && how != SIG_UNBLOCK && how != SIG_SETMASK {
        set_errno(EINVAL);
        return -1;
    }

    let mut signals_to_block = SigSet::new();
    let mut signals_to_unblock = SigSet::new();

    *oldset = manager::get_block_mask().raw();
    let newset = unsafe { SigSet::from_raw(*set) };

    if how == SIG_BLOCK || how == SIG_SETMASK {
        signals_to_block = newset;
    }
    if how == SIG_UNBLOCK {
        signals_to_unblock = newset;
    } else if how == SIG_SETMASK {
        signals_to_unblock = newset.complement();
    }
    // Unblock signals inside the enclave before unblocking signals on the host.
    // |oldset| is already filled with the signal mask inside the enclave.
    manager::unblock(&signals_to_unblock);
    let result = unsafe { sigprocmask(how, set as *const sigset_t, ptr::null_mut::<sigset_t>()) };
    // Block signals inside the enclave after the host.
    manager::block(&signals_to_block);
    result
}

pub fn rsgx_raise(signum: c_int) -> c_int {
    unsafe { raise(signum) }
}

pub fn register<F>(signal: c_int, action: F) -> Result<SignalId, Error>
where
    F: Fn() + Sync + Send + 'static,
{
    register_sigaction_impl(signal, move |_: &_| action())
}

pub fn register_sigaction<F>(signal: c_int, action: F) -> Result<SignalId, Error>
where
    F: Fn(&siginfo_t) + Sync + Send + 'static,
{
    register_sigaction_impl(signal, action)
}

fn register_sigaction_impl<F>(signal: c_int, action: F) -> Result<SignalId, Error>
where
    F: Fn(&siginfo_t) + Sync + Send + 'static,
{
    register_impl(signal, action)
}

fn register_impl<F>(signal: c_int, action: F) -> Result<SignalId, Error>
where
    F: Fn(&siginfo_t) + Sync + Send + 'static,
{
    if FORBIDDEN.contains(&signal) {
        set_errno(EINVAL);
        return Err(Error::from_raw_os_error(EINVAL));
    }

    let eid = get_enclave_id();
    if eid == 0 {
        set_errno(ESGX);
        return Err(Error::from_sgx_error(
            sgx_status_t::SGX_ERROR_INVALID_ENCLAVE_ID,
        ));
    }

    let signo = match SigNum::from_raw(signal) {
        Some(signo) => signo,
        None => {
            set_errno(EINVAL);
            return Err(Error::from_raw_os_error(EINVAL));
        }
    };

    let act: sigaction = unsafe { mem::zeroed() };
    let mut oldact: sigaction = unsafe { mem::zeroed() };

    native_sigaction_impl(signo, &act, &mut oldact, Arc::from(action))
        .map(|action_id| SignalId {
            signal: signo,
            action: action_id,
        })
        .map_err(Error::from_raw_os_error)
}

pub fn unregister(id: SignalId) -> bool {
    let globals = GlobalData::ensure();
    globals.signal_manager.remove_action(id.signal, id.action)
}

pub fn unregister_signal(signal: c_int) -> bool {
    let globals = GlobalData::ensure();
    let signo = unsafe { SigNum::from_raw_uncheck(signal) };
    globals.signal_manager.clear_action(signo)
}

pub fn raise_signal(signal: c_int) -> bool {
    rsgx_raise(signal) == 0
}
