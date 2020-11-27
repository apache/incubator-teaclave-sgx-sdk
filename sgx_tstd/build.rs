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

use std::env;

fn main() {
    if cfg!(feature = "backtrace") {
        println!("cargo:rustc-cfg=RUST_BACKTRACE=\"1\"");
        //println!("cargo:rustc-cfg=RUST_BACKTRACE=\"full\"");
    }

    let sdk_dir = env::var("SGX_SDK")
                    .unwrap_or_else(|_| "/opt/sgxsdk".to_string());
    let _is_sim = env::var("SGX_MODE")
                    .unwrap_or_else(|_| "HW".to_string());

    if cfg!(feature = "thread") {
        println!("cargo:rustc-link-search=native={}/lib64", sdk_dir);
        println!("cargo:rustc-link-lib=static=sgx_pthread");
    }

    // since nightly-2020-11-26 (rustc 2020-11-25), auto_traits replaced
    // optin_builtin_traits
    // see https://github.com/rust-lang/rust/commit/810324d1f31eb8d75e8f0044df720652986ef133
    if let Some(true) = version_check::is_min_date("2020-11-25") {
        println!("cargo:rustc-cfg=enable_auto_traits");
    }
}
