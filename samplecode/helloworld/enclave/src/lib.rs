#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![no_std]
#![feature(collections)]

#[macro_use]
extern crate collections;

extern crate sgx_types;

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

    sgx_status_t::SGX_SUCCESS
}
