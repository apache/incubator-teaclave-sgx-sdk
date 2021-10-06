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

use sgx_align_struct_attribute::sgx_align;
use sgx_types::*;

//no repr c
// #[sgx_align(align=1, size=64)]
// pub struct user_struct {
//     pub a: u8,
//     pub b: [u16; 2],
//     pub c: u32,
//     pub d: [u64; 6],
// }

// Attributes illegal
// #[sgx_align(align=1, size=64)]
// #[repr(align(4))]
// pub struct user_struct {
//     pub a: u8,
//     pub b: [u16; 2],
//     pub c: u32,
//     pub d: [u64; 6],
// }

// //no align
// s! {
//     #[sgx_align(align=1, size=64)]
//     pub struct user_struct {
//         pub a: u8,
//         pub b: [u16; 2],
//         pub c: u32,
//         pub d: [u64; 6],
//     }
// }

// //no align
// s! {
//     #[sgx_align(align=16, size=64)]
//     pub struct user_struct {
//         pub a: u8,
//         pub b: [u16; 2],
//         pub c: u32,
//         pub d: [u64; 6],
//     }
// }

//align
s! {
    #[sgx_align(align=1, size=16)]
    pub struct struct_align_128_t {
        pub key: sgx_key_128bit_t,
    }
}

// //no align
// s! {
//     #[sgx_align(align=32, size=64)]
//     pub struct struct_no_align_t {
//         pub key1: sgx_key_128bit_t,
//         pub key2: sgx_key_128bit_t,
//         pub key3: sgx_key_128bit_t,
//         pub key4: sgx_key_128bit_t,
//     }
// }
