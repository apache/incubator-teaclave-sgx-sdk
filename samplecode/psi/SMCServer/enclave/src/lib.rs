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

#![crate_name = "psienclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![feature(asm)]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate sgx_types;
extern crate sgx_trts;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_tdh;
extern crate sgx_tcrypto;
extern crate sgx_tkey_exchange;
extern crate sgx_rand;

use sgx_types::*;
use sgx_trts::memeq::ConsttimeMemEq;
use sgx_tcrypto::*;
use sgx_tkey_exchange::*;
use sgx_rand::{Rng, StdRng};
use std::slice;
use std::vec::Vec;
use std::cell::RefCell;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::boxed::Box;

const G_SP_PUB_KEY: sgx_ec256_public_t = sgx_ec256_public_t {
    gx : [0x72, 0x12, 0x8a, 0x7a, 0x17, 0x52, 0x6e, 0xbf,
          0x85, 0xd0, 0x3a, 0x62, 0x37, 0x30, 0xae, 0xad,
          0x3e, 0x3d, 0xaa, 0xee, 0x9c, 0x60, 0x73, 0x1d,
          0xb0, 0x5b, 0xe8, 0x62, 0x1c, 0x4b, 0xeb, 0x38],
    gy : [0xd4, 0x81, 0x40, 0xd9, 0x50, 0xe2, 0x57, 0x7b,
          0x26, 0xee, 0xb7, 0x41, 0xe7, 0xc6, 0x14, 0xe2,
          0x24, 0xb7, 0xbd, 0xc9, 0x03, 0xf2, 0x9a, 0x28,
          0xa8, 0x3c, 0xc8, 0x10, 0x11, 0x14, 0x5e, 0x06]
};

const SGX_SALT_SIZE: usize = 32;
const CLIENT_MAX_NUMBER: usize = 2;
const HASH_DATA_FINISH: u32 = 1;
const RESULT_FINISH: u32 = 2;

#[derive(Clone, Default)]
struct SetIntersection {
    salt: [u8; SGX_SALT_SIZE],
    data: [HashDataBuffer; CLIENT_MAX_NUMBER],
    number: u32,
}

#[derive(Clone, Default)]
struct HashDataBuffer {
    hashdata: Vec<[u8; SGX_HASH_SIZE]>,
    result: Vec<u8>,
    state: u32,
}

impl SetIntersection {
    pub fn new() -> Self {
        SetIntersection::default()
    }
}

static GLOBAL_HASH_BUFFER: AtomicPtr<()> = AtomicPtr::new(0 as * mut ());

fn get_ref_hash_buffer() -> Option<&'static RefCell<SetIntersection>>
{
    let ptr = GLOBAL_HASH_BUFFER.load(Ordering::SeqCst) as * mut RefCell<SetIntersection>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &* ptr })
    }
}


#[no_mangle]
pub extern "C"
fn initialize() -> sgx_status_t {

    let mut data = SetIntersection::new();
    let mut rand = match StdRng::new() {
        Ok(rng) => rng,
        Err(_) => { return sgx_status_t::SGX_ERROR_UNEXPECTED; },
    };
    rand.fill_bytes(&mut data.salt);

    let data_box = Box::new(RefCell::<SetIntersection>::new(data));
    let ptr = Box::into_raw(data_box);
    GLOBAL_HASH_BUFFER.store(ptr as *mut (), Ordering::SeqCst);

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn uninitialize() {

    let ptr = GLOBAL_HASH_BUFFER.swap(0 as * mut (), Ordering::SeqCst) as * mut RefCell<SetIntersection>;
    if ptr.is_null() {
       return;
    }
    let _ = unsafe { Box::from_raw(ptr) };
}


#[no_mangle]
pub extern "C"
fn enclave_init_ra(b_pse: i32, p_context: &mut sgx_ra_context_t) -> sgx_status_t {
    let mut ret: sgx_status_t = sgx_status_t::SGX_SUCCESS;
    match rsgx_ra_init(&G_SP_PUB_KEY, b_pse) {
        Ok(p) => {
            *p_context = p;
            ret = sgx_status_t::SGX_SUCCESS;
        },
        Err(x) => {
            ret = x;
            return ret;
        }
    }
    ret
}

#[no_mangle]
pub extern "C"
fn enclave_ra_close(context: sgx_ra_context_t) -> sgx_status_t {
    match rsgx_ra_close(context) {
        Ok(()) => sgx_status_t::SGX_SUCCESS,
        Err(x) => x
    }
}

#[no_mangle]
pub extern "C"
fn verify_att_result_mac(context: sgx_ra_context_t,
                         message: * const u8,
                         msg_size: size_t,
                         mac: &[u8; SGX_MAC_SIZE]) -> sgx_status_t {

    if msg_size > u32::max_value as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let message_slice = unsafe {
        slice::from_raw_parts(message, msg_size as usize)
    };
    if message_slice.len() != msg_size as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let mk_key: sgx_ec_key_128bit_t = match rsgx_ra_get_keys(context, sgx_ra_key_type_t::SGX_RA_KEY_MK) {
        Ok(k) => k,
        Err(x) => return x,
    };

    let mac_result: sgx_cmac_128bit_tag_t = match rsgx_rijndael128_cmac_slice(&mk_key, message_slice) {
        Ok(tag) => tag,
        Err(x) => return x,
    };

    if mac.consttime_memeq(&mac_result) == false {
        return sgx_status_t::SGX_ERROR_MAC_MISMATCH;
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn verify_secret_data(context: sgx_ra_context_t,
                      secret: * const u8,
                      sec_size: u32,
                      gcm_mac: &[u8; SGX_MAC_SIZE],
                      max_vlen: u32,
                      salt: &mut [u8; SGX_SALT_SIZE],
                      salt_mac: &mut [u8; SGX_MAC_SIZE],
                      id: &mut u32) -> sgx_status_t {

    let mut data = get_ref_hash_buffer().unwrap().borrow_mut();
    if data.number < CLIENT_MAX_NUMBER as u32 {
        data.number +=1;
    } else {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }

    let sk_key: sgx_ec_key_128bit_t = match rsgx_ra_get_keys(context, sgx_ra_key_type_t::SGX_RA_KEY_SK) {
        Ok(key) => key,
        Err(x) => return x,
    };

    let secret_slice = unsafe {
        slice::from_raw_parts(secret, sec_size as usize)
    };

    if secret_slice.len() != sec_size as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let mut decrypted_vec: Vec<u8> = vec![0; sec_size as usize];
    let decrypted_slice = &mut decrypted_vec[..];
    let iv = [0; SGX_AESGCM_IV_SIZE];
    let aad:[u8; 0] = [0; 0];

    let ret = rsgx_rijndael128GCM_decrypt(&sk_key,
                                          &secret_slice,
                                          &iv,
                                          &aad,
                                          gcm_mac,
                                          decrypted_slice);
    match ret {
        Ok(()) => {
            if decrypted_slice[0] == 0 && decrypted_slice[1] != 1 {
                return sgx_status_t::SGX_ERROR_INVALID_SIGNATURE;
            }
            else {
                let ret = rsgx_rijndael128GCM_encrypt(&sk_key,
                                                      &data.salt,
                                                      &iv,
                                                      &aad,
                                                      salt,
                                                      salt_mac);
                match ret {
                    Ok(()) => {
                        *id = data.number;
                        return sgx_status_t::SGX_SUCCESS;
                    },
                    Err(_) => { return sgx_status_t::SGX_ERROR_UNEXPECTED; },
                }
            }
        },
        Err(_) => {
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    }
}


#[no_mangle]
pub extern "C"
fn add_hash_data(id: u32,
                 context: sgx_ra_context_t,
                 hashdata: * const u8,
                 hash_size: usize,
                 mac: &[u8; SGX_MAC_SIZE]) -> sgx_status_t {

    if (id == 0) || (id > CLIENT_MAX_NUMBER as u32) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let sk_key: sgx_ec_key_128bit_t = match rsgx_ra_get_keys(context, sgx_ra_key_type_t::SGX_RA_KEY_SK) {
        Ok(key) => key,
        Err(x) => return x
    };

    let hash_slice = unsafe {
        slice::from_raw_parts(hashdata, hash_size as usize)
    };

    if hash_slice.len() != hash_size {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let mut decrypted: Vec<u8> = vec![0; hash_size];
    let iv = [0; SGX_AESGCM_IV_SIZE];
    let aad:[u8; 0] = [0; 0];

    let ret = rsgx_rijndael128GCM_decrypt(&sk_key,
                                          &hash_slice,
                                          &iv,
                                          &aad,
                                          mac,
                                          decrypted.as_mut_slice());

    match ret {
        Ok(()) => {},
        Err(x) => return x,
    };

    let mut intersection = get_ref_hash_buffer().unwrap().borrow_mut();
    let buffer = &mut intersection.data[id as usize - 1].hashdata;

    for i in 0_usize..(hash_size/SGX_HASH_SIZE) {
        let mut hash = [0_u8; SGX_HASH_SIZE];
        hash.copy_from_slice(&decrypted[i*SGX_HASH_SIZE..(i + 1)*SGX_HASH_SIZE]);
        buffer.push(hash);
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn get_result_size(id: u32, len: &mut usize) -> sgx_status_t {

    if (id == 0) || (id > CLIENT_MAX_NUMBER as u32) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let cid: usize = id as usize - 1;
    let other = if cid == 0 {
        CLIENT_MAX_NUMBER - 1
    } else {
        0
    };

    let mut intersection = get_ref_hash_buffer().unwrap().borrow_mut();

    if intersection.data[cid].state == 0 {
        intersection.data[cid].state = HASH_DATA_FINISH;
    }

    let state1 = intersection.data[cid].state;
    let state2 = intersection.data[other].state;

    let result_len = if (state1 == 0) || (state2 == 0) {
        return sgx_status_t::SGX_ERROR_INVALID_STATE;
    } else if (state1 == HASH_DATA_FINISH) && (state2 == HASH_DATA_FINISH) {

        let mut v_cid: Vec<u8> = vec![0; intersection.data[cid].hashdata.len()];
        let mut v_other: Vec<u8> = vec![0; intersection.data[other].hashdata.len()];

        oget_intersection(&intersection.data[cid].hashdata,
                          &intersection.data[other].hashdata,
                          &mut v_cid,
                          &mut v_other);

        intersection.data[cid].result = v_cid;
        intersection.data[other].result = v_other;
        intersection.data[cid].state = RESULT_FINISH;
        intersection.data[other].state = RESULT_FINISH;
        intersection.data[cid].result.len()
    } else if (state1 == RESULT_FINISH) && (state2 == RESULT_FINISH) {
        intersection.data[cid].result.len()
    } else {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    *len = result_len;
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn get_result(id: u32,
              context: sgx_ra_context_t,
              result: * mut u8,
              result_size: usize,
              result_mac: &mut [u8; SGX_MAC_SIZE]) -> sgx_status_t {

    if (id == 0) || (id > CLIENT_MAX_NUMBER as u32) {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let cid: usize = id as usize - 1;
    let other = if cid == 0 {
        CLIENT_MAX_NUMBER - 1
    } else {
        0
    };

    let sk_key: sgx_ec_key_128bit_t = match rsgx_ra_get_keys(context, sgx_ra_key_type_t::SGX_RA_KEY_SK) {
        Ok(key) => key,
        Err(x) => return x,
    };

    let mut intersection = get_ref_hash_buffer().unwrap().borrow_mut();

    let state1 = intersection.data[cid].state;
    let state2 = intersection.data[other].state;
    if (state1 != RESULT_FINISH) && (state2 != RESULT_FINISH) {
        return sgx_status_t::SGX_ERROR_INVALID_STATE;
    }

    let len = intersection.data[cid].result.len();
    if len > 0 {
        if result_size != len {
            return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
        }

        let result_slice = unsafe {
            slice::from_raw_parts_mut(result, result_size)
        };

        let iv = [0; SGX_AESGCM_IV_SIZE];
        let aad:[u8; 0] = [0; 0];
        let ret = rsgx_rijndael128GCM_encrypt(&sk_key,
                                              intersection.data[cid].result.as_slice(),
                                              &iv,
                                              &aad,
                                              result_slice,
                                              result_mac);
        match ret {
            Ok(()) => {},
            Err(x) => return x,
        };
    }

    intersection.number -= 1;
    if intersection.number == 0 {
        for i in 0..CLIENT_MAX_NUMBER {
            intersection.data[i].hashdata = Vec::new();
            intersection.data[i].result = Vec::new();
            intersection.data[i].state = 0;
        }
    }

    sgx_status_t::SGX_SUCCESS
}

fn oget_intersection(a: &Vec<[u8; SGX_HASH_SIZE]>, b: &Vec<[u8; SGX_HASH_SIZE]>, v1: &mut Vec<u8>, v2: &mut Vec<u8>) {

    let n = a.len();
    for i in 0..n {
        let ret = obinary_search(b, &a[i], v2);
        let miss = oequal(usize::max_value(), ret as usize);
        v1[i] = omov(miss as isize, 0, 1) as u8;
    }
}

fn obinary_search(b: &Vec<[u8; SGX_HASH_SIZE]>, target: &[u8; SGX_HASH_SIZE], v2: &mut Vec<u8>) -> isize {

    let mut lo: isize = 0;
    let mut hi: isize = b.len() as isize - 1;
    let mut ret: isize = -1;

    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let hit = eq(&b[mid as usize], target);
        ret = omov(hit, mid, ret);
        v2[mid as usize] = omov(hit, 1, v2[mid as usize] as isize) as u8;
        let be = le(&b[mid as usize], target);
        lo = omov(be, mid + 1, lo);
        hi = omov(be, hi, mid - 1);
    }
    ret
}

fn eq(a: &[u8; SGX_HASH_SIZE], b: &[u8; SGX_HASH_SIZE]) -> isize {

    let mut ret: isize = 1;
    for i in 0..SGX_HASH_SIZE {
        let hit = oequal(a[i] as usize, b[i] as usize);
        ret = omov(hit as isize, ret, 0);
    }
    ret
}

fn le(a: &[u8; SGX_HASH_SIZE], b: &[u8; SGX_HASH_SIZE]) -> isize {

    let mut ret: isize = 0;
    for i in 0..SGX_HASH_SIZE {

        let hit = oequal(a[i] as usize, b[i] as usize);
        let be = ob(a[i] as usize, b[i] as usize);
        let cmp = omov(hit as isize, 0, omov(be as isize, -1, 1));
        ret = omov(ret, ret, cmp)
    }
    (ret <= 0) as isize
}

fn ge(a: &[u8; SGX_HASH_SIZE], b: &[u8; SGX_HASH_SIZE]) -> isize {

    let mut ret: isize = 0;
    for i in 0..SGX_HASH_SIZE {

        let hit = oequal(a[i] as usize, b[i] as usize);
        let ae = oa(a[i] as usize, b[i] as usize);
        let cmp = omov(hit as isize, 0, omov(ae as isize, 1, -1));
        ret = omov(ret, ret, cmp)
    }
    (ret >= 0) as isize
}

fn oequal(x: usize, y: usize) -> bool {

    let ret: bool;
    unsafe {
        asm!(
            "cmp %rcx, %rdx \n\t
             sete %al \n\t"
            : "={al}"(ret)
            : "{rcx}"(x), "{rdx}" (y)
            : "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}

fn obe(x: usize, y: usize) -> bool {

    let ret: bool;
    unsafe {
        asm!(
            "cmp %rdx, %rcx \n\t
             setbe %al \n\t"
            : "={al}"(ret)
            : "{rcx}"(x), "{rdx}" (y)
            : "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}

fn ob(x: usize, y: usize) -> bool {

    let ret: bool;
    unsafe {
        asm!(
            "cmp %rdx, %rcx \n\t
             setb %al \n\t"
            : "={al}"(ret)
            : "{rcx}"(x), "{rdx}" (y)
            : "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}

fn oae(x: usize, y: usize) -> bool {

    let ret: bool;
    unsafe {
        asm!(
            "cmp %rdx, %rcx \n\t
             setae %al \n\t"
            : "={al}"(ret)
            : "{rcx}"(x), "{rdx}" (y)
            : "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}

fn oa(x: usize, y: usize) -> bool {

    let ret: bool;
    unsafe {
        asm!(
            "cmp %rdx, %rcx \n\t
             seta %al \n\t"
            : "={al}"(ret)
            : "{rcx}"(x), "{rdx}" (y)
            : "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}

fn omov(flag: isize, x: isize, y: isize) -> isize {

    let ret: isize;
    unsafe {
        asm!(
            "xor %rcx, %rcx \n\t
             mov $1, %rcx \n\t
             test %rcx, %rcx \n\t
             cmovz %rdx, %rax \n\t"
            : "={rax}"(ret)
            : "r"(flag), "{rax}" (x), "{rdx}" (y)
            : "rax", "rcx", "rdx"
            : "volatile"
        );
    }
    ret
}
