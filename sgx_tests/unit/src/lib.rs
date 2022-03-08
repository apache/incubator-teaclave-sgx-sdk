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

//! sgx_unit_test is for performing unit tests in enclaves.
//!
//! To use this crate, import the assertion macros defined in sgx_tstd and
//! this crate like this at first:
//!
//! ```
//! #[macro_use]
//! extern crate sgx_tstd as std;
//! #[macro_use]
//! extern crate sgx_unit_test;
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
//! To launch the unit test, one should use the macro `unit_test!`.
//! For example, assuming that we have three unit test functions: `foo`,
//! `bar` and `zoo`. To start the test, just write as the following:
//!
//! ```
//! run_unit_tests!(foo, bar, zoo);
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

#![cfg_attr(not(target_vendor = "teaclave"), no_std)]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]

#[cfg(not(target_vendor = "teaclave"))]
#[macro_use]
extern crate sgx_tstd as std;

pub use sgx_test_utils;
pub use sgx_test_utils::{bench_case, test_case};

use sgx_test_utils::{fmt_bench_samples, BenchSamples, Bencher, Summary};
use std::cmp;
use std::panic::AssertUnwindSafe;
use std::string::String;
use std::vec::Vec;

#[macro_export]
macro_rules! run_test_cases {
    ($predicate:expr) => {{
        $crate::unit_test_start();
        let mut ntestcases: u64 = 0_u64;
        let mut failurecases: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();

        for t in $crate::sgx_test_utils::TestIter::<$crate::sgx_test_utils::TestCase>::new() {
            if $predicate(&t.0) {
                $crate::unit_test(&mut ntestcases, &mut failurecases, t.1, &t.0);
            }
        }

        $crate::unit_test_end(ntestcases, failurecases)
    }};
    () => {
        $crate::run_test_cases!(|_| true);
    };
}

#[macro_export]
macro_rules! run_bench_cases {
    ($predicate:expr, $freq:expr) => {{
        $crate::unit_test_start();
        let mut ntestcases: u64 = 0_u64;
        let mut failurecases: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();

        for t in $crate::sgx_test_utils::TestIter::<$crate::sgx_test_utils::BenchCase>::new() {
            if $predicate(&t.0) {
                $crate::benchmark(&mut ntestcases, &mut failurecases, $freq, t.1, &t.0);
            }
        }

        $crate::unit_test_end(ntestcases, failurecases)
    }};
    ($freq:expr) => {
        $crate::run_bench_cases!(|_| true, $freq);
    };
}

/// This macro works as test case driver.
///
/// `run_unit_tests!` works as a variadic function. It takes a list of test
/// case function as arguments and then execute them sequentially. It prints
/// the statistics on the test result at the end, and returns the amount of
/// failed tests. meaning if everything works the return vlaue will be 0.
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
/// extern crate sgx_unit_test;
///
/// #[no_mangle]
/// pub extern "C"
/// fn test_ecall() -> sgx_status_t {
///     run_unit_tests!(foo, bar, zoo);
///     sgx_status_t::SGX_SUCCESS
/// }
/// ```
#[macro_export]
macro_rules! run_unit_tests {
    (
        $($f : expr),* $(,)?
    ) => {
        {
            $crate::unit_test_start();
            let mut ntestcases: u64 = 0_u64;
            let mut failurecases: ::std::vec::Vec<String> = ::std::vec::Vec::new();
            $($crate::unit_test(&mut ntestcases, &mut failurecases, $f,stringify!($f));)*
            $crate::unit_test_end(ntestcases, failurecases)
        }
    }
}

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
    ($fmt:expr) => {{
        match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| $fmt)).is_err() {
            true => {}
            false => ::std::rt::begin_panic($fmt),
        }
    }};
}

/// A prologue function for Rust SGX unit testing.
///
/// To initiate the test environment, `run_unit_tests!` macro would trigger
/// `unit_test_start` at the very beginning. `unit_test_start` inits
/// the test counter and fail test list, and print the prologue message.
pub fn unit_test_start() {
    println!("\nstart running tests");
}

/// An epilogue function for Rust SGX unit testing.
///
/// `unit_test_end` prints the statistics on test result, including
/// a list of failed tests and the statistics.
/// It will return the amount of failed tests. (success == 0)
pub fn unit_test_end(ntestcases: u64, failurecases: Vec<String>) -> usize {
    let ntotal = ntestcases as usize;
    let nsucc = ntestcases as usize - failurecases.len();

    if !failurecases.is_empty() {
        print!("\nfailures: ");
        println!(
            "    {}",
            failurecases
                .iter()
                .fold(String::new(), |s, per| s + "\n    " + per)
        );
    }

    if ntotal == nsucc {
        print!("\ntest result \x1B[1;32mok\x1B[0m. ");
    } else {
        print!("\ntest result \x1B[1;31mFAILED\x1B[0m. ");
    }

    println!(
        "{} tested, {} passed, {} failed",
        ntotal,
        nsucc,
        ntotal - nsucc
    );
    failurecases.len()
}

/// Perform one test case at a time.
///
/// This is the core function of sgx_unit_test. It runs one test case at a
/// time and saves the result. On test passes, it increases the passed counter
/// and on test fails, it records the failed test.
/// Required test function must be `Fn()`, taking nothing as input and returns
/// nothing.
#[allow(clippy::print_literal)]
pub fn unit_test<F, R>(ncases: &mut u64, failurecases: &mut Vec<String>, f: F, name: &str)
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    *ncases += 1;
    match std::panic::catch_unwind(|| {
        f();
    })
    .is_ok()
    {
        true => {
            println!("{} ... {}!", name, "\x1B[1;32mok\x1B[0m");
        }
        false => {
            println!("{} ... {}!", name, "\x1B[1;31mfailed\x1B[0m");
            failurecases.push(String::from(name));
        }
    }
}

#[allow(clippy::print_literal)]
pub fn benchmark<F>(ncases: &mut u64, failurecases: &mut Vec<String>, freq: u64, f: F, name: &str)
where
    F: FnMut(&mut Bencher),
{
    let mut bs = Bencher::new(freq);
    *ncases += 1;
    match std::panic::catch_unwind(AssertUnwindSafe(|| bs.bench(f))) {
        Ok(Some(ns_iter_summ)) => {
            let ns_iter = cmp::max(ns_iter_summ.median as u64, 1);
            let mb_s = bs.bytes * 1000 / ns_iter;

            let bs = BenchSamples {
                ns_iter_summ,
                mb_s: mb_s as usize,
            };
            println!(
                "{} ... {}: {}",
                name,
                "\x1B[1;32mbench\x1B[0m",
                fmt_bench_samples(&bs)
            );
        }
        Ok(None) => {
            // iter not called, so no data.
            // FIXME: error in this case?
            let samples: &mut [f64] = &mut [0.0_f64; 1];
            let bs = BenchSamples {
                ns_iter_summ: Summary::new(samples),
                mb_s: 0,
            };
            println!(
                "{} ... {}: {}",
                name,
                "\x1B[1;32mbench\x1B[0m",
                fmt_bench_samples(&bs)
            );
        }
        Err(_) => {
            println!("{} ... {}!", name, "\x1B[1;31mfailed\x1B[0m");
            failurecases.push(String::from(name));
        }
    };
}
