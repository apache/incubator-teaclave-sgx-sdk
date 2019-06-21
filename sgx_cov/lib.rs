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

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

#[cfg(not(target_env = "sgx"))]
use std::prelude::v1::*;

#[cfg(target_env = "sgx")]
extern crate profiler_builtins as _;

extern crate sgx_rand;
extern crate sgx_types;

use lazy_static::lazy_static;
use sgx_rand::Rng;
use sgx_types::{c_char, c_int};
use std::ffi::CStr;
use std::io::Write;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::slice;
use std::sync::{Once, SgxMutex};
use std::untrusted::fs::{copy, File, OpenOptions};

static INIT: Once = Once::new();

lazy_static! {
    static ref GCDA_FILE: SgxMutex<c_int> = SgxMutex::new(-1);
    static ref WROUT_FNS: SgxMutex<Vec<extern "C" fn()>> = SgxMutex::new(Vec::new());
    static ref RND: SgxMutex<u32> = SgxMutex::new(0);
}

pub fn cov_writeout() {
    for f in WROUT_FNS.lock().unwrap().iter() {
        f();
    }
}

#[no_mangle]
pub extern "C" fn llvm_gcov_init(writeout: extern "C" fn(), _flush: extern "C" fn()) {
    INIT.call_once(|| {
        let mut rng = sgx_rand::thread_rng();
        let mut rnd = RND.lock().unwrap();
        *rnd = rng.gen();
    });
    let mut writeout_fns = WROUT_FNS.lock().unwrap();
    writeout_fns.push(writeout);
}

#[no_mangle]
pub extern "C" fn llvm_gcda_summary_info() {
    match GCDA_FILE.lock() {
        Ok(fd) => {
            let mut file = unsafe { File::from_raw_fd(*fd) };

            let summary_tag: u32 = 0xa1;
            file.write_all(&summary_tag.to_be_bytes()).unwrap();
            let len: u32 = 9;
            file.write_all(&len.to_le_bytes()).unwrap();
            let zero: u32 = 0;
            let one: u32 = 1;
            file.write_all(&zero.to_le_bytes()).unwrap();
            file.write_all(&zero.to_le_bytes()).unwrap();
            file.write_all(&one.to_le_bytes()).unwrap();
            for _ in 0..(len - 3) {
                file.write_all(&zero.to_le_bytes()).unwrap();
            }
            let prog_tag: u32 = 0xa3;
            file.write_all(&prog_tag.to_be_bytes()).unwrap();
            file.write_all(&zero.to_le_bytes()).unwrap();

            // Prevent it from drop
            let _ = file.into_raw_fd();
        }
        Err(_) => panic!("llvm_gcda_emit_arcs failed"),
    }
}

#[no_mangle]
pub extern "C" fn llvm_gcda_emit_arcs(num_counters: u32, counters: *const u64) {
    let cnts = unsafe { slice::from_raw_parts(counters, num_counters as usize) };
    match GCDA_FILE.lock() {
        Ok(fd) => {
            let mut file = unsafe { File::from_raw_fd(*fd) };

            let arcs_tag: u32 = 0xa101;
            file.write_all(&arcs_tag.to_be_bytes()).unwrap();
            let len: u32 = num_counters * 2;
            file.write_all(&len.to_le_bytes()).unwrap();
            for i in 0..num_counters {
                file.write_all(&cnts[i as usize].to_le_bytes()).unwrap();
            }

            // Prevent it from drop
            let _ = file.into_raw_fd();
        }
        Err(_) => panic!("llvm_gcda_emit_arcs failed"),
    }
}

#[no_mangle]
pub extern "C" fn llvm_gcda_emit_function(
    ident: u32,
    raw_func_name: *const c_char,
    fchecksum: u32,
    use_extra_checksum: u8,
    cfg_checksum: u32,
) {
    let func_name_str: &CStr = unsafe { CStr::from_ptr(raw_func_name) };
    let func_name = func_name_str.to_str().unwrap();
    let mut len: u32 = 2;
    if use_extra_checksum != 0 {
        len += 1;
    }
    let str_len = (1 + func_name.len() / 4) as u32;
    len += str_len;

    match GCDA_FILE.lock() {
        Ok(fd) => {
            let mut file = unsafe { File::from_raw_fd(*fd) };

            let func_tag: u32 = 1;
            file.write_all(&func_tag.to_be_bytes()).unwrap();
            file.write_all(&len.to_le_bytes()).unwrap();
            file.write_all(&ident.to_le_bytes()).unwrap();
            file.write_all(&fchecksum.to_le_bytes()).unwrap();
            if use_extra_checksum != 0 {
                file.write_all(&cfg_checksum.to_le_bytes()).unwrap();
            }
            file.write_all(&str_len.to_le_bytes()).unwrap();
            file.write_all(func_name.as_bytes()).unwrap();

            let zero: u8 = 0;
            let padding_size = 4 - func_name.len() % 4;
            for _ in 0..padding_size {
                file.write_all(&zero.to_le_bytes()).unwrap();
            }

            // Prevent it from drop
            let _ = file.into_raw_fd();
        }
        Err(_) => panic!("llvm_gcda_emit_function failed"),
    }
}

#[no_mangle]
pub extern "C" fn llvm_gcda_start_file(
    raw_file_name: *const c_char,
    ver: *const u8,
    checksum: u32,
) {
    let file_name_str: &CStr = unsafe { CStr::from_ptr(raw_file_name) };
    let file_name = file_name_str.to_str().unwrap();
    let version = unsafe { slice::from_raw_parts(ver, 4) };

    let mut prefix = String::from(file_name);
    prefix.truncate(file_name.len() - 5);
    let orig_gcno_name = format!("{}.gcno", prefix);
    let rnd = RND.lock().unwrap();
    let new_gcno_name = format!("{}.{:08x}.gcno", prefix, *rnd);
    let new_gcda_name = format!("{}.{:08x}.gcda", prefix, *rnd);

    match GCDA_FILE.lock() {
        Ok(mut fd) => {
            copy(orig_gcno_name, new_gcno_name).unwrap();
            let mut file = match OpenOptions::new()
                .write(true)
                .append(false)
                .open(&new_gcda_name)
            {
                Ok(file) => file,
                Err(_) => File::create(&new_gcda_name).unwrap(),
            };
            file.write_all(b"adcg").unwrap();
            file.write_all(version).unwrap();
            file.write_all(&checksum.to_le_bytes()).unwrap();
            *fd = file.into_raw_fd();
        }
        Err(_) => panic!("llvm_gcda_start_file failed!"),
    }
}

#[no_mangle]
pub extern "C" fn llvm_gcda_end_file() {
    match GCDA_FILE.lock() {
        Ok(fd) => {
            let mut file = unsafe { File::from_raw_fd(*fd) };
            let eof: u64 = 0;
            file.write_all(&eof.to_be_bytes()).unwrap();
            // Let it drop
        }
        Err(_) => panic!("llvm_gcda_end_file failed!"),
    }
}
