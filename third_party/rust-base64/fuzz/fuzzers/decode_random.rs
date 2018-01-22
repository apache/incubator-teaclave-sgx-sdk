#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate base64;

use base64::*;

mod utils;

fuzz_target!(|data: &[u8]| {
    let config = utils::random_config(data);

    // The data probably isn't valid base64 input, but as long as it returns an error instead
    // of crashing, that's correct behavior.
    let _ = decode_config(&data, config);
});
