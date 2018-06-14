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

#![crate_name = "filesampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_rand;
extern crate sgx_types;
#[macro_use]
extern crate sgx_rand_derive;
extern crate sgx_serialize;
#[macro_use]
extern crate sgx_serialize_derive;
extern crate secp256k1;
extern crate bigint;
extern crate block_transaction as block;
extern crate hex;

use bigint::{Address, H160, U256, Gas};
use block::{FromKey, SignaturePatch, UnsignedTransaction, TransactionAction, RlpStream, Encodable};
use secp256k1::*;

use std::sgxfs::SgxFile;
use std::io::{self, Read, Write};
use std::slice;
use std::string::{String, ToString};
use std::vec::Vec;
use sgx_types::*;

use sgx_serialize::{SerializeHelper, DeSerializeHelper};

#[derive(Copy, Clone, Default, Debug, Serializable, DeSerializable, Rand)]
struct EthereumSecretWallet {
    signing_key: [u8; 32],
}

const CHAIN_ID: u64 = 57777;

pub struct MyChainPatch;

impl SignaturePatch for MyChainPatch {
    fn chain_id() -> Option<u64> { Some(CHAIN_ID) }
}

#[no_mangle]
pub extern "C" fn write_file() -> sgx_status_t {
    let wallet = sgx_rand::random::<EthereumSecretWallet>();

    let helper = SerializeHelper::new();
    let data = match helper.encode(wallet) {
        Some(d) => d,
        None => {
            println!("encode data failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let mut file = match SgxFile::create("sgx_file") {
        Ok(f) => f,
        Err(_) => {
            println!("SgxFile::create failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let write_size = match file.write(data.as_slice()) {
        Ok(len) => len,
        Err(_) => {
            println!("SgxFile::write failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    println!("write file success, write size: {}.", write_size);
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn read_file() -> sgx_status_t {
    let mut data = [0_u8; 100];

    let mut file = match SgxFile::open("sgx_file") {
        Ok(f) => f,
        Err(_) => {
            println!("SgxFile::open failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let read_size = match file.read(&mut data) {
        Ok(len) => len,
        Err(_) => {
            println!("SgxFile::read failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let helper = DeSerializeHelper::<EthereumSecretWallet>::new(data.to_vec());
    let wallet = match helper.decode() {
        Some(d) => d,
        None => {
            println!("decode data failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let secret_key = SecretKey::parse(&wallet.signing_key).unwrap();
    let address = Address::from_secret_key(&secret_key).unwrap();

    println!("read file success, read size: {}.", read_size);
    println!("Your SGX ethereum wallet is: 0x{:02x?}", address);
    sgx_status_t::SGX_SUCCESS
}


#[no_mangle]
pub extern "C" fn sign(some_string: *const u8, some_len: usize) -> sgx_status_t {
    let mut data = [0_u8; 100];

    let mut file = match SgxFile::open("sgx_file") {
        Ok(f) => f,
        Err(_) => {
            println!("SgxFile::open failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let read_size = match file.read(&mut data) {
        Ok(len) => len,
        Err(_) => {
            println!("SgxFile::read failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let helper = DeSerializeHelper::<EthereumSecretWallet>::new(data.to_vec());
    let wallet = match helper.decode() {
        Some(d) => d,
        None => {
            println!("decode data failed.");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

//    Ganache dev
//    let ganache_key: [u8; 32] = [0xce, 0x1f, 0x60, 0xd5, 0x97, 0xa2, 0xac, 0xf1, 0x74, 0x3f, 0x84, 0x65, 0xcd, 0x92, 0x63, 0xc1, 0x10, 0xb1, 0xb9, 0x89, 0x4e, 0x7b, 0xe9, 0xab, 0x09, 0x7d, 0xfe, 0x45, 0x84, 0x82, 0xd7, 0x03];
//    let secret_key = SecretKey::parse(&ganache_key).unwrap();

    //from sealed data
    let secret_key = SecretKey::parse(&wallet.signing_key).unwrap();

    let address = Address::from_secret_key(&secret_key).unwrap();

    println!("read file success, read size: {}.", read_size);
    println!("Your SGX ethereum wallet is: 0x{:02x?}", address);

    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

//    println!("{:?}", str_slice);
    let string: String = String::from_utf8(str_slice.to_vec()).unwrap();
    let split = string.split("|");

    // NONCE|gasprice|gaslimit|[address_smart_contract or creation or destination]|value|[hex string data, only when calling a smart contract or sending value, otherwise empty]
    let vec: Vec<&str> = split.collect();
    let _nonce: u64 = vec[0].parse::<u64>().unwrap();
    let _gas_price: u64 = vec[1].parse::<u64>().unwrap();
    let _gas_limit: u64 = vec[2].parse::<u64>().unwrap();
    let mut destination: String = vec[3].parse::<String>().unwrap();
    let _value: u64 = vec[4].parse::<u64>().unwrap();
    let mut hex_data: String = vec[5].parse::<String>().unwrap();

    if hex_data.len() > 2 && hex_data[0..2] == "0x".to_string() {
        hex_data = hex_data[2..].to_string();
    }
    let _input: Vec<u8> = hex::decode(hex_data).unwrap();

    let mut _action = TransactionAction::Create;

    if destination.len() > 2 {
        if destination.starts_with("0x") {
            destination = destination[2..].to_string();
        }
        let mut _dest: Vec<u8> = hex::decode(destination).unwrap();
        let mut _address: H160 = Address::new();
        if _dest.len() > 1 {
            _address = Address::from(_dest.as_slice());
            _action = TransactionAction::Call(_address);
        }
    }

    let unsigned = UnsignedTransaction {
        nonce: U256::from(_nonce),
        gas_price: Gas::from(_gas_price),
        gas_limit: Gas::from(_gas_limit),
        action: _action,
        value: U256::from(_value),
        input: _input,
    };

    let signed = unsigned.sign::<MyChainPatch>(&secret_key);

    let mut stream = RlpStream::new();
    signed.rlp_append(&mut stream);
    let rlp_out = stream.as_raw();

//    assert_eq!(signed.signature.chain_id(), Some(CHAIN_ID));
//    assert_eq!(signed.caller().unwrap(), address);

    println!("CHAIN_ID {}, caller {}", CHAIN_ID, signed.caller().unwrap());
    println!("Signed transaction: \n{}", hex::encode(rlp_out));
    // TODO: pass the rlp_out to the outside world.
    sgx_status_t::SGX_SUCCESS
}
