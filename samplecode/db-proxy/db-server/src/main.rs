extern crate hex;
extern crate merklebtree;
extern crate parking_lot;
extern crate ring;
extern crate rocksdb;
extern crate serde;
extern crate serde_json;

mod client;
mod server;
mod verifytree;

use parking_lot::RwLock;
use rocksdb::{DBVector, WriteBatch, DB};
use server::new_server;
use std::fs;
use std::io::{Read, Write};
use std::path;

fn main() {
    vertias_db();
}

pub fn vertias_db() {
    // generate the key_value of hmac key
    let mut key_value = Vec::new();
    for i in 0..32 {
        key_value.push(i as u8)
    }

    let mut server = new_server(key_value);
    client::client_test(&mut server);
}
