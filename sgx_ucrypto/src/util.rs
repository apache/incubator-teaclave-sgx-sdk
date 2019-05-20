use libc::{size_t, c_int, c_void};
use libc::{EINVAL, E2BIG, EOVERFLOW};
use libc::memset;
use sgx_types::sgx_status_t;
use rand_core::RngCore;
use rdrand;
use std::ptr::copy_nonoverlapping;

type errno_t = c_int;
const SIZE_MAX: size_t = std::usize::MAX;

// memset_s was defined in ISO C11
// ISO/IEC 9899:2011 section K.3.7.4.1 The memset_s function
// It returns:
// [EINVAL]           The s argument was a null pointer.
// [E2BIG]            One or both of smax or n was larger than RSIZE_MAX.
// [EOVERFLOW]        n was larger than smax.
//
// In Linux-SGX environment, RSIZE_MAX is chosen to be -1ULL.
// So the comparison is always false -- but I think keeping this code
// here as well as these comments is better.
#[allow(clippy::absurd_extreme_comparisons)]
#[no_mangle]
pub extern "C"
fn memset_s(p : *mut c_void,
            destsz: size_t,
            ch: c_int,
            count: size_t) -> errno_t {
    if p.is_null() {
        return EINVAL;
    }

    if destsz > SIZE_MAX || count > SIZE_MAX {
        return E2BIG;
    }

    if count > destsz {
        return EOVERFLOW;
    }

    unsafe { memset(p, ch, count); }

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
    unsafe { copy_nonoverlapping(tmp.as_ptr(), rand, len); }
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
