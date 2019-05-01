// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use sgx_types::*;
use std::vec::Vec;
use std::string::String;

use sgx_trts::trts::*;
use sgx_trts::veh::*;
use sgx_trts::ascii::AsciiExt;
use sgx_trts::c_str::*;
use sgx_trts::error;
use sgx_trts::memchr;
use sgx_trts::libc;
use sgx_trts::enclave::*;

//Only during dev
//use core::mem;

global_ctors_object! {
    VARNAME, func_name = {()}
}

// veh
extern "C" fn sample_exception_handler(_ : *mut sgx_exception_info_t) -> uint32_t {
    0
}

pub fn test_rsgx_get_thread_policy() {
    assert_eq!(rsgx_get_thread_policy(), SgxThreadPolicy::Bound);
}

pub fn test_trts_sizes() {
    //Only during dev
    //assert_eq!(mem::size_of::<global_data_t>(), 1488);
    //assert_eq!(mem::size_of::<thread_data_t>(), 160);
}

pub fn test_register_first_exception_handler() {
    let handle = rsgx_register_exception_handler(1, sample_exception_handler);
    assert!(handle.is_some());
    assert_eq!(rsgx_unregister_exception_handler(handle.unwrap()), true);
}

pub fn test_register_last_exception_handler() {
    let handle = rsgx_register_exception_handler(0, sample_exception_handler);
    assert!(handle.is_some());
    assert_eq!(rsgx_unregister_exception_handler(handle.unwrap()), true);
}

pub fn test_register_multiple_exception_handler() {
    let mut handler_vec : Vec<exception_handle> = Vec::new();
    let ntest:usize = 100;

    for i in 0..ntest {
        let handle = rsgx_register_exception_handler(i as uint32_t % 2,
                                                     sample_exception_handler);
        assert!(handle.is_some());
        handler_vec.push(handle.unwrap());
    }

    for i in 0..ntest {
        let h = handler_vec[i];
        assert_eq!(rsgx_unregister_exception_handler(h), true);
    }

    for i in 0..ntest {
        let h = handler_vec[i];
        assert_eq!(rsgx_unregister_exception_handler(h), false);
    }
}

// trts
pub fn test_read_rand(){
    let mut rand_arr = [0; 100];
    assert_eq!(rsgx_read_rand(&mut rand_arr[..]), Ok(()));
    let x = rand_arr.iter().fold(0, |sum, &x| sum + x);
    // Cannot all be zero
    assert_ne!(x, 0);
}

pub fn test_data_is_within_enclave() {
    #[derive(Clone, Copy)]
    struct SampleDs{
        x : i32,
        y : i32,
        z : [i32;100],
    };
    unsafe impl marker::ContiguousMemory for SampleDs{};
    let mut sample_object : SampleDs = SampleDs{ x : 0, y : 0, z : [0;100]};
    sample_object.x = 100;
    sample_object.y = 100;
    sample_object.z[0] = 100;
    assert_eq!(rsgx_data_is_within_enclave(&sample_object), true);

    let ooo;
    unsafe {
        let ppp = 0xdeadbeafdeadbeaf as * const u8;
        ooo = &*ppp;
    }
    assert_eq!(rsgx_data_is_within_enclave(ooo), false);
}

pub fn test_slice_is_within_enlave() {
    let one_array = [0; 100];
    assert_eq!(rsgx_slice_is_within_enclave(&one_array[..]), true);

    // TODO: Not compiling
    //let mut ooo : &[u8];
    //unsafe {
    //    let ppp = 0xdeadbeafdeadbeaf as * const [u8];
    //    ooo = &*ppp;
    //}
    //assert_eq!(rsgx_slice_is_within_enclave(ooo), false);
}

pub fn test_raw_is_within_enclave(){
    assert_eq!(rsgx_raw_is_within_enclave(test_raw_is_within_enclave as * const u8,
                                          10),
               true);
    assert_eq!(rsgx_raw_is_within_enclave(0xdeadbeafdeadbeaf as * const u8,
                                          10),
               false);
}

pub fn test_data_is_outside_enclave() {
    #[derive(Clone, Copy)]
    struct SampleDs{
        x : i32,
        y : i32,
        z : [i32;100],
    };
    unsafe impl marker::ContiguousMemory for SampleDs{};
    let mut sample_object : SampleDs = SampleDs{ x : 0, y : 0, z : [0;100]};
    sample_object.x = 100;
    sample_object.y = 100;
    sample_object.z[0] = 100;
    assert_eq!(rsgx_data_is_outside_enclave(&sample_object), false);

    let ooo;
    unsafe {
        let ppp = 0xdeadbeafdeadbeaf as * const u8;
        ooo = &*ppp;
    }
    assert_eq!(rsgx_data_is_outside_enclave(ooo), true);
}

pub fn test_slice_is_outside_enclave() {
    let one_array = [0; 100];
    assert_eq!(rsgx_slice_is_outside_enclave(&one_array[..]), false);

    // TODO: Not compiling
    //let mut ooo : &[u8];
    //unsafe {
    //    let ppp = 0xdeadbeafdeadbeaf as * const [u8];
    //    ooo = &*ppp;
    //}
    //assert_eq!(rsgx_slice_is_within_enclave(ooo), true);
}

pub fn test_raw_is_outside_enclave(){
    assert_eq!(rsgx_raw_is_outside_enclave(test_raw_is_outside_enclave as * const u8,
                                          10),
               false);
    assert_eq!(rsgx_raw_is_outside_enclave(0xdeadbeafdeadbeaf as * const u8,
                                          10),
               true);
}

// macros

pub fn test_global_ctors_object (){
    assert_eq!(VARNAME(), ());
}

// oom
// I don't think we can test oom

// error
pub fn test_error() {
    // XXX: Top 11 should be the same in all unix?
    let errorinfo_vec:Vec<(i32,&'static str)> = vec![
        (1 , "Operation not permitted"),
        (2 , "No such file or directory"),
        (3 , "No such process"),
        (4 , "Interrupted system call"),
        (5 , "Input/output error"),
        (6 , "Device not configured"),
        (7 , "Argument list too long"),
        (8 , "Exec format error"),
        (9 , "Bad file descriptor"),
        (10, "No child processes"),
        (11, "Resource deadlock avoided"),
        ];

    for case in errorinfo_vec {
        let mut buf:[i8;64] = [0;64];
        error::set_errno(case.0);
        unsafe { error::error_string(error::errno(),&mut buf[..]);}
        let answer:Vec<u8> = buf.iter().map(|&x| x as u8).collect();
        let ans_str = String::from_utf8(answer).unwrap();
        assert_eq!(ans_str.trim_matches('\0'),
                   case.1);
    }

}

// libc
pub fn test_rts_libc_memchr(){
    let test_str = "abcdedfg";
    assert_eq!(
            unsafe {
                libc::memchr(
                        test_str.as_ptr() as * const u8,
                        'd' as u8,
                        test_str.len())},
            test_str[3..].as_ptr());
    assert_eq!(
            unsafe {
                libc::memchr(
                        "abcdefg".as_ptr() as * const u8,
                        'z' as u8,
                        test_str.len())},
            0 as * const u8);
}

pub fn test_rts_libc_memrchr(){
    let test_str = "abcdedfg";
    assert_eq!(
            unsafe {
                libc::memrchr(
                        test_str.as_ptr() as * const u8,
                        'd' as u8,
                        test_str.len())},
            test_str[5..].as_ptr());
    assert_eq!(
            unsafe {
                libc::memrchr(
                        "abcdefg".as_ptr() as * const u8,
                        'z' as u8,
                        test_str.len())},
            0 as * const u8);
}

// memchr

pub fn test_rts_memchr_memchr(){
    let test_str = "abcdedfg".as_bytes();
    let needle = 'd' as u8;
    assert_eq!(memchr::memchr(needle, test_str), Some(3));
    let needle = 'z' as u8;
    assert_eq!(memchr::memchr(needle, test_str), None);
}

pub fn test_rts_memchr_memrchr(){
    let test_str = "abcdedfg".as_bytes();
    let needle = 'd' as u8;
    assert_eq!(memchr::memrchr(needle, test_str), Some(5));
    let needle = 'z' as u8;
    assert_eq!(memchr::memrchr(needle, test_str), None);
}

// ascii
pub fn test_ascii(){
    assert_eq!("café".to_ascii_uppercase(), "CAFÉ");
    assert_eq!("café".to_ascii_uppercase(), "CAFé");

    let ascii = 'a';
    let non_ascii = '❤';
    let int_ascii = 97;

    assert!(ascii.is_ascii());
    assert!(!non_ascii.is_ascii());
    assert!(int_ascii.is_ascii());
    assert_eq!('A', ascii.to_ascii_uppercase());
    assert_eq!('❤', non_ascii.to_ascii_uppercase());
    assert_eq!(65, int_ascii.to_ascii_uppercase());


    let ascii = 'A';
    let non_ascii = '❤';
    let int_ascii = 65;

    assert_eq!('a', ascii.to_ascii_lowercase());
    assert_eq!('❤', non_ascii.to_ascii_lowercase());
    assert_eq!(97, int_ascii.to_ascii_lowercase());

    let ascii1 = 'A';
    let ascii2 = 'a';
    let ascii3 = 'A';
    let ascii4 = 'z';

    assert!(ascii1.eq_ignore_ascii_case(&ascii2));
    assert!(ascii1.eq_ignore_ascii_case(&ascii3));
    assert!(!ascii1.eq_ignore_ascii_case(&ascii4));

    let mut ascii = 'a';
    ascii.make_ascii_uppercase();
    assert_eq!('A', ascii);

    let mut ascii = 'A';
    ascii.make_ascii_lowercase();
    assert_eq!('a', ascii);
}

// c_str
pub fn test_cstr(){
    let c_string = CString::new("foo").unwrap();
    let ptr = c_string.into_raw();

    unsafe {
        assert_eq!(b'f', *ptr as u8);
        assert_eq!(b'o', *ptr.offset(1) as u8);
        assert_eq!(b'o', *ptr.offset(2) as u8);
        assert_eq!(b'\0', *ptr.offset(3) as u8);
        // retake pointer to free memory
        let _ = CString::from_raw(ptr);
    }

    let c_string = CString::new("foo").unwrap();
    let bytes = c_string.into_bytes();
    assert_eq!(bytes, vec![b'f', b'o', b'o']);

    let c_string = CString::new("foo").unwrap();
    let bytes = c_string.into_bytes_with_nul();
    assert_eq!(bytes, vec![b'f', b'o', b'o', b'\0']);

    let c_string = CString::new("foo").unwrap();
    let bytes = c_string.as_bytes();
    assert_eq!(bytes, &[b'f', b'o', b'o']);

    let c_string = CString::new("foo").unwrap();
    let bytes = c_string.as_bytes_with_nul();
    assert_eq!(bytes, &[b'f', b'o', b'o', b'\0']);

    let c_string = CString::new(b"foo".to_vec()).unwrap();
    let c_str = c_string.as_c_str();
    assert_eq!(c_str, CStr::from_bytes_with_nul(b"foo\0").unwrap());

    let c_string = CString::new(b"foo".to_vec()).unwrap();
    let boxed = c_string.into_boxed_c_str();
    assert_eq!(&*boxed, CStr::from_bytes_with_nul(b"foo\0").unwrap());

    let c_str = CStr::from_bytes_with_nul(b"foo\0").unwrap();
    assert_eq!(c_str.to_bytes(), b"foo");

    let c_str = CStr::from_bytes_with_nul(b"foo\0").unwrap();
    assert_eq!(c_str.to_bytes_with_nul(), b"foo\0");

    let c_str = CStr::from_bytes_with_nul(b"foo\0").unwrap();
    assert_eq!(c_str.to_str(), Ok("foo"));

    let c_str = CStr::from_bytes_with_nul(b"Hello World\0").unwrap();
    assert_eq!(c_str.to_string_lossy(), Cow::Borrowed("Hello World"));

    use std::borrow::Cow;

    let c_str = CStr::from_bytes_with_nul(b"Hello \xF0\x90\x80World\0").unwrap();
    assert_eq!(
                c_str.to_string_lossy(),
                    Cow::Owned(String::from("Hello �World")) as Cow<str>
            );
}
