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
#![cfg_attr(
    all(target_env = "sgx", target_vendor = "mesalock"),
    feature(rustc_private)
)]

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
const GCOV_DATA_MAGIC: u32 = 0x6763_6461;
const GCOV_TAG_FUNCTION: u32 = 0x0100_0000;
const GCOV_TAG_COUNTER_ARCS: u32 = 0x01a1_0000;
const GCOV_TAG_OBJECT_SUMMARY: u32 = 0xa100_0000;
const GCOV_TAG_PROGRAM_SUMMARY: u32 = 0xa300_0000;

lazy_static! {
    static ref GCDA_FILE: SgxMutex<(c_int, u32)> = SgxMutex::new((-1, u32::MAX));
    static ref WROUT_FNS: SgxMutex<Vec<extern "C" fn()>> = SgxMutex::new(Vec::new());
    static ref RND: SgxMutex<u32> = SgxMutex::new(0);
}

pub fn cov_writeout() {
    for f in WROUT_FNS.lock().unwrap().iter() {
        f();
    }
}

#[no_mangle]
pub extern "C" fn llvm_gcov_init(
    writeout: extern "C" fn(),
    _flush: extern "C" fn(),
    _reset: extern "C" fn(),
) {
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
    GCDA_FILE
        .lock()
        .map_or_else(
            |e| panic!("llvm_gcda_summary_info failed {:?}", e),
            |tup| Ok((unsafe { File::from_raw_fd(tup.0) }, tup.1)),
        )
        .and_then(|(mut file, gcov_version)| {
            if gcov_version >= 90 {
                file.write_all(&GCOV_TAG_OBJECT_SUMMARY.to_le_bytes())?;
                file.write_all(&(2 as u32).to_le_bytes())?;
                file.write_all(&(1 as u32).to_le_bytes())?; // runs. we never merge so it's always 1
                file.write_all(&(0 as u32).to_le_bytes())?; // sum_max
            } else {
                file.write_all(&GCOV_TAG_PROGRAM_SUMMARY.to_le_bytes())?;
                file.write_all(&(3 as u32).to_le_bytes())?;
                file.write_all(&(0 as u32).to_le_bytes())?;
                file.write_all(&(0 as u32).to_le_bytes())?;
                file.write_all(&(1 as u32).to_le_bytes())?; // runs. we never merge so it's always 1
            }
            let _ = file.into_raw_fd();
            Ok(())
        })
        .unwrap_or_else(|e: std::io::Error| panic!("llvm_gcda_summary_info failed {:?}", e))
}

#[no_mangle]
pub extern "C" fn llvm_gcda_emit_arcs(num_counters: u32, counters: *const u64) {
    // we never merge
    // so `counters` is no longer * mut u64
    let cnts = unsafe { slice::from_raw_parts(counters, num_counters as usize) };

    GCDA_FILE
        .lock()
        .map_or_else(
            |e| panic!("llvm_gcda_emit_arcs failed {:?}", e),
            |tup| Ok(unsafe { File::from_raw_fd(tup.0) }),
        )
        .and_then(|mut file| {
            file.write_all(&GCOV_TAG_COUNTER_ARCS.to_le_bytes())?;
            let len: u32 = num_counters * 2;
            file.write_all(&len.to_le_bytes())?;
            for c in cnts {
                file.write_all(&c.to_le_bytes())?;
            }
            let _ = file.into_raw_fd();
            Ok(())
        })
        .unwrap_or_else(|e: std::io::Error| panic!("llvm_gcda_emit_arcs failed {:?}", e))
}

#[no_mangle]
pub extern "C" fn llvm_gcda_emit_function(ident: u32, func_checksum: u32, cfg_checksum: u32) {
    let mut len: u32 = 2;
    let use_extra_checksum: bool = GCDA_FILE.lock().map(|tup| tup.1 >= 47).unwrap();

    if use_extra_checksum {
        len += 1;
    }

    GCDA_FILE
        .lock()
        .map_or_else(
            |e| panic!("llvm_gcda_emit_function failed {:?}", e),
            |tup| Ok(unsafe { File::from_raw_fd(tup.0) }),
        )
        .and_then(|mut file| {
            file.write_all(&GCOV_TAG_FUNCTION.to_le_bytes())?;
            file.write_all(&len.to_le_bytes())?;
            file.write_all(&ident.to_le_bytes())?;
            file.write_all(&func_checksum.to_le_bytes())?;
            if use_extra_checksum {
                file.write_all(&cfg_checksum.to_le_bytes())?;
            }
            let _ = file.into_raw_fd();
            Ok(())
        })
        .unwrap_or_else(|e: std::io::Error| panic!("llvm_gcda_emit_function failed {:?}", e))
}

#[no_mangle]
pub extern "C" fn llvm_gcda_start_file(orig_filename: *const c_char, version: u32, checksum: u32) {
    let file_name_str: &CStr = unsafe { CStr::from_ptr(orig_filename) };
    let file_name = file_name_str.to_str().unwrap();

    let mut prefix = String::from(file_name);
    prefix.truncate(file_name.len() - 5);
    let orig_gcno_name = format!("{}.gcno", prefix);
    let rnd = RND.lock().unwrap();
    let new_gcno_name = format!("{}.{:08x}.gcno", prefix, *rnd);
    let new_gcda_name = format!("{}.{:08x}.gcda", prefix, *rnd);

    GCDA_FILE
        .lock()
        .map_or_else(
            |e| panic!("llvm_gcda_emit_function failed {:?}", e),
            |tup| Ok(tup),
        )
        .and_then(|mut tup| {
            copy(orig_gcno_name, new_gcno_name)?;
            let mut file = match OpenOptions::new()
                .write(true)
                .append(false)
                .open(&new_gcda_name)
            {
                Ok(file) => file,
                Err(_) => File::create(&new_gcda_name)?,
            };

            let c3: u8 = ((version >> 24) & 0x000000FF) as u8;
            let c2: u8 = ((version >> 16) & 0x000000FF) as u8;
            let c1: u8 = ((version >> 8) & 0x000000FF) as u8;
            let parsed_gcov_version: u32 = if c3 >= 'A' as u8 {
                ((c3 - 'A' as u8) as u32) * 100
                    + ((c2 - '0' as u8) as u32) * 10
                    + (c1 - '0' as u8) as u32
            } else {
                ((c3 - '0' as u8) as u32) * 10 + (c1 - '0' as u8) as u32
            };

            tup.1 = parsed_gcov_version;

            file.write_all(&GCOV_DATA_MAGIC.to_le_bytes()).unwrap();
            file.write_all(&version.to_le_bytes()).unwrap();
            file.write_all(&checksum.to_le_bytes()).unwrap();

            tup.0 = file.into_raw_fd();

            Ok(())
        })
        .unwrap_or_else(|e: std::io::Error| panic!("llvm_gcda_start_file failed {:?}", e))
}

#[no_mangle]
pub extern "C" fn llvm_gcda_end_file() {
    if let Ok(ref tup) = GCDA_FILE.lock() {
        let fd = &tup.0;
        let mut file = unsafe { File::from_raw_fd(*fd) };
        let eof: u64 = 0;
        file.write_all(&eof.to_be_bytes()).unwrap();
    // Let it drop
    } else {
        panic!("llvm_gcda_end_file failed!");
    }
}

#[no_mangle]
pub extern "C" fn llvm_gcda_increment_indirect_counter(
    predecessor: *mut u32,
    counters: *mut *mut u64,
) {
    let counter: *mut u64;
    let pred: u32;

    if predecessor.is_null() || counters.is_null() {
        return;
    }

    pred = unsafe { *predecessor };

    if pred == 0xFFFF_FFFF {
        return;
    }

    counter = unsafe { *counters.offset(pred as isize) };

    if !counter.is_null() {
        unsafe {
            *counter = *counter + 1;
        }
    }
}
