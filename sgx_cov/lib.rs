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
