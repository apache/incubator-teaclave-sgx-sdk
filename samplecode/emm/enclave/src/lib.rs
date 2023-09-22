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
#![feature(pointer_byte_offsets)]

#[cfg(not(target_vendor = "teaclave"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_trts;
extern crate sgx_types;

use core::ffi::c_void;
use sgx_trts::emm::{self, AllocFlags, PageInfo, PageType, PfHandler, PfInfo, ProtFlags};
use sgx_trts::veh::HandleResult;
use sgx_types::error::errno::{EACCES, EEXIST, EINVAL, EPERM};
use sgx_types::error::SgxStatus;
use std::io::{self, Write};
use std::slice;
use std::string::String;
use std::string::ToString;
use std::thread;
use std::vec::Vec;

const ALLOC_SIZE: usize = 0x2000;
const SE_PAGE_SIZE: usize = 0x1000;

#[no_mangle]
fn ecall_test_sgx_mm_unsafe() -> SgxStatus {
    let input_string = "Enclave memory management test: \n";
    unsafe {
        say_something(input_string.as_ptr(), input_string.len());
    }
    test_emm_alloc_dealloc();
    test_stack_expand();
    test_commit_and_uncommit();
    test_modify_types();
    test_dynamic_expand_tcs();
    test_modify_perms()
}

#[derive(Clone, Copy, Default)]
struct PfData {
    pf: PfInfo,
    access: i32,
    addr_expected: usize,
}

pub extern "C" fn permission_pfhandler(info: &mut PfInfo, priv_data: *mut c_void) -> HandleResult {
    let mut pd = unsafe { &mut *(priv_data as *mut PfData) };
    pd.pf = *info;

    let addr = pd.pf.maddr as usize;
    let prot = ProtFlags::from_bits(pd.access as u8).unwrap();
    let rw_bit = unsafe { pd.pf.pfec.bits.rw() };
    if (rw_bit == 1) && (prot == ProtFlags::W) {
        emm::user_mm_modify_perms(addr, SE_PAGE_SIZE, ProtFlags::W | ProtFlags::R);
    } else if (rw_bit == 0) && prot.contains(ProtFlags::R) {
        emm::user_mm_modify_perms(addr, SE_PAGE_SIZE, prot);
    } else {
        panic!()
    }

    HandleResult::Execution
}

#[no_mangle]
fn test_modify_perms() -> SgxStatus {
    let mut pd = PfData::default();

    // example 1:
    let base = emm::user_mm_alloc(
        None,
        ALLOC_SIZE,
        AllocFlags::COMMIT_NOW,
        PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W,
        },
        Some(permission_pfhandler),
        Some(&mut pd as *mut PfData as *mut c_void),
    )
    .unwrap();

    let data = unsafe { (base as *const u8).read() };
    assert!(data == 0);

    // read success without PF
    assert!(unsafe { pd.pf.pfec.errcd } == 0);
    unsafe { (base as *mut u8).write(0xFF) };

    // write success without PF
    assert!(unsafe { pd.pf.pfec.errcd } == 0);

    let res = emm::user_mm_modify_perms(base, ALLOC_SIZE / 2, ProtFlags::R);
    assert!(res.is_ok());

    pd.access = ProtFlags::R.bits() as i32;
    let data = unsafe { (base as *const u8).read() };
    assert!(data == 0xFF);
    // read success without PF
    assert!(unsafe { pd.pf.pfec.errcd } == 0);

    // 出问题了
    pd.access = ProtFlags::W.bits() as i32;
    let count = (ALLOC_SIZE - 1) as isize;
    unsafe {
        let ptr = (base as *mut u8).byte_offset(count);
        ptr.write(0xFF);
    };
    // write success without PF
    assert!(unsafe { pd.pf.pfec.errcd } == 0);

    pd.access = ProtFlags::W.bits() as i32;
    unsafe { (base as *mut u8).write(0xFF) };
    // write success with PF
    assert!(unsafe { pd.pf.pfec.errcd } != 0);

    // write indicated with PFEC
    assert!(unsafe { pd.pf.pfec.bits.rw() } == 1);

    println!(
        "{}",
        "Successfully run modify permissions and customized page fault handler!"
    );
    SgxStatus::Success
}

#[no_mangle]
fn test_dynamic_expand_tcs() -> SgxStatus {
    thread::Builder::new()
        .name("thread1".to_string())
        .spawn(move || {
            println!("Hello, this is a spawned thread!");
        });

    for _ in 0..40 {
        let _t = thread::spawn(move || {
            println!("Hello, this is a spawned thread!");
        });
    }

    println!("{}", "Successfully dynamic expand tcs!");
    SgxStatus::Success
}

#[no_mangle]
fn test_modify_types() -> SgxStatus {
    // example 1:
    let base = emm::user_mm_alloc(
        None,
        SE_PAGE_SIZE,
        AllocFlags::COMMIT_NOW,
        PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W,
        },
        None,
        None,
    )
    .unwrap();

    let res = emm::user_mm_modify_type(base, SE_PAGE_SIZE, PageType::Tcs);
    assert!(res.is_ok());

    let res = emm::user_mm_uncommit(base, SE_PAGE_SIZE);
    assert!(res.is_ok());

    // example 2:
    let base = emm::user_mm_alloc(
        None,
        SE_PAGE_SIZE,
        AllocFlags::COMMIT_NOW,
        PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W,
        },
        None,
        None,
    )
    .unwrap();

    let res = emm::user_mm_modify_perms(base, SE_PAGE_SIZE, ProtFlags::NONE);
    assert!(res.is_ok());

    let res = emm::user_mm_uncommit(base, SE_PAGE_SIZE);
    assert!(res.is_ok());

    // example 3:
    let res = emm::user_mm_dealloc(0, ALLOC_SIZE);
    assert!(res == Err(EINVAL));

    let base = emm::user_mm_alloc(
        None,
        ALLOC_SIZE,
        AllocFlags::COMMIT_NOW,
        PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W,
        },
        None,
        None,
    )
    .unwrap();

    let res = emm::user_mm_modify_type(base + SE_PAGE_SIZE, SE_PAGE_SIZE, PageType::Frist);
    assert!(res == Err(EPERM));

    let res = emm::user_mm_modify_perms(
        base + SE_PAGE_SIZE,
        SE_PAGE_SIZE,
        ProtFlags::R | ProtFlags::X,
    );
    assert!(res.is_ok());

    let res = emm::user_mm_modify_type(base + SE_PAGE_SIZE, SE_PAGE_SIZE, PageType::Tcs);
    assert!(res == Err(EACCES));

    let res = emm::user_mm_modify_type(base, SE_PAGE_SIZE, PageType::Tcs);
    assert!(res.is_ok());

    let res = emm::user_mm_uncommit(base, ALLOC_SIZE);
    assert!(res.is_ok());

    let res = emm::user_mm_modify_type(base, SE_PAGE_SIZE, PageType::Tcs);
    assert!(res == Err(EACCES));

    let res = emm::user_mm_dealloc(base, ALLOC_SIZE);
    assert!(res.is_ok());

    println!("{}", "Successfully run modify types!");
    SgxStatus::Success
}

#[no_mangle]
fn test_commit_and_uncommit() -> SgxStatus {
    let res = emm::user_mm_dealloc(0, ALLOC_SIZE);
    assert!(res == Err(EINVAL));

    let base = emm::user_mm_alloc(
        None,
        ALLOC_SIZE,
        AllocFlags::COMMIT_NOW,
        PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W,
        },
        None,
        None,
    )
    .unwrap();

    let res = emm::user_mm_commit(base, ALLOC_SIZE);
    assert!(res.is_ok());

    let res = emm::user_mm_alloc(
        Some(base),
        ALLOC_SIZE,
        AllocFlags::COMMIT_NOW | AllocFlags::FIXED,
        PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W,
        },
        None,
        None,
    );
    assert!(res == Err(EEXIST));

    let res = emm::user_mm_uncommit(base, ALLOC_SIZE);
    assert!(res.is_ok());

    let res = emm::user_mm_uncommit(base, ALLOC_SIZE);
    assert!(res.is_ok());

    let res = emm::user_mm_commit(base, ALLOC_SIZE);
    assert!(res.is_ok());

    let res = emm::user_mm_dealloc(base, ALLOC_SIZE);
    assert!(res.is_ok());

    let res = emm::user_mm_dealloc(base, ALLOC_SIZE);
    assert!(res == Err(EINVAL));

    let res = emm::user_mm_uncommit(base, ALLOC_SIZE);
    assert!(res == Err(EINVAL));

    let base2 = emm::user_mm_alloc(
        None,
        ALLOC_SIZE,
        AllocFlags::COMMIT_ON_DEMAND | AllocFlags::FIXED,
        PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W,
        },
        None,
        None,
    )
    .unwrap();

    assert!(base == base2);

    let ptr = base2 as *mut u8;
    unsafe {
        ptr.write(0xFF);
        ptr.offset((ALLOC_SIZE - 1) as isize).write(0xFF);
    };

    let res = emm::user_mm_dealloc(base2, ALLOC_SIZE);
    assert!(res.is_ok());

    println!("{}", "Successfully run commit and uncommit!");
    SgxStatus::Success
}

#[no_mangle]
fn test_stack_expand() -> SgxStatus {
    const STATIC_REGION: usize = 0x8000;
    let mut buf = [0_u8; STATIC_REGION];
    for (idx, item) in buf.iter_mut().enumerate() {
        *item = (idx % 256) as u8;
    }
    for (idx, item) in buf.iter().enumerate() {
        assert!(*item == (idx % 256) as u8);
    }
    println!("{}", "Successfully expand stack!");
    SgxStatus::Success
}

#[no_mangle]
fn test_emm_alloc_dealloc() -> SgxStatus {
    let res = emm::user_mm_dealloc(0, ALLOC_SIZE);
    assert!(res == Err(EINVAL));

    let base = emm::user_mm_alloc(
        None,
        ALLOC_SIZE,
        AllocFlags::COMMIT_NOW,
        PageInfo {
            typ: PageType::Reg,
            prot: ProtFlags::R | ProtFlags::W,
        },
        None,
        None,
    )
    .unwrap();

    let res = emm::user_mm_dealloc(base, ALLOC_SIZE);
    assert!(res.is_ok());
    println!("{}", "Successfully run alloc and dealloc!");
    SgxStatus::Success
}

/// # Safety
#[no_mangle]
unsafe fn say_something(some_string: *const u8, some_len: usize) -> SgxStatus {
    let str_slice = slice::from_raw_parts(some_string, some_len);
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    let word: [u8; 4] = [82, 117, 115, 116];
    // An vector
    let word_vec: Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8").as_str();

    // Ocall to normal world for output
    println!("{}", &hello_string);

    SgxStatus::Success
}
