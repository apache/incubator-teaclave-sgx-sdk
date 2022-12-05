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

use crate::se::{EnclaveKey, EnclaveReport, EnclaveTarget, TeeReport};
use sgx_types::error::SgxStatus;
use sgx_types::types::{Key128bit, KeyRequest, Report, Report2Mac, ReportData, TargetInfo};

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_create_report(
    target_info: *const TargetInfo,
    report_data: *const ReportData,
    report: *mut Report,
) -> SgxStatus {
    if report.is_null() {
        return SgxStatus::InvalidParameter;
    }

    let result = match (target_info.is_null(), report_data.is_null()) {
        (true, true) => Report::for_self(),
        (false, false) => Report::for_target(&*target_info, &*report_data),
        (true, false) => Report::for_target(&TargetInfo::default(), &*report_data),
        (false, true) => Report::for_target(&*target_info, &ReportData::default()),
    };

    match result {
        Ok(r) => {
            *report = r;
            SgxStatus::Success
        }
        Err(e) => e,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_self_report() -> *const Report {
    Report::get_self() as *const Report
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_verify_report(report: *const Report) -> SgxStatus {
    if report.is_null() {
        return SgxStatus::InvalidParameter;
    }

    match (*report).verify() {
        Ok(_) => SgxStatus::Success,
        Err(e) => e,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_self_target(target_info: *mut TargetInfo) -> SgxStatus {
    if target_info.is_null() {
        return SgxStatus::InvalidParameter;
    }

    match TargetInfo::for_self() {
        Ok(t) => {
            *target_info = t;
            SgxStatus::Success
        }
        Err(e) => e,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_get_key(
    key_request: *const KeyRequest,
    key: *mut Key128bit,
) -> SgxStatus {
    if key_request.is_null() || key.is_null() {
        return SgxStatus::InvalidParameter;
    }

    match (*key_request).get_key() {
        Ok(k) => {
            *key = k;
            SgxStatus::Success
        }
        Err(e) => e,
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_verify_report2(report_mac: *const Report2Mac) -> SgxStatus {
    if report_mac.is_null() {
        return SgxStatus::InvalidParameter;
    }

    match (*report_mac).verify() {
        Ok(_) => SgxStatus::Success,
        Err(e) => e,
    }
}
