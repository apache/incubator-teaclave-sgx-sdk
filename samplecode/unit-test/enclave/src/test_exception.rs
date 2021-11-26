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

use sgx_signal::exception::{register_exception, unregister};
use sgx_signal::ContinueType;
use sgx_trts::enclave;
use sgx_types::sgx_exception_info_t;
use std::backtrace::{self, PrintFormat};
use std::panic;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[no_mangle]
#[inline(never)]
fn test_abort() -> ! {
    let td = enclave::SgxThreadData::current();
    println!(
        "test_abort stack: {:x}-{:x}",
        td.stack_base(),
        td.stack_limit()
    );

    std::intrinsics::abort()
}

#[cfg_attr(not(feature = "hw_test"), allow(unreachable_code))]
pub fn test_exception_handler() {
    #[cfg(not(feature = "hw_test"))]
    return;

    let _ = backtrace::enable_backtrace("enclave.signed.so", PrintFormat::Full);

    let status = Arc::new(AtomicUsize::new(2));
    let handler1 = {
        let status = Arc::clone(&status);
        move |_info: &mut sgx_exception_info_t| {
            status.fetch_add(2, Ordering::Relaxed);
            ContinueType::Search
        }
    };

    let handler2 = {
        let status = Arc::clone(&status);
        move |_info: &mut sgx_exception_info_t| {
            status.store(1, Ordering::Relaxed);
            ContinueType::Search
        }
    };

    let r1 = register_exception(false, handler1);
    let r2 = register_exception(true, handler2);

    panic::catch_unwind(|| test_abort()).ok();

    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        let current = status.load(Ordering::Relaxed);
        match current {
            // Not yet
            0 => continue,
            // Good, we are done with the correct result
            _ if current == 3 => return,
            _ => panic!("Wrong result value {}", current),
        }
    }

    unregister(r1.unwrap());
    unregister(r2.unwrap());
    panic!("Timed out waiting for the exception");
}
