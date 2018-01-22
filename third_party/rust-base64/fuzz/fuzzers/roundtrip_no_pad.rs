#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate base64;

fuzz_target!(|data: &[u8]| {
    let config = base64::Config::new(base64::CharacterSet::Standard, false, false, base64::LineWrap::NoWrap);

    let encoded = base64::encode_config(&data, config);
    let decoded = base64::decode_config(&encoded, config).unwrap();
    assert_eq!(data, decoded.as_slice());
});
