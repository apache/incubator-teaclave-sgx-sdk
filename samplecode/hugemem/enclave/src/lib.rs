#![crate_name = "hugememsampleenclave"]
#![crate_type = "staticlib"]

#![no_std]
#![feature(collections)]

#[macro_use]
extern crate collections;

extern crate sgx_types;
extern crate sgx_trts;

use sgx_types::*;
use collections::string::String;
use collections::vec::Vec;

/// The Ocall declared in Enclave.edl and implemented in app.c
///
/// # Parameters
///
/// **str**
///
/// A pointer to the string to be printed
///
/// **len**
///
/// An unsigned int indicates the length of str
///
/// # Return value
///
/// None
extern "C" {
    fn ocall_print_string(str: *const c_uchar, len: size_t);
}

/// A function simply invokes ocall print to print the incoming string
///
/// # Parameters
///
/// **some_string**
///
/// A pointer to the string to be printed
///
/// **len**
///
/// An unsigned int indicates the length of str
///
/// # Return value
///
/// Always returns SGX_SUCCESS
#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: u32) -> sgx_status_t {
    unsafe {
        ocall_print_string(some_string as *const c_uchar, some_len as size_t);
    }

    // A sample &'static string
    let rust_raw_string = "This is a ";
    // An array
    let word:[u8;4] = [82, 117, 115, 116];
    // An vector
    let word_vec:Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8")
                                               .as_str();

    // Ocall to normal world for output
    unsafe {
        ocall_print_string(hello_string.as_ptr() as *const c_uchar,
                           hello_string.len() as size_t);
    }

    let mut sum:u64 = 0;
    let mut vv:Vec<Vec<u8>> = Vec::new();
    let mut onev:Vec<u8>; // 1Mbyte
    let testblocksize:usize = 128 * 1024 * 1024; // 128Mbyte
    let total = 0x7E8000000 ; // 31.625 GB = rounddown (31.75 GB - essential costs)

    for i in 0..total / testblocksize{
        onev = Vec::with_capacity(testblocksize); // 128Mbyte
        for j in 0..testblocksize {
            onev.push(((j as u32) % 256) as u8);
        }
        vv.push(onev);
        sum += testblocksize as u64;
        let outstr = format!("{}th allocate {} vec sum = {} bytes", i, testblocksize, sum);
        unsafe {
            ocall_print_string(outstr.as_ptr() as *const c_uchar,
                               outstr.len() as size_t);
        }
    }

    hello_string = String::from("Checking for values in allocated memory");

    unsafe {
        ocall_print_string(hello_string.as_ptr() as *const c_uchar,
                           hello_string.len() as size_t);
    }
    
    for i in 0..total / testblocksize{
        for j in 0..testblocksize {
            if vv[i][j] != ((j as u32) % 256) as u8 {
                return sgx_status_t::SGX_ERROR_UNEXPECTED;
            }
        }
    }

    hello_string = String::from("All check success!");
    unsafe {
        ocall_print_string(hello_string.as_ptr() as *const c_uchar,
                           hello_string.len() as size_t);
    }

    sgx_status_t::SGX_SUCCESS
}
