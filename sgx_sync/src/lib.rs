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

#![no_std]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]
#![deny(unused_features)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(dropck_eyepatch)]
#![feature(hashmap_internals)]
#![feature(linked_list_remove)]
#![feature(negative_impls)]
#![feature(never_type)]
#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]

extern crate alloc;

extern crate sgx_trts;
#[macro_use]
extern crate sgx_types;

mod barrier;
mod condvar;
mod futex;
mod lazy_lock;
mod lock_api;
mod mutex;
mod once;
mod once_lock;
mod remutex;
mod rwlock;
mod spin;
mod sys;

#[cfg(feature = "capi")]
pub mod capi;

pub use sys::ocall::Timespec;

pub use barrier::{Barrier, BarrierWaitResult};
pub use condvar::Condvar;
pub use futex::{Futex, FutexClockId, FutexFlags, FutexOp};
pub use lazy_lock::LazyLock;
pub use lock_api::{RawMutex, RawRwLock};
pub use mutex::{MovableMutex, StaticMutex, StaticMutexGuard};
pub use once::{Once, OnceState, ONCE_INIT};
pub use once_lock::OnceLock;
pub use remutex::{ReentrantMutex, ReentrantMutexGuard};
pub use rwlock::{MovableRwLock, StaticRwLock, StaticRwLockReadGuard, StaticRwLockWriteGuard};
pub use spin::{SpinMutex, SpinMutexGuard, SpinRwLock, SpinRwLockReadGuard, SpinRwLockWriteGuard};
