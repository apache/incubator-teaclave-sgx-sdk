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

pub(crate) mod epc;
#[cfg(not(any(feature = "sim", feature = "hyper")))]
pub(crate) mod layout;
pub(crate) mod mem;
pub(crate) mod perm;
pub(crate) mod tcs;
#[cfg(not(any(feature = "sim", feature = "hyper")))]
pub(crate) mod trim;

pub use epc::{PageFlags, PageInfo, PageRange, PageType};
pub use mem::{apply_epc_pages, trim_epc_pages};
pub use perm::{modpr_ocall, mprotect_ocall};
