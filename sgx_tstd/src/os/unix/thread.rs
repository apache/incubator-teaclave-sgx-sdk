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

//! Unix-specific extensions to primitives in the `std::thread` module.

#[allow(deprecated)]
use crate::os::unix::raw::pthread_t;
use crate::sys_common::{AsInner, IntoInner};
use crate::thread::JoinHandle;

#[allow(deprecated)]
pub type RawPthread = pthread_t;

/// Unix-specific extensions to [`JoinHandle`].
pub trait JoinHandleExt {
    /// Extracts the raw pthread_t without taking ownership
    fn as_pthread_t(&self) -> RawPthread;

    /// Consumes the thread, returning the raw pthread_t
    ///
    /// This function **transfers ownership** of the underlying pthread_t to
    /// the caller. Callers are then the unique owners of the pthread_t and
    /// must either detach or join the pthread_t once it's no longer needed.
    fn into_pthread_t(self) -> RawPthread;
}

impl<T> JoinHandleExt for JoinHandle<T> {
    fn as_pthread_t(&self) -> RawPthread {
        self.as_inner().id() as RawPthread
    }

    fn into_pthread_t(self) -> RawPthread {
        self.into_inner().into_id() as RawPthread
    }
}
