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
// under the License.

extern crate sgx_types;

use regex::Regex;
use sgx_types::error::SgxStatus;

/// A function simply test regex lib
///
/// # Return value
///
/// Always returns SGX_SUCCESS
#[no_mangle]
pub unsafe extern "C" fn regex() -> SgxStatus {
    const TO_SEARCH: &str = "On 2010-03-14, foo happened. On 2014-10-14, bar happened.";

    let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();

    for caps in re.captures_iter(TO_SEARCH) {
        // Note that all of the unwraps are actually OK for this regex
        // because the only way for the regex to match is if all of the
        // capture groups match. This is not true in general though!
        println!(
            "year: {}, month: {}, day: {}",
            caps.get(1).unwrap().as_str(),
            caps.get(2).unwrap().as_str(),
            caps.get(3).unwrap().as_str()
        );
    }

    SgxStatus::Success
}
