#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

extern crate bigint;
extern crate rlp;
extern crate secp256k1;
extern crate sha3;
extern crate block_core;

mod transaction;
mod address;

pub use block_core::*;
pub use transaction::*;
pub use address::FromKey;
pub use rlp::*;

use bigint::H256;

pub trait RlpHash {
    fn rlp_hash(&self) -> H256;
}
