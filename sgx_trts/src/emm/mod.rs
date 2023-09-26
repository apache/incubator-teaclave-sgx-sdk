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

#[cfg(not(any(feature = "sim", feature = "hyper")))]
pub(crate) mod alloc;
pub(crate) mod bitmap;
pub(crate) mod ema;
pub(crate) mod init;
#[cfg(not(any(feature = "sim", feature = "hyper")))]
pub(crate) mod layout;
pub(crate) mod ocall;
pub(crate) mod page;
pub(crate) mod pfhandler;
pub(crate) mod range;
pub(crate) mod tcs;

pub use ocall::{alloc_ocall, modify_ocall};
pub use page::{AllocFlags, PageInfo, PageRange, PageType, ProtFlags};
pub use pfhandler::{PfHandler, PfInfo, Pfec, PfecBits};

pub use range::{
    rts_mm_alloc, rts_mm_commit, rts_mm_dealloc, rts_mm_modify_perms, rts_mm_modify_type,
    rts_mm_uncommit,
};
pub use range::{
    user_mm_alloc, user_mm_commit, user_mm_dealloc, user_mm_modify_perms, user_mm_modify_type,
    user_mm_uncommit,
};
