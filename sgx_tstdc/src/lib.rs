// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

//!
//! The library is named sgx_tstdc, provides the following functions:
//!
//! * Mutex
//! * Condition
//! * Rwlock
//! * Once call
//! * thread
//! * Query CPUID inside Enclave
//! * Spin lock
//!
#![crate_name = "sgx_tstdc"]
#![crate_type = "rlib"]

#![cfg_attr(not(feature = "use_std"), no_std)]
#![feature(optin_builtin_traits)]
#![feature(const_fn)]
#![feature(dropck_eyepatch)]
#![feature(generic_param_attrs)]
#![cfg_attr(not(feature = "use_std"), feature(alloc))]

#![allow(non_camel_case_types)]
#![allow(deprecated)]

#[cfg(feature = "use_std")]
extern crate std as core;

#[cfg(not(feature = "use_std"))]
extern crate alloc;

extern crate sgx_types;

#[macro_use]
extern crate sgx_trts;

mod cpuid;
pub use cpuid::*;

mod mutex;
pub use self::mutex::{SgxMutex, SgxMutexGuard};

mod condvar;
pub use self::condvar::{SgxCond};

mod spinlock;
pub use self::spinlock::{SgxSpinlock, SgxSpinlockGuard};

#[allow(unused_must_use)]
mod rwlock;
pub use self::rwlock::{SgxRwLock, SgxRwLockReadGuard, SgxRwLockWriteGuard};

pub mod thread;
pub use self::thread::{SgxThread, rsgx_thread_self, rsgx_thread_equal};

pub mod thread_info;
pub mod once;

mod poison;
pub use poison::{PoisonError, TryLockError, TryLockResult, LockResult};




