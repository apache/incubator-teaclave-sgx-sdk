#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate base64;

use base64::*;

mod utils;

fuzz_target!(|data: &[u8]| {
    let config = utils::random_config(data);

    let encoded = encode_config(&data, config);
    let decoded = decode_config(&encoded, config).unwrap();
    assert_eq!(data, decoded.as_slice());
});
