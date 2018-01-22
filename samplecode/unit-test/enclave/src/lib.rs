// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

#![crate_name = "unittestsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_tcrypto;
#[macro_use]
extern crate sgx_tunittest;
extern crate sgx_trts;
extern crate sgx_rand;
extern crate sgx_tseal;

extern crate sgx_serialize;
pub use sgx_serialize::*;
#[macro_use]
extern crate sgx_serialize_derive;

pub use sgx_serialize::*;

use sgx_types::*;
use sgx_tunittest::*;

use std::vec::Vec;
use std::string::String;

mod utils;

mod test_crypto;
use test_crypto::*;

mod test_assert;
use test_assert::*;

pub mod test_rts;
use test_rts::*;

mod test_seal;
use test_seal::*;

mod test_rand;
use test_rand::*;

mod test_serialize;
use test_serialize::*;

mod test_file;
use test_file::*;

mod test_time;
use test_time::*;

#[no_mangle]
pub extern "C"
fn test_main_entrance() -> sgx_status_t {
    rsgx_unit_tests!(
                     // tcrypto
                     test_rsgx_sha256_slice,
                     test_rsgx_sha256_handle,
                     // assert
                     foo_panic,
                     foo_should,
                     foo_assert,
                     // rts::veh
                     test_register_first_exception_handler,
                     test_register_last_exception_handler,
                     test_register_multiple_exception_handler,
                     // rts::trts
                     test_read_rand,
                     test_data_is_within_enclave,
                     test_slice_is_within_enlave,
                     test_raw_is_within_enclave,
                     test_data_is_outside_enclave,
                     test_slice_is_outside_enclave,
                     test_raw_is_outside_enclave,
                     // rts::macros
                     test_global_ctors_object,
                     // rts::error
                     test_error,
                     // rts::libc
                     test_rts_libc_memchr,
                     test_rts_libc_memrchr,
                     // rts::memchr
                     test_rts_memchr_memchr,
                     test_rts_memchr_memrchr,
                     test_ascii,
                     // rts::c_str
                     test_cstr,
                     // tseal
                     test_seal_unseal,
                     // rand
                     test_rand_os_sgxrng,
                     test_rand_distributions,
                     test_rand_isaac_isaacrng,
                     test_rand_chacharng,
                     test_rand_reseeding,
                     // serialize
                     test_serialize_base,
                     test_serialize_struct,
                     test_serialize_enum,
                     // std::sgxfs
                     test_sgxfs,
                     // std::fs
                     test_fs,
                     // std::time
                     test_std_time
                     );
    sgx_status_t::SGX_SUCCESS
}

