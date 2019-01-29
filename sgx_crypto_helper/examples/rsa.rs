extern crate sgx_crypto_helper;
extern crate sgx_types;
extern crate sgx_ucrypto as crypto;

use crypto::*;
use sgx_crypto_helper::*;
use sgx_types::*;

fn rsa_key() -> sgx_status_t {
    let text = String::from("abc");
    let text_slice = &text.into_bytes();
    let block_size = 256;

    let mod_size: i32 = 256;
    let exp_size: i32 = 4;
    let mut n: Vec<u8> = vec![0_u8; mod_size as usize];
    let mut d: Vec<u8> = vec![0_u8; mod_size as usize];
    let mut e: Vec<u8> = vec![1, 0, 0, 1];
    let mut p: Vec<u8> = vec![0_u8; mod_size as usize / 2];
    let mut q: Vec<u8> = vec![0_u8; mod_size as usize / 2];
    let mut dmp1: Vec<u8> = vec![0_u8; mod_size as usize / 2];
    let mut dmq1: Vec<u8> = vec![0_u8; mod_size as usize / 2];
    let mut iqmp: Vec<u8> = vec![0_u8; mod_size as usize / 2];

    let result = rsgx_create_rsa_key_pair(
        mod_size,
        exp_size,
        n.as_mut_slice(),
        d.as_mut_slice(),
        e.as_mut_slice(),
        p.as_mut_slice(),
        q.as_mut_slice(),
        dmp1.as_mut_slice(),
        dmq1.as_mut_slice(),
        iqmp.as_mut_slice(),
    );

    match result { Err(x) => {
            return x;
        }
        Ok(()) => {}
    }

    let privkey = SgxRsaPrivKey::new();
    let pubkey = SgxRsaPubKey::new();

    let result = pubkey.create(mod_size, exp_size, n.as_slice(), e.as_slice());
    match result {
        Err(x) => return x,
        Ok(()) => {}
    };

    let result = privkey.create(
        mod_size,
        exp_size,
        e.as_slice(),
        p.as_slice(),
        q.as_slice(),
        dmp1.as_slice(),
        dmq1.as_slice(),
        iqmp.as_slice(),
    );
    match result {
        Err(x) => return x,
        Ok(()) => {}
    };

    let mut ciphertext: Vec<u8> = vec![0_u8; block_size];
    let mut chipertext_len: usize = ciphertext.len();
    let ret = pubkey.encrypt_sha256(ciphertext.as_mut_slice(), &mut chipertext_len, text_slice);
    match ret {
        Err(x) => {
            return x;
        }
        Ok(()) => {
            println!("rsa chipertext_len: {:?}", chipertext_len);
        }
    };

    let mut plaintext: Vec<u8> = vec![0_u8; block_size];
    let mut plaintext_len: usize = plaintext.len();
    let ret = privkey.decrypt_sha256(
        plaintext.as_mut_slice(),
        &mut plaintext_len,
        ciphertext.as_slice(),
    );
    match ret {
        Err(x) => {
            return x;
        }
        Ok(()) => {
            println!("rsa plaintext_len: {:?}", plaintext_len);
        }
    };

    if plaintext[..plaintext_len] != text_slice[..] {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }

    println!(
        "Recovered plaintext: {}",
        String::from_utf8(plaintext).unwrap()
    );

    sgx_status_t::SGX_SUCCESS
}

fn rsa2048() -> sgx_status_t {
    let text = String::from("abc");
    let text_slice = &text.into_bytes();

    let k = rsa2048::Rsa2048KeyPair::new().unwrap();
    println!("Generated k = {:?}", k);

    let mut ciphertext = Vec::new();
    k.encrypt_buffer(text_slice, &mut ciphertext).unwrap();
    let mut decrypted_vec = Vec::new();
    k.decrypt_buffer(&ciphertext, &mut decrypted_vec).unwrap();

    println!(
        "Recovered plaintext: {}",
        String::from_utf8(decrypted_vec).unwrap()
    );

    sgx_status_t::SGX_SUCCESS
}

fn rsa3072() -> sgx_status_t {
    let text = String::from("abc");
    let text_slice = &text.into_bytes();

    let k = rsa3072::Rsa3072KeyPair::new().unwrap();
    println!("Generated k = {:?}", k);

    let mut ciphertext = Vec::new();
    k.encrypt_buffer(text_slice, &mut ciphertext).unwrap();
    let mut decrypted_vec = Vec::new();
    k.decrypt_buffer(&ciphertext, &mut decrypted_vec).unwrap();

    println!(
        "Recovered plaintext: {}",
        String::from_utf8(decrypted_vec).unwrap()
    );

    sgx_status_t::SGX_SUCCESS
}
fn main() {
    rsa_key();
    rsa2048();
    rsa3072();
}
