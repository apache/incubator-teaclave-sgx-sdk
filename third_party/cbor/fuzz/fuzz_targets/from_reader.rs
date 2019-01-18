#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate serde_cbor;

use serde_cbor::Value;

fuzz_target!(|data: &[u8]| {
    let mut data = data;
    let _ = serde_cbor::from_reader::<Value, _>(&mut data);
});
