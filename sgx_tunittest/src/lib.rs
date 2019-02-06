// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

//! sgx_tunittest is for performing unit tests in enclaves.
//!
//! To use this crate, import the assertion macros defined in sgx_tstd and
//! this crate like this at first:
//!
//! ```
//! #[macro_use]
//! extern crate sgx_tstd as std;
//! #[macro_use]
//! extern crate sgx_tunittest;
//! ```
//!
//! Similar to `#[test]` in Rust, unit test functions are required
//! to take zero arguments and return nothing. One test is success
//! only when the test function returns without panic.
//!
//! Different from Rust, we don't use features like `#[test]`,
//! `#[should_panic]` for unit test function declaration. Instead,
//! to declare a unit test function, one just need implement it as normal.
//!
//! Here is a sample unit test function:
//!
//! ```
//! fn foo() {
//!     assert!(true);
//!     assert_eq!(1,1);
//!     assert_ne!(1,0);
//! }
//! ```
//!
//! To launch the unit test, one should use the macro `rsgx_unit_test!`.
//! For example, assuming that we have three unit test functions: `foo`,
//! `bar` and `zoo`. To start the test, just write as the following:
//!
//! ```
//! rsgx_unit_tests!(foo, bar, zoo);
//! ```
//!
//! sgx_tunittest supports fail test (something must panic). But it does
//! not provide it in Rust style (#[should_panic]). One should use macro
//! `should_panic!` to assert the statement that would panic. For example:
//!
//! ```
//! fn foo_panic() {
//!     let v = vec![]
//!     should_panic!(vec[0]); // vec[0] would panic
//! }
//! ```
//!
//! In this way, `vec[0]` would panic. But `should_panic!` catches it. Thus
//! `foo_panic` would pass the unit test.
//!

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![feature(const_fn)]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use std::string::String;
use std::vec::Vec;

/// This macro implements the fail test.
///
/// For example, in traditional Rust testing, we write
///
/// ```
/// #[test]
/// #[should_panic]
/// fn foo () {
///     assert!(false);
/// }
/// ```
///
/// This test would pass because it would panic and is expected to panic
/// (`#[should_panic]`).
///
/// An equivalent version of Rust SGX unit test is:
///
/// ```
/// fn foo() {
///     should_panic!(assert!(false));
/// }
/// ```
///
/// This requires developer to identify the line which triggers panic exactly.
#[macro_export]
macro_rules! should_panic {
    ($fmt:expr) => ({
        match panic::catch_unwind( || { $fmt }).is_err() {
            true => {},
            false => {
                ::std::rt::begin_panic($fmt, {
                    // static requires less code at runtime, more constant data
                    static _FILE_LINE_COL: (&'static str, u32, u32) = (file!(), line!(), column!());
                    &_FILE_LINE_COL
                })
            }
        }
    });
}

/// This macro works as test case driver.
///
/// `rsgx_unit_tests!` works as a variadic function. It takes a list of test
/// case function as arguments and then execute them sequentially. It prints
/// the statistics on the test result at the end.
///
/// One test fails if and only if it panics. For fail test (similar to
/// `#[should_panic]` in Rust, one should wrap the line which would panic with
/// macro `should_panic!`.
///
/// Here is one sample. For the entire sample, please reference to the sample
/// codes in this project.
///
/// ```
/// #[macro_use]
/// extern crate sgx_tstd as std;
/// #[macro_use]
/// extern crate sgx_unittest;
///
/// #[no_mangle]
/// pub extern "C"
/// fn test_ecall() -> sgx_status_t {
///     rsgx_unit_tests!(foo, bar, zoo);
///     sgx_status_t::SGX_SUCCESS
/// }
/// ```
#[macro_export]
macro_rules! rsgx_unit_tests {
    (
        $($f : ident),*
    ) => {
        rsgx_unit_test_start();
        let mut ntestcases : u64 = 0u64;
        let mut failurecases : Vec<String> = Vec::new();
        $(rsgx_unit_test(&mut ntestcases, &mut failurecases, $f,stringify!($f));)*
        rsgx_unit_test_end(ntestcases, failurecases);
    }
}

/// A prologue function for Rust SGX unit testing.
///
/// To initiate the test environment, `rsgx_unit_tests!` macro would trigger
/// `rsgx_unit_test_start` at the very beginning. `rsgx_unit_test_start` inits
/// the test counter and fail test list, and print the prologue message.
pub fn rsgx_unit_test_start () {
    println!("\nstart running tests");
}

/// An epilogue function for Rust SGX unit testing.
///
/// `rsgx_unit_test_end` prints the statistics on test result, including
/// a list of failed tests and the statistics.
pub fn rsgx_unit_test_end(ntestcases : u64, failurecases : Vec<String>) {
    let ntotal = ntestcases as usize;
    let nsucc  = ntestcases as usize - failurecases.len();

    if failurecases.len() != 0{
        let vfailures = failurecases;
        print!("\nfailures: ");
        println!("    {}",
                 vfailures.iter()
                          .fold(
                              String::new(),
                              |s, per| s + "\n    " + per));
    }

    if ntotal == nsucc {
        print!("\ntest result \x1B[1;32mok\x1B[0m. ");
    } else {
        print!("\ntest result \x1B[1;31mFAILED\x1B[0m. ");
    }

    println!("{} tested, {} passed, {} failed", ntotal, nsucc, ntotal - nsucc);
}

/// Perform one test case at a time.
///
/// This is the core function of sgx_tunittest. It runs one test case at a
/// time and saves the result. On test passes, it increases the passed counter
/// and on test fails, it records the failed test.
/// Required test function must be `Fn()`, taking nothing as input and returns
/// nothing.
pub fn rsgx_unit_test<F, R>(ncases: &mut u64, failurecases: &mut Vec<String>, f:F, name: &str)
    where F: FnOnce() -> R + std::panic::UnwindSafe {
    *ncases = *ncases + 1;
    match std::panic::catch_unwind (|| { f(); } ).is_ok() {
        true => {
                  println!("{} {} ... {}!",
                           "testing",
                           name,
                           "\x1B[1;32mok\x1B[0m");
                },
        false => {
                  println!("{} {} ... {}!",
                           "testing",
                            name,
                           "\x1B[1;31mfailed\x1B[0m");
                  failurecases.push(String::from(name));
        },
    }
}

