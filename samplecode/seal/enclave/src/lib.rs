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

#[cfg(not(target_vendor = "teaclave"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_trts;
extern crate sgx_types;

use sgx_serialize::opaque;
use sgx_serialize::{Deserialize, Serialize};
use sgx_trts::rand::Rng;
use sgx_tseal::seal::*;
use sgx_types::error::{SgxResult, SgxStatus};
use std::string::{String, ToString};
use std::untrusted::fs;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SealData {
    key: u32,
    msg: String,
    data: Vec<u8>,
}

impl SealData {
    pub fn new(msg: String, data: Vec<u8>) -> Self {
        let mut rng = Rng::new();
        let key = rng.next_u32();
        Self { key, msg, data }
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn seal_data() -> SgxStatus {
    let aad = "aad mac text".to_string();
    let msg = "Data to encrypt".to_string();
    let mut data = vec![0_u8; 128];

    let mut rng = Rng::new();
    rng.fill_bytes(data.as_mut());

    let data = SealData::new(msg, data);
    println!("sealdata: {data:?}");

    let sealed_bytes = match seal(data.clone(), aad.clone()) {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("seal failed. {e:?}");
            return e;
        }
    };

    let _ = fs::write("sealed_data.txt", sealed_bytes);
    let sealed_bytes = match fs::read("sealed_data.txt") {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("read sealed_data.txt failed. {e:?}");
            return SgxStatus::Unexpected;
        }
    };
    let _ = fs::remove_file("sealed_data.txt");

    let (unsealed_data, unsealed_aad) = match unseal(sealed_bytes) {
        Ok(data) => data,
        Err(e) => {
            println!("unseal failed. {e:?}");
            return e;
        }
    };

    println!("unsealed data: {unsealed_data:?}");
    println!("aad: {unsealed_aad:?}");

    assert_eq!(data, unsealed_data);
    assert_eq!(aad, unsealed_aad);

    SgxStatus::Success
}

fn seal(data: SealData, aad: String) -> SgxResult<Vec<u8>> {
    let bytes = opaque::encode(&data).unwrap();

    let sealed_data = SealedData::<[u8]>::seal(bytes.as_slice(), Some(aad.as_bytes()))?;
    sealed_data.into_bytes()
}

fn unseal(bytes: Vec<u8>) -> SgxResult<(SealData, String)> {
    let unsealed_bytes = UnsealedData::<[u8]>::unseal_from_bytes(bytes)?;
    let seal_data = opaque::decode(unsealed_bytes.to_plaintext()).ok_or(SgxStatus::Unexpected)?;

    let aad =
        String::from_utf8(unsealed_bytes.to_aad().to_vec()).map_err(|_| SgxStatus::Unexpected)?;
    Ok((seal_data, aad))
}
