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

#![cfg_attr(not(target_vendor = "teaclave"), no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]
#![feature(once_cell)]
#![allow(clippy::missing_safety_doc)]

#[cfg(not(target_vendor = "teaclave"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_trts;
extern crate sgx_types;

use sgx_trts::rand::Rng;
use sgx_types::types::c_char;
use std::ffi::CStr;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::lazy::SyncLazy;
use std::slice;
use std::string::String;
use std::sync::{Mutex, Once};
use std::untrusted::fs::{copy, File, OpenOptions};
use std::vec::Vec;

static INIT: Once = Once::new();
const GCOV_DATA_MAGIC: u32 = 0x6763_6461;
const GCOV_TAG_FUNCTION: u32 = 0x0100_0000;
const GCOV_TAG_COUNTER_ARCS: u32 = 0x01a1_0000;
const GCOV_TAG_OBJECT_SUMMARY: u32 = 0xa100_0000;
const GCOV_TAG_PROGRAM_SUMMARY: u32 = 0xa300_0000;

static GCDA_FILE: SyncLazy<Mutex<(Option<File>, u32)>> =
    SyncLazy::new(|| Mutex::new((None, u32::MAX)));
static WROUT_FNS: SyncLazy<Mutex<Vec<extern "C" fn()>>> = SyncLazy::new(|| Mutex::new(Vec::new()));
static RND: SyncLazy<Mutex<u32>> = SyncLazy::new(|| Mutex::new(0));

pub fn cov_writeout() {
    for f in WROUT_FNS.lock().unwrap().iter() {
        f();
    }
}

#[no_mangle]
pub unsafe extern "C" fn llvm_gcov_init(
    writeout: extern "C" fn(),
    _flush: extern "C" fn(),
    _reset: extern "C" fn(),
) {
    INIT.call_once(|| {
        let mut rnd = RND.lock().unwrap();
        *rnd = Rng::new().next_u32();
    });
    let mut writeout_fns = WROUT_FNS.lock().unwrap();
    writeout_fns.push(writeout);
}

#[no_mangle]
pub unsafe extern "C" fn llvm_gcda_summary_info() {
    let mut gcda_file = GCDA_FILE
        .lock()
        .unwrap_or_else(|e| panic!("llvm_gcda_summary_info failed {:?}", e));

    let gcov_version = gcda_file.1;
    gcda_file
        .0
        .as_mut()
        .ok_or_else(|| Error::from(ErrorKind::NotFound))
        .and_then(|file| {
            if gcov_version >= 90 {
                file.write_all(&GCOV_TAG_OBJECT_SUMMARY.to_le_bytes())?;
                file.write_all(&(2_u32).to_le_bytes())?;
                file.write_all(&(1_u32).to_le_bytes())?; // runs. we never merge so it's always 1
                file.write_all(&(0_u32).to_le_bytes())?; // sum_max
            } else {
                file.write_all(&GCOV_TAG_PROGRAM_SUMMARY.to_le_bytes())?;
                file.write_all(&(3_u32).to_le_bytes())?;
                file.write_all(&(0_u32).to_le_bytes())?;
                file.write_all(&(0_u32).to_le_bytes())?;
                file.write_all(&(1_u32).to_le_bytes())?; // runs. we never merge so it's always 1
            }
            Ok(())
        })
        .unwrap_or_else(|e: std::io::Error| panic!("llvm_gcda_summary_info failed {:?}", e))
}

#[no_mangle]
pub unsafe extern "C" fn llvm_gcda_emit_arcs(num_counters: u32, counters: *const u64) {
    let mut gcda_file = GCDA_FILE
        .lock()
        .unwrap_or_else(|e| panic!("llvm_gcda_summary_info failed {:?}", e));

    gcda_file
        .0
        .as_mut()
        .ok_or_else(|| Error::from(ErrorKind::NotFound))
        .and_then(|file| {
            file.write_all(&GCOV_TAG_COUNTER_ARCS.to_le_bytes())?;
            let len = num_counters * 2;
            file.write_all(&len.to_le_bytes())?;

            let cnts = slice::from_raw_parts(counters, num_counters as usize);
            for c in cnts {
                file.write_all(&c.to_le_bytes())?;
            }
            Ok(())
        })
        .unwrap_or_else(|e: std::io::Error| panic!("llvm_gcda_summary_info failed {:?}", e))
}

#[no_mangle]
pub unsafe extern "C" fn llvm_gcda_emit_function(
    ident: u32,
    func_checksum: u32,
    cfg_checksum: u32,
) {
    let mut gcda_file = GCDA_FILE
        .lock()
        .unwrap_or_else(|e| panic!("llvm_gcda_emit_function failed {:?}", e));

    let mut len = 2_u32;
    let use_extra_checksum = gcda_file.1 >= 47;
    if use_extra_checksum {
        len += 1;
    }

    gcda_file
        .0
        .as_mut()
        .ok_or_else(|| Error::from(ErrorKind::NotFound))
        .and_then(|file| {
            file.write_all(&GCOV_TAG_FUNCTION.to_le_bytes())?;
            file.write_all(&len.to_le_bytes())?;
            file.write_all(&ident.to_le_bytes())?;
            file.write_all(&func_checksum.to_le_bytes())?;
            if use_extra_checksum {
                file.write_all(&cfg_checksum.to_le_bytes())?;
            }
            Ok(())
        })
        .unwrap_or_else(|e: std::io::Error| panic!("llvm_gcda_emit_function failed {:?}", e))
}

#[no_mangle]
pub unsafe extern "C" fn llvm_gcda_start_file(
    orig_filename: *const c_char,
    version: u32,
    checksum: u32,
) {
    let file_name = CStr::from_ptr(orig_filename);
    let file_name = file_name.to_str().unwrap();

    let mut prefix = String::from(file_name);
    prefix.truncate(file_name.len() - 5);
    let orig_gcno_name = format!("{}.gcno", prefix);
    let rnd = RND.lock().unwrap();
    let new_gcno_name = format!("{}.{:08x}.gcno", prefix, *rnd);
    let new_gcda_name = format!("{}.{:08x}.gcda", prefix, *rnd);

    GCDA_FILE
        .lock()
        .map_or_else(|e| panic!("llvm_gcda_emit_function failed {:?}", e), Ok)
        .and_then(|mut tup| {
            copy(orig_gcno_name, new_gcno_name)?;
            let mut file = OpenOptions::new()
                .write(true)
                .append(false)
                .open(&new_gcda_name)
                .or_else(|_| File::create(&new_gcda_name))?;

            let c3 = ((version >> 24) & 0x000000FF) as u8;
            let c2 = ((version >> 16) & 0x000000FF) as u8;
            let c1 = ((version >> 8) & 0x000000FF) as u8;
            let parsed_gcov_version = if c3 >= b'A' {
                ((c3 - b'A') as u32) * 100 + ((c2 - b'0') as u32) * 10 + (c1 - b'0') as u32
            } else {
                ((c3 - b'0') as u32) * 10 + (c1 - b'0') as u32
            };

            tup.1 = parsed_gcov_version;

            file.write_all(&GCOV_DATA_MAGIC.to_le_bytes()).unwrap();
            file.write_all(&version.to_le_bytes()).unwrap();
            file.write_all(&checksum.to_le_bytes()).unwrap();

            tup.0 = Some(file);
            Ok(())
        })
        .unwrap_or_else(|e: Error| panic!("llvm_gcda_start_file failed {:?}", e))
}

#[no_mangle]
pub unsafe extern "C" fn llvm_gcda_end_file() {
    GCDA_FILE
        .lock()
        .map_or_else(
            |e| panic!("llvm_gcda_end_file failed {:?}", e),
            |mut tup| tup.0.take().ok_or_else(|| Error::from(ErrorKind::NotFound)),
        )
        .and_then(|mut file| {
            let eof: u64 = 0;
            file.write_all(&eof.to_be_bytes())?;
            Ok(())
        })
        .unwrap_or_else(|e: Error| panic!("llvm_gcda_end_file failed {:?}", e))
}

#[no_mangle]
pub unsafe extern "C" fn llvm_gcda_increment_indirect_counter(
    predecessor: *mut u32,
    counters: *mut *mut u64,
) {
    if predecessor.is_null() || counters.is_null() {
        return;
    }

    let pred = *predecessor;
    if pred == 0xFFFF_FFFF {
        return;
    }

    let counter = *counters.offset(pred as isize);
    if !counter.is_null() {
        *counter += 1
    }
}
