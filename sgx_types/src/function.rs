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

use crate::error::{PceError, QcnlError, Quote3Error, SgxStatus};
use crate::metadata::MetaData;
use crate::types::*;

// #[link(name = "sgx_urts")]
extern "C" {
    //
    // sgx_urts.h
    //
    pub fn sgx_create_enclave(
        enclave_file: *const c_char,
        debug: i32,
        launch_token: *mut LaunchToken,
        launch_token_updated: *mut i32,
        enclave_id: *mut EnclaveId,
        misc_attr: *mut MiscAttribute,
    ) -> SgxStatus;

    /* intel sgx sdk 2.1.3 */
    pub fn sgx_create_encrypted_enclave(
        enclave_file: *const c_char,
        debug: i32,
        launch_token: *mut LaunchToken,
        launch_token_updated: *mut i32,
        enclave_id: *mut EnclaveId,
        misc_attr: *mut MiscAttribute,
        sealed_key: *const u8,
    ) -> SgxStatus;

    /* intel sgx sdk 2.2 */
    pub fn sgx_create_enclave_ex(
        enclave_file: *const c_char,
        debug: i32,
        launch_token: *mut LaunchToken,
        launch_token_updated: *mut i32,
        enclave_id: *mut EnclaveId,
        misc_attr: *mut MiscAttribute,
        ex_features: u32,
        ex_features_p: *const [*const c_void; MAX_EXT_FEATURES_COUNT],
    ) -> SgxStatus;

    /* intel sgx sdk 2.4 */
    pub fn sgx_create_enclave_from_buffer_ex(
        buffer: *const u8,
        buffer_size: usize,
        debug: i32,
        enclave_id: *mut EnclaveId,
        misc_attr: *mut MiscAttribute,
        ex_features: u32,
        ex_features_p: *const [*const c_void; MAX_EXT_FEATURES_COUNT],
    ) -> SgxStatus;

    pub fn sgx_destroy_enclave(enclave_id: EnclaveId) -> SgxStatus;

    /* intel sgx sdk 2.4 */
    pub fn sgx_get_target_info(enclave_id: EnclaveId, target_info: *mut TargetInfo) -> SgxStatus;

    /* intel sgx sdk 2.9.1 */
    pub fn sgx_get_metadata(enclave_file: *const c_char, metadata: *mut MetaData) -> SgxStatus;
}

#[cfg(feature = "hyper")]
extern "C" {
    //
    // sgx_edger8r.h
    //
    pub fn sgx_ecall_ms_buffer_alloc(enclave_id: EnclaveId, size: usize) -> *mut c_void;
    pub fn sgx_ecall_ms_buffer_alloc_ex(
        enclave_id: EnclaveId,
        size: usize,
        error: *mut SgxStatus,
    ) -> *mut c_void;
    pub fn sgx_ecall_ms_buffer_alloc_aligned(
        enclave_id: EnclaveId,
        align: usize,
        size: usize,
        error: *mut SgxStatus,
    ) -> *mut c_void;
    pub fn sgx_ecall_ms_buffer_free(enclave_id: EnclaveId) -> SgxStatus;
    pub fn sgx_ecall_ms_buffer_remain_size(enclave_id: EnclaveId) -> usize;
}

extern "C" {
    //
    // sgx_edger8r.h
    //
    pub fn sgx_ecall(
        enclave_id: EnclaveId,
        index: i32,
        ocall_table: *const c_void,
        ms: *const c_void,
    ) -> SgxStatus;
}

/* intel sgx sdk 2.0 */
// #[link(name = "sgx_capable")]
extern "C" {
    //
    // sgx_capable.h
    //
    pub fn sgx_is_capable(sgx_capable: *mut i32) -> SgxStatus;
    pub fn sgx_cap_enable_device(sgx_device_status: *mut SgxDeviceStatus) -> SgxStatus;
    pub fn sgx_cap_get_status(sgx_device_status: *mut SgxDeviceStatus) -> SgxStatus;
}

//#[link(name = "sgx_epid")]
extern "C" {
    //
    // sgx_uae_epid.h
    //
    pub fn sgx_init_quote(p_target_info: *mut TargetInfo, p_gid: *mut EpidGroupId) -> SgxStatus;

    /* intel sgx sdk 1.9 */
    pub fn sgx_calc_quote_size(
        p_sig_rl: *const u8,
        sig_rl_size: u32,
        p_quote_size: *mut u32,
    ) -> SgxStatus;
    pub fn sgx_get_quote_size(p_sig_rl: *const u8, p_quote_size: *mut u32) -> SgxStatus;

    pub fn sgx_get_quote(
        p_report: *const Report,
        quote_type: QuoteSignType,
        p_spid: *const Spid,
        p_nonce: *const QuoteNonce,
        p_sig_rl: *const u8,
        sig_rl_size: u32,
        p_qe_report: *mut Report,
        p_quote: *mut Quote,
        quote_size: u32,
    ) -> SgxStatus;

    pub fn sgx_get_extended_epid_group_id(p_extended_epid_group_id: *mut u32) -> SgxStatus;
    pub fn sgx_report_attestation_status(
        p_platform_info: *const PlatformInfo,
        attestation_status: i32,
        p_update_info: *mut UpdateInfoBit,
    ) -> SgxStatus;

    /* intel sgx sdk 2.6 */
    pub fn sgx_check_update_status(
        p_platform_info: *const PlatformInfo,
        p_update_info: *mut UpdateInfoBit,
        config: u32,
        p_status: *mut u32,
    ) -> SgxStatus;
}

//#[link(name = "sgx_launch")]
extern "C" {
    //
    // sgx_uae_launch.h
    //
    pub fn sgx_get_whitelist_size(p_whitelist_size: *mut u32) -> SgxStatus;
    pub fn sgx_get_whitelist(p_whitelist: *mut u8, whitelist_size: u32) -> SgxStatus;

    /* intel sgx sdk 2.1 */
    pub fn sgx_register_wl_cert_chain(
        p_wl_cert_chain: *const u8,
        wl_cert_chain_size: u32,
    ) -> SgxStatus;
}

//#[link(name = "sgx_quote_ex")]
extern "C" {
    //
    // sgx_uae_quote_ex.h
    //
    /* intel sgx sdk 2.5 */
    pub fn sgx_select_att_key_id(
        p_att_key_id_list: *const u8,
        att_key_id_list_size: u32,
        pp_selected_key_id: *mut AttKeyId,
    ) -> SgxStatus;

    pub fn sgx_init_quote_ex(
        p_att_key_id: *const AttKeyId,
        p_qe_target_info: *mut TargetInfo,
        p_pub_key_id_size: *mut usize,
        p_pub_key_id: *mut u8,
    ) -> SgxStatus;

    pub fn sgx_get_quote_size_ex(
        p_att_key_id: *const AttKeyId,
        p_quote_size: *mut u32,
    ) -> SgxStatus;

    pub fn sgx_get_quote_ex(
        p_app_report: *const Report,
        p_att_key_id: *const AttKeyId,
        p_qe_report_info: *mut QeReportInfo,
        p_quote: *mut u8,
        quote_size: u32,
    ) -> SgxStatus;

    /* intel sgx sdk 2.9.1 */
    pub fn sgx_get_supported_att_key_id_num(p_att_key_id_num: *mut u32) -> SgxStatus;
    pub fn sgx_get_supported_att_key_ids(
        p_att_key_id_list: *mut AttKeyId,
        att_key_id_num: u32,
    ) -> SgxStatus;
}

//#[link(name = "sgx_pce_wrapper")]
extern "C" {
    //
    // sgx_pce.h
    //
    pub fn sgx_set_pce_enclave_load_policy(policy: QlRequestPolicy) -> PceError;
    pub fn sgx_pce_get_target(p_pce_target: *mut TargetInfo, p_pce_isv_svn: *mut u16) -> PceError;
    pub fn sgx_get_pce_info(
        p_report: *const Report,
        p_public_key: *const u8,
        key_size: u32,
        crypto_suite: u8,
        p_encrypted_ppid: *mut u8,
        encrypted_ppid_buf_size: u32,
        p_encrypted_ppid_out_size: *mut u32,
        p_pce_isv_svn: *mut u16,
        p_pce_id: *mut u16,
        p_signature_scheme: *mut u8,
    ) -> PceError;
    pub fn sgx_pce_sign_report(
        isv_svn: *const u16,
        cpu_svn: *const u16,
        p_report: *const Report,
        p_signature: *mut u8,
        signature_buf_size: u32,
        p_signature_out_size: *mut u32,
    ) -> PceError;

    /* intel DCAP 1.5 */
    pub fn sgx_get_pce_info_without_ppid(p_pce_isvsvn: *mut u16, p_pce_id: *mut u16) -> PceError;
}

//#[link(name = "sgx_dcap_ql")]
extern "C" {
    //
    // sgx_dcap_ql_wrapper.h
    //
    pub fn sgx_qe_set_enclave_load_policy(policy: QlRequestPolicy) -> Quote3Error;
    pub fn sgx_qe_get_target_info(p_qe_target_info: *mut TargetInfo) -> Quote3Error;
    pub fn sgx_qe_get_quote_size(p_quote_size: *mut u32) -> Quote3Error;
    pub fn sgx_qe_get_quote(
        p_app_report: *const Report,
        quote_size: u32,
        p_quote: *mut u8,
    ) -> Quote3Error;
    pub fn sgx_qe_cleanup_by_policy() -> Quote3Error;

    /* intel DCAP 1.6 */
    pub fn sgx_ql_set_path(path_type: QlPathType, p_path: *const c_char) -> Quote3Error;
}

pub type QlLoggingCallbackFn = extern "C" fn(level: QlLogLevel, message: *const c_char);

//#[link(name = "dcap_quoteprov")]
extern "C" {
    //
    // sgx_default_quote_provider.h
    //
    pub fn sgx_ql_get_quote_config(
        p_pck_cert_id: *const CQlPckCertId,
        pp_quote_config: *mut *mut CQlConfig,
    ) -> Quote3Error;
    pub fn sgx_ql_free_quote_config(p_quote_config: *const CQlConfig) -> Quote3Error;
    pub fn sgx_ql_get_quote_verification_collateral(
        fmspc: *const u8,
        fmspc_size: u16,
        pck_ra: *const c_char,
        pp_quote_collateral: *mut *mut CQlQveCollateral,
    ) -> Quote3Error;

    /* intel DCAP 1.13 */
    pub fn sgx_ql_get_quote_verification_collateral_with_params(
        fmspc: *const u8,
        fmspc_size: u16,
        pck_ra: *const c_char,
        custom_param: *const c_void,
        custom_param_length: u16,
        pp_quote_collateral: *mut *mut CQlQveCollateral,
    ) -> Quote3Error;
    pub fn sgx_ql_free_quote_verification_collateral(
        p_quote_collateral: *const CQlQveCollateral,
    ) -> Quote3Error;

    /* intel DCAP 1.14 */
    pub fn tdx_ql_get_quote_verification_collateral(
        fmspc: *const u8,
        fmspc_size: u16,
        pck_ra: *const c_char,
        pp_quote_collateral: *mut *mut CQlQveCollateral,
    ) -> Quote3Error;
    pub fn tdx_ql_free_quote_verification_collateral(
        p_quote_collateral: *const CQlQveCollateral,
    ) -> Quote3Error;

    pub fn sgx_ql_get_qve_identity(
        pp_qve_identity: *mut *mut c_char,
        p_qve_identity_size: *mut u32,
        pp_qve_identity_issuer_chain: *mut *mut c_char,
        p_qve_identity_issuer_chain_size: *mut u32,
    ) -> Quote3Error;
    pub fn sgx_ql_free_qve_identity(
        p_qve_identity: *const c_char,
        p_qve_identity_issuer_chain: *const c_char,
    ) -> Quote3Error;

    /* intel DCAP 1.4 */
    pub fn sgx_ql_get_root_ca_crl(
        pp_root_ca_crl: *mut *mut u8,
        p_root_ca_crl_size: *mut u16,
    ) -> Quote3Error;
    pub fn sgx_ql_free_root_ca_crl(p_root_ca_crl: *const uint8_t) -> Quote3Error;
    /* intel DCAP 2.14 */
    pub fn sgx_ql_set_logging_callback(logger: QlLoggingCallbackFn) -> Quote3Error;
}

//#[link(name = "sgx_default_qcnl_wrapper")]
extern "C" {
    //
    // sgx_default_qcnl_wrapper.h
    //
    pub fn sgx_qcnl_get_pck_cert_chain(
        p_pck_cert_id: *const CQlPckCertId,
        pp_quote_config: *mut *mut CQlConfig,
    ) -> QcnlError;
    pub fn sgx_qcnl_free_pck_cert_chain(p_quote_config: *const CQlConfig);
    pub fn sgx_qcnl_get_pck_crl_chain(
        ca: *const c_char,
        ca_size: u16,
        custom_param_b64_string: *const c_char,
        p_crl_chain: *mut *mut u8,
        p_crl_chain_size: *mut u16,
    ) -> QcnlError;
    pub fn sgx_qcnl_free_pck_crl_chain(p_crl_chain: *const u8);
    pub fn sgx_qcnl_get_tcbinfo(
        fmspc: *const c_char,
        fmspc_size: u16,
        custom_param_b64_string: *const c_char,
        p_tcbinfo: *mut *mut u8,
        p_tcbinfo_size: *mut u16,
    ) -> QcnlError;
    pub fn sgx_qcnl_free_tcbinfo(p_tcbinfo: *const u8);

    /* intel DCAP 1.14 */
    pub fn tdx_qcnl_get_tcbinfo(
        fmspc: *const c_char,
        fmspc_size: u16,
        custom_param_b64_string: *const c_char,
        p_tcbinfo: *mut *mut u8,
        p_tcbinfo_size: *mut u16,
    ) -> QcnlError;
    pub fn tdx_qcnl_free_tcbinfo(p_tcbinfo: *const u8);

    pub fn sgx_qcnl_get_qe_identity(
        qe_type: QeType,
        custom_param_b64_string: *const c_char,
        p_qe_identity: *mut *mut u8,
        p_qe_identity_size: *mut u16,
    ) -> QcnlError;
    pub fn sgx_qcnl_free_qe_identity(p_qe_identity: *const u8);
    pub fn sgx_qcnl_get_qve_identity(
        custom_param_b64_string: *const c_char,
        pp_qve_identity: *mut *mut c_char,
        p_qve_identity_size: *mut u32,
        pp_qve_identity_issuer_chain: *mut *mut c_char,
        p_qve_identity_issuer_chain_size: *mut u32,
    ) -> QcnlError;
    pub fn sgx_qcnl_free_qve_identity(
        p_qve_identity: *const c_char,
        p_qve_identity_issuer_chain: *const c_char,
    );
    pub fn sgx_qcnl_get_root_ca_crl(
        root_ca_cdp_url: *const c_char,
        custom_param_b64_string: *const c_char,
        p_root_ca_crl: *mut *mut u8,
        p_root_ca_crl_size: *mut u16,
    ) -> QcnlError;
    pub fn sgx_qcnl_free_root_ca_crl(p_root_ca_crl: *const u8);
    /* intel DCAP 1.13 */
    pub fn sgx_qcnl_get_api_version(p_major_ver: *mut u16, p_minor_ver: *mut u16) -> bool;
    pub fn sgx_qcnl_set_logging_callback(logger: QlLoggingCallbackFn) -> QcnlError;
}

//#[link(name = "dcap_quoteverify")]
extern "C" {
    //
    // sgx_dcap_quoteverify.h
    //
    pub fn sgx_qv_verify_quote(
        p_quote: *const u8,
        quote_size: u32,
        p_quote_collateral: *const CQlQveCollateral,
        expiration_check_date: time_t,
        p_collateral_expiration_status: *mut u32,
        p_quote_verification_result: *mut QlQvResult,
        p_qve_report_info: *mut QlQeReportInfo,
        supplemental_data_size: u32,
        p_supplemental_data: *mut u8,
    ) -> Quote3Error;
    pub fn sgx_qv_get_quote_supplemental_data_size(p_data_size: *mut u32) -> Quote3Error;
    pub fn sgx_qv_set_enclave_load_policy(policy: QlRequestPolicy) -> Quote3Error;

    /* intel DCAP 1.5 */
    pub fn sgx_qv_get_qve_identity(
        pp_qveid: *mut *mut u8,
        p_qveid_size: *mut u32,
        pp_qveid_issue_chain: *mut *mut u8,
        p_qveid_issue_chain_size: *mut u32,
        pp_root_ca_crl: *mut *mut u8,
        p_root_ca_crl_size: *mut u16,
    ) -> Quote3Error;

    pub fn sgx_qv_free_qve_identity(
        p_qveid: *const u8,
        p_qveid_issue_chain: *const u8,
        p_root_ca_crl: *const u8,
    ) -> Quote3Error;

    /* intel DCAP 1.6 */
    pub fn sgx_qv_set_path(path_type: QvPathType, p_path: *const c_char) -> Quote3Error;

    /* intel DCAP 1.13 */
    pub fn tdx_qv_get_quote_supplemental_data_size(p_data_size: *mut u32) -> Quote3Error;
    pub fn tdx_qv_verify_quote(
        p_quote: *const u8,
        quote_size: u32,
        p_quote_collateral: *const CQlQveCollateral,
        expiration_check_date: time_t,
        p_collateral_expiration_status: *mut u32,
        p_quote_verification_result: *mut QlQvResult,
        p_qve_report_info: *mut QlQeReportInfo,
        supplemental_data_size: u32,
        p_supplemental_data: *mut u8,
    ) -> Quote3Error;
}
