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
