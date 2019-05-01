extern crate sgx_crypto_helper;
extern crate sgx_ucrypto as crypto;

use crypto::*;

static HASH_TEST_VEC: &'static [&'static str] = &[
    &"abc",
    &"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    &"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
];

static HASH_SHA256_TRUTH: &'static [&'static str] = &[
    &"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    &"248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
    &"cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1",
];

fn test_rsgx_sha256_slice() {
    let test_size = HASH_TEST_VEC.len();
    for i in 0..test_size {
        let input_str = String::from(HASH_TEST_VEC[i]);
        let hash = rsgx_sha256_slice(input_str.as_bytes()).unwrap();
        assert_eq!(hex_to_bytes(HASH_SHA256_TRUTH[i]), hash);
    }
}

fn test_rsgx_sha256_handle() {
    let test_size = HASH_TEST_VEC.len();
    for i in 0..test_size {
        let input_str = String::from(HASH_TEST_VEC[i]);
        let shah = SgxShaHandle::new();
        shah.init().unwrap();
        shah.update_slice(input_str.as_bytes()).unwrap();
        let hash = shah.get_hash().unwrap();
        shah.close().unwrap();
        assert_eq!(hex_to_bytes(HASH_SHA256_TRUTH[i]), hash);
    }
}

fn main() {
    test_rsgx_sha256_slice();
    test_rsgx_sha256_handle();
    println!("Complete!");
}
