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

#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![feature(ptr_offset_from)]
extern crate libc;
extern crate sgx_types;

mod enclave;
pub mod mem;
pub mod time;
pub mod fd;
pub mod file;
pub mod socket;
pub mod asyncio;
pub mod env;
pub mod sys;
pub mod pipe;
pub mod event;
pub mod thread;
pub mod net;
pub use enclave::*;

