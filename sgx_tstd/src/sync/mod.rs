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

//!
//! The Intel(R) Software Guard Extensions SDK already supports mutex and conditional
//! variable synchronization mechanisms by means of the following API and data types
//! defined in the Types and Enumerations section. Some functions included in the
//! trusted Thread Synchronization library may make calls outside the enclave (OCALLs).
//! If you use any of the APIs below, you must first import the needed OCALL functions
//! from sgx_tstd.edl. Otherwise, you will get a linker error when the enclave is
//! being built; see Calling Functions outside the Enclave for additional details.
//! The table below illustrates the primitives that the Intel(R) SGX Thread
//! Synchronization library supports, as well as the OCALLs that each API function needs.
//!

pub use alloc_crate::sync::{Arc, Weak};
pub use core::sync::atomic;

pub use self::barrier::{Barrier, BarrierWaitResult};
pub use self::condvar::{SgxCondvar, SgxThreadCondvar, WaitTimeoutResult};
pub use self::mutex::{SgxMutex, SgxMutexGuard, SgxThreadMutex};
pub use self::once::{Once, OnceState, ONCE_INIT};
pub use self::poison::{LockResult, PoisonError, TryLockError, TryLockResult};
pub use self::rwlock::{SgxRwLock, SgxRwLockReadGuard, SgxRwLockWriteGuard, SgxThreadRwLock};
pub use self::spinlock::{SgxSpinlock, SgxSpinlockGuard, SgxThreadSpinlock};

#[cfg(feature = "thread")]
pub mod mpsc;

mod barrier;
mod condvar;
mod mutex;
mod once;
mod poison;
mod rwlock;
mod spinlock;
