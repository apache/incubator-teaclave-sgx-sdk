// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

use super::*;
use alloc::borrow::Cow::{Borrowed, Owned};
use alloc::rc::Rc;
use alloc::sync::Arc;
use sgx_types::types::c_char;

use sgx_test_utils::test_case;

#[test_case]
fn c_to_rust() {
    let data = b"123\0";
    let ptr = data.as_ptr() as *const c_char;
    unsafe {
        assert_eq!(CStr::from_ptr(ptr).to_bytes(), b"123");
        assert_eq!(CStr::from_ptr(ptr).to_bytes_with_nul(), b"123\0");
    }
}

#[test_case]
fn simple() {
    let s = CString::new("1234").unwrap();
    assert_eq!(s.as_bytes(), b"1234");
    assert_eq!(s.as_bytes_with_nul(), b"1234\0");
}

#[test_case]
fn build_with_zero1() {
    assert!(CString::new(&b"\0"[..]).is_err());
}

#[test_case]
fn build_with_zero2() {
    assert!(CString::new(vec![0]).is_err());
}

#[test_case]
fn formatted() {
    let s = CString::new(&b"abc\x01\x02\n\xE2\x80\xA6\xFF"[..]).unwrap();
    assert_eq!(format!("{:?}", s), r#""abc\x01\x02\n\xe2\x80\xa6\xff""#);
}

#[test_case]
fn borrowed() {
    unsafe {
        let s = CStr::from_ptr(b"12\0".as_ptr() as *const _);
        assert_eq!(s.to_bytes(), b"12");
        assert_eq!(s.to_bytes_with_nul(), b"12\0");
    }
}

#[test_case]
fn to_str() {
    let data = b"123\xE2\x80\xA6\0";
    let ptr = data.as_ptr() as *const c_char;
    unsafe {
        assert_eq!(CStr::from_ptr(ptr).to_str(), Ok("123…"));
        assert_eq!(CStr::from_ptr(ptr).to_string_lossy(), Borrowed("123…"));
    }
    let data = b"123\xE2\0";
    let ptr = data.as_ptr() as *const c_char;
    unsafe {
        assert!(CStr::from_ptr(ptr).to_str().is_err());
        assert_eq!(
            CStr::from_ptr(ptr).to_string_lossy(),
            Owned::<str>(format!("123\u{FFFD}"))
        );
    }
}

#[test_case]
fn to_owned() {
    let data = b"123\0";
    let ptr = data.as_ptr() as *const c_char;

    let owned = unsafe { CStr::from_ptr(ptr).to_owned() };
    assert_eq!(owned.as_bytes_with_nul(), data);
}

#[test_case]
fn from_bytes_with_nul() {
    let data = b"123\0";
    let cstr = CStr::from_bytes_with_nul(data);
    assert_eq!(cstr.map(CStr::to_bytes), Ok(&b"123"[..]));
    let cstr = CStr::from_bytes_with_nul(data);
    assert_eq!(cstr.map(CStr::to_bytes_with_nul), Ok(&b"123\0"[..]));

    unsafe {
        let cstr = CStr::from_bytes_with_nul(data);
        let cstr_unchecked = CStr::from_bytes_with_nul_unchecked(data);
        assert_eq!(cstr, Ok(cstr_unchecked));
    }
}

#[test_case]
fn from_bytes_with_nul_unterminated() {
    let data = b"123";
    let cstr = CStr::from_bytes_with_nul(data);
    assert!(cstr.is_err());
}

#[test_case]
fn from_bytes_with_nul_interior() {
    let data = b"1\023\0";
    let cstr = CStr::from_bytes_with_nul(data);
    assert!(cstr.is_err());
}

#[test_case]
fn into_boxed() {
    let orig: &[u8] = b"Hello, world!\0";
    let cstr = CStr::from_bytes_with_nul(orig).unwrap();
    let boxed: Box<CStr> = Box::from(cstr);
    let cstring = cstr.to_owned().into_boxed_c_str().into_c_string();
    assert_eq!(cstr, &*boxed);
    assert_eq!(&*boxed, &*cstring);
    assert_eq!(&*cstring, cstr);
}

#[test_case]
fn boxed_default() {
    let boxed = <Box<CStr>>::default();
    assert_eq!(boxed.to_bytes_with_nul(), &[0]);
}

#[test_case]
fn test_c_str_clone_into() {
    let mut c_string = CString::new("lorem").unwrap();
    let c_ptr = c_string.as_ptr();
    let c_str = CStr::from_bytes_with_nul(b"ipsum\0").unwrap();
    c_str.clone_into(&mut c_string);
    assert_eq!(c_str, c_string.as_c_str());
    // The exact same size shouldn't have needed to move its allocation
    assert_eq!(c_ptr, c_string.as_ptr());
}

#[test_case]
fn into_rc() {
    let orig: &[u8] = b"Hello, world!\0";
    let cstr = CStr::from_bytes_with_nul(orig).unwrap();
    let rc: Rc<CStr> = Rc::from(cstr);
    let arc: Arc<CStr> = Arc::from(cstr);

    assert_eq!(&*rc, cstr);
    assert_eq!(&*arc, cstr);

    let rc2: Rc<CStr> = Rc::from(cstr.to_owned());
    let arc2: Arc<CStr> = Arc::from(cstr.to_owned());

    assert_eq!(&*rc2, cstr);
    assert_eq!(&*arc2, cstr);
}

#[test_case]
fn cstr_const_constructor() {
    const CSTR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Hello, world!\0") };

    assert_eq!(CSTR.to_str().unwrap(), "Hello, world!");
}

#[test_case]
fn cstr_index_from() {
    let original = b"Hello, world!\0";
    let cstr = CStr::from_bytes_with_nul(original).unwrap();
    let result = CStr::from_bytes_with_nul(&original[7..]).unwrap();

    assert_eq!(&cstr[7..], result);
}

#[test_case]
fn c_string_from_empty_string() {
    let original = "";
    let cstring = CString::new(original).unwrap();
    assert_eq!(original.as_bytes(), cstring.as_bytes());
    assert_eq!([b'\0'], cstring.as_bytes_with_nul());
}

#[test_case]
fn c_str_from_empty_string() {
    let original = b"\0";
    let cstr = CStr::from_bytes_with_nul(original).unwrap();
    assert_eq!([] as [u8; 0], cstr.to_bytes());
    assert_eq!([b'\0'], cstr.to_bytes_with_nul());
}
