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

use crate::QveReportInfo;
use core::slice;
use sgx_types::error::Quote3Error;
use sgx_types::types::time_t;
use sgx_types::types::{QlQeReportInfo, QlQvResult};

#[allow(unaligned_references)]
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn sgx_tvl_verify_qve_report_and_identity(
    quote: *const u8,
    quote_size: u32,
    qve_report_info: *const QlQeReportInfo,
    expiration_check_date: time_t,
    collateral_expiration_status: u32,
    quote_verification_result: QlQvResult,
    supplemental_data: *const u8,
    supplemental_data_size: u32,
    qve_isvsvn_threshold: u16,
) -> Quote3Error {
    if quote.is_null() || quote_size == 0 || qve_report_info.is_null() {
        return Quote3Error::InvalidParameter;
    }

    if supplemental_data.is_null() && supplemental_data_size != 0 {
        return Quote3Error::InvalidParameter;
    }
    if !supplemental_data.is_null() && supplemental_data_size == 0 {
        return Quote3Error::InvalidParameter;
    }

    let quote = slice::from_raw_parts(quote, quote_size as usize);
    let supplemental_data = if !supplemental_data.is_null() {
        Some(slice::from_raw_parts(
            supplemental_data,
            supplemental_data_size as usize,
        ))
    } else {
        None
    };

    let qve_report_info = &*qve_report_info;
    let qve_report_info = QveReportInfo {
        qve_report: &qve_report_info.qe_report,
        expiration_time: expiration_check_date,
        collateral_expiration_status,
        quote_verification_result,
        qve_nonce: qve_report_info.nonce,
        supplemental_data,
    };

    match qve_report_info.verify_report_and_identity(quote, qve_isvsvn_threshold) {
        Ok(_) => (),
        Err(e) => return e,
    };

    Quote3Error::Success
}
