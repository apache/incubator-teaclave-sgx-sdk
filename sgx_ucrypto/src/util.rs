use libc::{size_t, c_int, c_void};
use libc::{EINVAL, EOVERFLOW};
use libc::memset;
use sgx_types::sgx_status_t;
use rand_core::RngCore;
use rdrand;
use std::ptr::copy_nonoverlapping;

type errno_t = c_int;

#[no_mangle]
pub unsafe extern "C"
fn memset_s(p : *mut c_void,
            destsz: size_t,
            ch: c_int,
            count: size_t) -> errno_t {
    if p.is_null() {
        return EINVAL;
    }

    if count > destsz {
        return EOVERFLOW;
    }

    memset(p, ch, count);

    0
}

#[no_mangle]
pub extern "C"
fn sgx_read_rand(rand: *mut u8, len: size_t) -> sgx_status_t {
    if rand.is_null() || len == 0 || len > std::u32::MAX as usize {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    let mut tmp = vec![0; len];
    let mut rnd = rdrand::RdRand::new().unwrap();
    rnd.fill_bytes(&mut tmp);
    unsafe {
        copy_nonoverlapping(tmp.as_ptr(), rand, len);
    }
    sgx_status_t::SGX_SUCCESS
}

pub fn hex_to_bytes(hex_string: &str) -> Vec<u8> {
    let input_chars: Vec<_> = hex_string.chars().collect();

    input_chars.chunks(2).map(|chunk| {
        let first_byte = chunk[0].to_digit(16).unwrap();
        let second_byte = chunk[1].to_digit(16).unwrap();
        ((first_byte << 4) | second_byte) as u8
    }).collect()
}
