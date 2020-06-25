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

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::io::{self, Write};
use std::slice;

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {

    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    let word:[u8;4] = [82, 117, 115, 116];
    // An vector
    let word_vec:Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8")
                                               .as_str();

    // Ocall to normal world for output
    println!("{}", &hello_string);

    sgx_status_t::SGX_SUCCESS
}

#[link(name = "sgx_trts")]
extern {
    static g_cpu_feature_indicator: uint64_t;
    static EDMM_supported: c_int;
}


fn get_thread_num() -> (u32, u32, u32) {
    let gd = unsafe {
        let p = enclave::rsgx_get_global_data();
        &*p
    };

    let mut static_thread_num: u32 = 0;
    let mut eremove_thread_num: u32 = 0;
    let mut dyn_thread_num: u32 = 0;
    let layout_table = &gd.layout_table[0..gd.layout_entry_num as usize];
    unsafe { traversal_layout(&mut static_thread_num, &mut dyn_thread_num, &mut eremove_thread_num, layout_table); }

    unsafe fn traversal_layout(static_num: &mut u32, dyn_num: &mut u32, eremove_num: &mut u32, layout_table: &[layout_t])
    {
        for (i, layout) in layout_table.iter().enumerate() {
            if !is_group_id!(layout.group.id as u32) {
                if (layout.entry.attributes & PAGE_ATTR_EADD) != 0 {
                    if (layout.entry.content_offset != 0) && (layout.entry.si_flags == SI_FLAGS_TCS) {
                        if (layout.entry.attributes & PAGE_ATTR_EREMOVE) == 0 {
                            *static_num += 1;
                        } else {
                            *eremove_num += 1;
                        }
                    }
                }
                if (layout.entry.attributes & PAGE_ATTR_POST_ADD) != 0 {
                    if layout.entry.id == LAYOUT_ID_TCS_DYN as u16 {
                        *dyn_num += 1;
                    }
                }
            } else {
                for _ in 0..layout.group.load_times {
                    traversal_layout(static_num, dyn_num, eremove_num, &layout_table[i - layout.group.entry_count as usize..i])
                }
            }
        }
    }
    (static_thread_num, eremove_thread_num, dyn_thread_num)
}
