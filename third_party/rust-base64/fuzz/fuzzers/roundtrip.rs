#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate base64;

fuzz_target!(|data: &[u8]| {
    let encoded = base64::encode_config(&data, base64::STANDARD);
    let decoded = base64::decode_config(&encoded, base64::STANDARD).unwrap();
    assert_eq!(data, decoded.as_slice());
});
