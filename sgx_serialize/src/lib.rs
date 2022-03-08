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

//! Support code for encoding and decoding types.

/*
Core encoding and decoding interfaces.
*/

#![cfg_attr(all(feature = "tserialize", not(target_vendor = "teaclave")), no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]
#![feature(box_syntax)]
#![feature(never_type)]
#![feature(nll)]
#![feature(associated_type_bounds)]
#![feature(min_specialization)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_slice)]
#![feature(new_uninit)]

#[cfg(all(feature = "tserialize", feature = "userialize"))]
compile_error!(
    "feature \"tserialize\" and feature \"userialize\" cannot be enabled at the same time"
);

#[cfg(not(any(feature = "tserialize", feature = "userialize")))]
compile_error!("need to enable feature \"tserialize\" or feature \"userialize\"");

#[cfg(all(feature = "tserialize", not(target_vendor = "teaclave")))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_types;

mod collection;
mod serialize;
mod types;

pub mod json;

pub mod leb128;
pub mod opaque;

pub use self::serialize::{Decodable, Decoder, Encodable, Encoder};

#[cfg(feature = "derive")]
pub use sgx_serialize_derive::{Deserialize, Serialize};
