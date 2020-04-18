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

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;

mod serialize;
pub use self::serialize::{Decoder, Encoder, DeSerializable, Serializable, SerializeHelper, DeSerializeHelper};

mod opaque;
mod leb128;

