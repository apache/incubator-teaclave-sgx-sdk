// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;
use sgx_types::metadata::*;
use sgx_trts::enclave;
//use sgx_trts::{is_x86_feature_detected, is_cpu_feature_supported};
use std::string::String;
use std::vec::Vec;
use std::io::{self, Write};
use std::slice;
use std::backtrace::{self, PrintFormat};

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

    let _ = backtrace::enable_backtrace("enclave.signed.so", PrintFormat::Full);

    let gd = enclave::SgxGlobalData::new();
    println!("gd: {} {} {} {} ", gd.get_static_tcs_num(), gd.get_eremove_tcs_num(), gd.get_dyn_tcs_num(), gd.get_tcs_max_num());
    let (static_num, eremove_num, dyn_num) = get_thread_num();
    println!("static: {} eremove: {} dyn: {}", static_num, eremove_num, dyn_num);

    unsafe {
        println!("EDMM: {}, feature: {}", EDMM_supported, g_cpu_feature_indicator);
    }
    if is_x86_feature_detected!("sgx") {
        println!("supported sgx");
    }

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