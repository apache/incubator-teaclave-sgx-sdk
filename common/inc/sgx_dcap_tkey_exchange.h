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

#ifndef _SGX_DCAP_TKEY_EXCHANGE_H_
#define _SGX_DCAP_TKEY_EXCHANGE_H_

#include "sgx.h"
#include "sgx_defs.h"
#include "sgx_dcap_key_exchange.h"
#include "sgx_qve_header.h"

#ifdef  __cplusplus
extern "C" {
#endif

/*
 * The sgx_ura_initiator_init function creates a context for the unidirectional remote attestation
 * and key exchange process.
 *
 * @param p_pub_key The EC public key of the service provider based on the NIST
 *                  P-256 elliptic curve.
 * @param p_context The output context for the subsequent remote attestation
 *                  and key exchange process.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_ura_initiator_init(
    const sgx_ec256_public_t *p_pub_key,
    sgx_ra_context_t *p_context);

/*
 * The sgx_mra_initiator_init function creates a context for the mutual remote attestation
 * and key exchange process.
 *
 * @param p_context The output context for the subsequent remote attestation
 *                  and key exchange process.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_mra_initiator_init(
    sgx_ra_context_t *p_context);

sgx_status_t sgx_dcap_ra_get_ga(
    sgx_ra_context_t context,
    sgx_ec256_public_t *g_a);

sgx_status_t sgx_dcap_ura_proc_msg2(
    sgx_ra_context_t context,
    const sgx_dcap_ura_msg2_t *msg2,
    const sgx_target_info_t *qe_target,
    sgx_report_t *report,
    sgx_quote_nonce_t *nonce);

 sgx_status_t sgx_dcap_mra_proc_msg2(
    sgx_ra_context_t context,
    const sgx_dcap_mra_msg2_t *msg2,
    uint32_t msg2_size,
    time_t expiration_time,
    uint32_t collateral_expiration_status,
    sgx_ql_qv_result_t quote_verification_result,
    const sgx_quote_nonce_t *qve_nonce,
    const sgx_report_t *qve_report,
    const uint8_t *supplemental_data,
    uint32_t supplemental_data_size,
    const sgx_target_info_t *qe_target,
    sgx_report_t *report,
    sgx_quote_nonce_t *nonce);

sgx_status_t sgx_dcap_ra_get_msg3(
    sgx_ra_context_t context,
    const sgx_report_t* qe_report,
    sgx_dcap_ra_msg3_t *msg3,
    uint32_t msg3_size);

/*
 * The sgx_ra_initiator_get_keys function is used to get the negotiated keys of a remote
 * attestation and key exchange session. This function should only be called after
 * the service provider or the responder accepts the remote attestation and key exchange
 * protocol message 3.
 *
 * @param context   Context returned by sgx_mra_initiator_init or sgx_ura_initiator_init.
 * @param type      The specifier of keys, can be SGX_RA_KEY_MK, SGX_RA_KEY_SK.
 * @param p_key     The key returned.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_ra_initiator_get_keys(
    sgx_ra_context_t context,
    sgx_ra_key_type_t type,
    sgx_ra_key_128_t *p_key);

/*
 * The sgx_mra_initiator_get_peer_identity function is used to get identity information of responder
 * and quote verification result. This function should only be called after key exchange protocol message 2.
 *
 * @param context                     Context returned by sgx_mra_initiator_init.
 * @param quote_verification_result   Quote verification result.
 * @param responder_identity          The identity information of responder.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_mra_initiator_get_peer_identity(
    sgx_ra_context_t context,
    sgx_ql_qv_result_t *quote_verification_result,
    sgx_dcap_enclave_identity_t *responder_identity);

/*
 * Call the sgx_ra_initiator_close function to release the remote attestation and key
 * exchange context after the process is done and the context isn't needed
 * anymore.
 *
 * @param context   Context returned by sgx_mra_initiator_init or sgx_ura_initiator_init.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_ra_initiator_close(
    sgx_ra_context_t context);

/*
 * The sgx_mra_responder_init function creates a context for the mutual remote attestation
 * and key exchange process.
 *
 * @param p_context The output context for the subsequent remote attestation
 *                  and key exchange process.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_mra_responder_init(
    sgx_ra_context_t *p_context);

sgx_status_t sgx_dcap_mra_proc_msg1(
    sgx_ra_context_t context,
    const sgx_dcap_ra_msg1_t *msg1,
    const sgx_target_info_t *qe_target,
    sgx_ec256_public_t *g_b,
    sgx_report_t *report,
    sgx_quote_nonce_t *nonce);

sgx_status_t sgx_dcap_mra_get_msg2(
    sgx_ra_context_t context,
    const sgx_report_t* qe_report,
    sgx_dcap_mra_msg2_t *msg2,
    uint32_t msg2_size);

sgx_status_t sgx_dcap_mra_proc_msg3(
    sgx_ra_context_t context,
    const sgx_dcap_ra_msg3_t *msg3,
    uint32_t msg3_size,
    time_t expiration_time,
    uint32_t collateral_expiration_status,
    sgx_ql_qv_result_t quote_verification_result,
    const sgx_quote_nonce_t *qve_nonce,
    const sgx_report_t *qve_report,
    const uint8_t *supplemental_data,
    uint32_t supplemental_data_size);

/*
 * The sgx_mra_responder_get_keys function is used to get the negotiated keys of a remote
 * attestation and key exchange session. This function should only be called after
 * the initiator accepts the remote attestation and key exchange protocol message 3.
 *
 * @param context   Context returned by sgx_mra_responder_init.
 * @param type      The specifier of keys, can be SGX_RA_KEY_MK, SGX_RA_KEY_SK.
 * @param p_key     The key returned.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_mra_responder_get_keys(
    sgx_ra_context_t context,
    sgx_ra_key_type_t type,
    sgx_ra_key_128_t *p_key);

/*
 * The sgx_mra_responder_get_peer_identity function is used to get identity information of initiator
 * and quote verification result. This function should only be called after key exchange protocol message 3.
 *
 * @param context                     Context returned by sgx_mra_responder_init.
 * @param quote_verification_result   Quote verification result.
 * @param responder_identity          The identity information of initiator.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_mra_responder_get_peer_identity(
    sgx_ra_context_t context,
    sgx_ql_qv_result_t *quote_verification_result,
    sgx_dcap_enclave_identity_t *initiator_identity);

/*
 * Call the sgx_mra_responder_close function to release the remote attestation and key
 * exchange context after the process is done and the context isn't needed
 * anymore.
 *
 * @param context   Context returned by sgx_mra_responder_init.
 * @return sgx_status_t
 */
sgx_status_t SGXAPI sgx_mra_responder_close(
    sgx_ra_context_t context);

#ifdef  __cplusplus
}
#endif

#endif
