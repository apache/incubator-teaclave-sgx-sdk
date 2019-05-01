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

use utils::*;
use std::string::String;
use sgx_tcrypto::*;

static HASH_TEST_VEC: &'static [&'static str] = &[
    &"abc",
    &"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    &"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
];

static HASH_SHA256_TRUTH: &'static [&'static str] = &[
    &"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    &"248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
    &"cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1",
];

pub fn test_rsgx_sha256_slice() {
    let test_size = HASH_TEST_VEC.len();
    for i in 0..test_size {
        let input_str = String::from(HASH_TEST_VEC[i]);
        let hash = rsgx_sha256_slice(input_str.as_bytes()).unwrap();
        assert_eq!(hex_to_bytes(HASH_SHA256_TRUTH[i]), hash);
    }
}

pub fn test_rsgx_sha256_handle(){
    let test_size = HASH_TEST_VEC.len();
    for i in 0..test_size {
        let input_str = String::from(HASH_TEST_VEC[i]);
        let shah = SgxShaHandle::new();
        shah.init().unwrap();
        shah.update_slice(input_str.as_bytes()).unwrap();
        let hash = shah.get_hash().unwrap();
        shah.close().unwrap();
        assert_eq!(hex_to_bytes(HASH_SHA256_TRUTH[i]), hash);
    }
}

