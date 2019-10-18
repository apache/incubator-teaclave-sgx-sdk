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

//!
//! The Intel(R) Software Guard Extensions SDK already supports mutex and conditional
//! variable synchronization mechanisms by means of the following APIand data types
//! defined in the Types and Enumerations section. Some functions included in the
//! trusted Thread Synchronization library may make calls outside the enclave (OCALLs).
//! If you use any of the APIs below, you must first import the needed OCALL functions
//! from sgx_tstdc.edl. Otherwise, you will get a linker error when the enclave is
//! being built; see Calling Functions outside the Enclave for additional details.
//! The table below illustrates the primitives that the Intel(R) SGX Thread
//! Synchronization library supports, as well as the OCALLs that each API function needs.
//!

pub use alloc_crate::sync::{Arc, Weak};
pub use core::sync::atomic;

pub use self::barrier::{Barrier, BarrierWaitResult};
pub use self::condvar::{SgxCondvar, SgxThreadCondvar};
pub use self::mutex::{SgxMutex, SgxMutexGuard, SgxThreadMutex};
pub use self::remutex::{SgxReentrantMutex, SgxReentrantMutexGuard, SgxReentrantThreadMutex};
pub use self::once::{Once, OnceState, ONCE_INIT};
pub use self::rwlock::{SgxRwLock, SgxRwLockReadGuard, SgxRwLockWriteGuard, SgxThreadRwLock};
pub use self::spinlock::{SgxSpinlock, SgxSpinlockGuard, SgxThreadSpinlock};
pub use crate::sys_common::poison::{PoisonError, TryLockError, TryLockResult, LockResult};
pub mod mpsc;
mod barrier;
mod condvar;
mod mutex;
mod remutex;
mod once;
mod rwlock;
mod spinlock;
