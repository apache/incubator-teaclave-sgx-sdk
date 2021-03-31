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

//! This crate provides sgx_serialize's two derive macros.
//!
//! ```rust,ignore
//! extern crate sgx_tstd as std; // Must do that!
//! #[derive(Serializable, DeSerializable)]
//! ```
//!

// The `quote!` macro requires deep recursion.
#![recursion_limit = "192"]
#![allow(unused_macros)]
#![allow(dead_code)]

#[macro_use]
extern crate quote;

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;
extern crate sgx_serialize_derive_internals as internals;

mod param;
mod bound;
#[macro_use]
mod fragment;

mod encode;
mod decode;

/// `derive_serialize` provides the `Serializable` macro for `sgx_serialize
///
/// `derive_serialize` takes one parameter typed `TokenStream` and parse the
/// input stream brought by it. Then expand the parsed the result as return
/// value.
#[proc_macro_derive(Serializable, attributes(sgx_serialize))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {

    let input = syn::parse_derive_input(&input.to_string()).unwrap();
    match encode::expand_derive_serialize(&input) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(msg) => panic!("{}", msg),
    }
}


/// `derive_deserialize` provides the `DeSerializable` macro for `sgx_serialize
///
/// `derive_deserialize` takes one parameter typed `TokenStream` and parse the
/// input stream brought by it. Then expand the parsed the result as return
/// value.
#[proc_macro_derive(DeSerializable, attributes(sgx_serialize))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {

    let input = syn::parse_derive_input(&input.to_string()).unwrap();
    match decode::expand_derive_deserialize(&input) {
        Ok(expanded) => expanded.parse().unwrap(),
        Err(msg) => panic!("{}", msg),
    }
}
