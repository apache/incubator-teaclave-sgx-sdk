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
use sgx_libc::{c_int, c_void};
use sgx_libc::{
    sigaction, sigaddset, sigdelset, sigemptyset, sigfillset, siginfo_t, sigismember, sigset_t,
};
use sgx_libc::{NSIG, SA_SIGINFO, SIGRTMAX};
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem;
use std::num::NonZeroU64;
use std::sync::Arc;
#[allow(deprecated)]
use std::sync::{SgxMutex, PoisonError};
use std::u64;

thread_local! { static SIGNAL_MASK: Cell<SigSet> = Cell::new(SigSet::new()) }

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SigNum(i32);

impl SigNum {
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

    pub fn raw(&self) -> i32 {
        self.0
    }
}
#[derive(Copy, Clone)]
pub struct SigSet(sigset_t);

impl SigSet {
    pub fn new() -> SigSet {
        let set = unsafe {
            let mut set: sigset_t = mem::zeroed();
            sigemptyset(&mut set as *mut sigset_t);
            set
        };
        SigSet(set)
    }

    #[allow(dead_code)]
    pub fn empty(&mut self) {
        unsafe {
            sigemptyset(&mut self.0 as *mut sigset_t);
        }
    }

    pub fn add(&mut self, signo: SigNum) -> bool {
        unsafe { sigaddset(&mut self.0 as *mut sigset_t, signo.raw()) == 0 }
    }

    pub fn delete(&mut self, signo: SigNum) -> bool {
        unsafe { sigdelset(&mut self.0 as *mut sigset_t, signo.raw()) == 0 }
    }

    pub fn fill(&mut self) {
        unsafe {
            sigfillset(&mut self.0 as *mut sigset_t);
        }
    }

    pub fn is_member(&self, signo: SigNum) -> bool {
        unsafe { sigismember(&self.0 as *const sigset_t, signo.raw()) == 1 }
    }

    pub fn complement(&self) -> SigSet {
        let mut set = SigSet::new();
        for num in 0..=SIGRTMAX {
            let signo = unsafe { SigNum::from_raw_uncheck(num) };
            if !self.is_member(signo) {
                set.add(signo);
            }
        }
        set
    }

    pub unsafe fn from_raw(set: sigset_t) -> SigSet {
        SigSet(set)
    }

    pub fn raw(&self) -> sigset_t {
        self.0
    }
}

pub fn block(set: &SigSet) {
    let mut old_mask = get_block_mask();
    for num in 0..SIGRTMAX {
        let signo = unsafe { SigNum::from_raw_uncheck(num) };
        if set.is_member(signo) {
            old_mask.add(signo);
        }
    }
    set_block_mask(old_mask);
}

pub fn unblock(set: &SigSet) {
    let mut old_mask = get_block_mask();
    for num in 0..=SIGRTMAX {
        let signo = unsafe { SigNum::from_raw_uncheck(num) };
        if set.is_member(signo) {
            old_mask.delete(signo);
        }
    }
    set_block_mask(old_mask);
}

#[inline]
pub fn get_block_mask() -> SigSet {
    SIGNAL_MASK.with(|s| s.get())
}

#[inline]
pub fn set_block_mask(set: SigSet) {
    SIGNAL_MASK.with(|s| s.set(set));
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ActionId(NonZeroU64);

impl ActionId {
    fn new() -> Self {
        #[cold]
        fn exhausted() -> ! {
            panic!("failed to generate unique Action ID: bitspace exhausted")
        }

        static COUNTER: SgxMutex<u64> = SgxMutex::new(0);

        let mut counter = COUNTER.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(id) = counter.checked_add(1) else {
            drop(counter);
            exhausted();
        };

        *counter = id;
        drop(counter);
        ActionId(NonZeroU64::new(id).unwrap())
    }

    
}

pub type Action = dyn Fn(&siginfo_t) + Send + Sync;

#[derive(Clone)]
pub struct ActionSlot {
    cur: sigaction,
    actions: BTreeMap<ActionId, Arc<Action>>,
}

impl ActionSlot {
    pub fn new(cur: &sigaction) -> Self {
        ActionSlot {
            cur: *cur,
            actions: BTreeMap::new(),
        }
    }
    pub fn set(&mut self, action: Arc<Action>) -> ActionId {
        let id = ActionId::new();
        self.actions.insert(id, action);
        id
    }
    pub fn remove(&mut self, id: ActionId) -> bool {
        self.actions.remove(&id).is_some()
    }
    pub fn get_act(&self) -> sigaction {
        self.cur
    }
}

pub struct SignalManager {
    action_set: SgxMutex<HashMap<SigNum, ActionSlot>>,
    reset_set: SgxMutex<HashSet<SigNum>>,
}

impl SignalManager {
    pub fn new() -> Self {
        SignalManager {
            action_set: SgxMutex::new(HashMap::new()),
            reset_set: SgxMutex::new(HashSet::new()),
        }
    }

    pub fn set_action(&self, signo: SigNum, act: &sigaction) {
        self.action_set
            .lock()
            .unwrap()
            .insert(signo, ActionSlot::new(act));
    }

    pub fn set_action_impl<F>(&self, signo: SigNum, act: &sigaction, action: Arc<F>) -> ActionId
    where
        F: Fn(&siginfo_t) + Sync + Send + 'static,
    {
        let action_id = if !self.action_set.lock().unwrap().contains_key(&signo) {
            let mut slot = ActionSlot::new(act);
            let id = slot.set(action);
            self.action_set.lock().unwrap().insert(signo, slot);
            id
        } else {
            self.action_set
                .lock()
                .unwrap()
                .get_mut(&signo)
                .map(|slot| slot.set(action))
                .unwrap()
        };
        action_id
    }

    pub fn get_action(&self, signo: SigNum) -> Option<ActionSlot> {
        self.action_set.lock().unwrap().get(&signo).cloned()
    }

    pub fn remove_action(&self, signo: SigNum, id: ActionId) -> bool {
        if let Some(ref mut slot) = self.action_set.lock().unwrap().get_mut(&signo) {
            slot.remove(id)
        } else {
            false
        }
    }

    pub fn clear_action(&self, signo: SigNum) -> bool {
        self.action_set.lock().unwrap().remove(&signo).is_some()
    }

    pub fn is_action_empty(&self) -> bool {
        self.action_set.lock().unwrap().is_empty()
    }

    pub fn set_reset_on_handle(&self, signo: SigNum) {
        self.reset_set.lock().unwrap().insert(signo);
    }

    pub fn is_reset_on_handle(&self, signo: SigNum) -> bool {
        self.reset_set.lock().unwrap().contains(&signo)
    }

    pub unsafe fn handler(&self, signum: i32, info: *const siginfo_t, context: *const c_void) {
        type FnSaSigaction = extern "C" fn(c_int, *const siginfo_t, *const c_void);
        type FnSaHandler = extern "C" fn(c_int);

        let signo = match SigNum::from_raw(signum) {
            Some(signo) => signo,
            None => return,
        };

        let action_slot = match self.get_action(signo) {
            Some(slot) => slot,
            None => return,
        };
        let act = action_slot.get_act();

        if self.is_reset_on_handle(signo) {
            self.clear_action(signo);
        }

        let old_mask = get_block_mask();
        block(&SigSet::from_raw(act.sa_mask));
        let is_siginfo: bool = (act.sa_flags & SA_SIGINFO) != 0;
        if is_siginfo && (act.sa_sigaction != 0) {
            let fn_sa_sigaction =
                mem::transmute::<*const (), FnSaSigaction>(act.sa_sigaction as *const ());
            fn_sa_sigaction(signo.raw(), info, context);
        } else if !is_siginfo && (act.sa_sigaction != 0) {
            let fn_sa_handler =
                mem::transmute::<*const (), FnSaHandler>(act.sa_sigaction as *const ());
            fn_sa_handler(signo.raw());
        }

        let info = info.as_ref().unwrap();
        for action in action_slot.actions.values() {
            action(info);
        }
        set_block_mask(old_mask);
    }
}
