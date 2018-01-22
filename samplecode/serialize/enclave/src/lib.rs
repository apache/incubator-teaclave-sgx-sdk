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

#![crate_name = "serializesampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
//#![feature(untagged_unions)]

/// Must use these extern for auto deriving
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

extern crate sgx_serialize;
use sgx_serialize::{SerializeHelper, DeSerializeHelper};
#[macro_use]
extern crate sgx_serialize_derive;

#[derive(Serializable, DeSerializable)]
struct TestStructUnit;

#[derive(Serializable, DeSerializable)]
struct TestStructNewType(i32);

#[derive(Serializable, DeSerializable)]
struct TestStructTuple(i32, i32);

#[derive(Serializable, DeSerializable)]
struct TestSturct {
    a1: u32,
    a2: u32,
}

#[derive(Serializable, DeSerializable)]
enum TestEnum {
    EnumUnit,
    EnumNewType(u32),
    EnumTuple(u32, u32),
    EnumStruct{a1:i32, a2:i32},
    EnumSubStruct(TestSturct),
}

#[no_mangle]
pub extern "C" fn serialize() {

    //let a = TestSturct {a1: 2017u32, a2: 829u32};

    //let a = TestEnum::EnumSubStruct(a);
    //let a = TestEnum::EnumTuple(2017, 829);
    //let a = TestEnum::EnumNewType(2017);
    let a = TestEnum::EnumStruct {a1: 2017, a2:829};
    // new a SerializeHelper
    let helper = SerializeHelper::new();
    // encode data
    let data = helper.encode(a).unwrap();
    // decode data
    let helper = DeSerializeHelper::<TestEnum>::new(data);
    let c = helper.decode().unwrap();

    // // for TestSturct
    // // println!("decode data: {}, {}", c.a1, c.a2);

    // // for TestEnum
    match c {
        TestEnum::EnumStruct {ref a1, ref a2} => {
            println!("decode data: {}, {}", a1, a2);
        },
        TestEnum::EnumNewType(ref a) => {
            println!("decode data: {}", a);
        },
        TestEnum::EnumTuple(ref a1, ref a2) => {
            println!("decode data: {}, {}", a1, a2);
        },
        TestEnum::EnumSubStruct(ref a) => {
            println!("decode data: {}, {}", a.a1, a.a2);
        },
        _ => {}
    }
}
