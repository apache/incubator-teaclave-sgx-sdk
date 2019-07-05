// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

use crate::types::*;
use crate::*;

//#[link(name = "sgx_tstdc")]
extern {
    //
    // sgx_cpuid.h
    //
    pub fn sgx_cpuid(cpuinfo: * mut [int32_t; 4], leaf: int32_t) -> sgx_status_t;
    pub fn sgx_cpuidex(cpuinfo: * mut [int32_t; 4], leaf: int32_t, subleaf: int32_t) -> sgx_status_t;

    //
    // sgx_spinlock.h
    //
    pub fn sgx_spin_lock(lock: * mut sgx_spinlock_t) -> uint32_t;
    pub fn sgx_spin_unlock(lock: * mut sgx_spinlock_t) -> uint32_t;

    //
    // sgx_thread.h
    //
    pub fn sgx_thread_mutex_init(mutex: * mut sgx_thread_mutex_t, unused: * const sgx_thread_mutexattr_t) -> int32_t;
    pub fn sgx_thread_mutex_destroy(mutex: * mut sgx_thread_mutex_t) -> int32_t;

    pub fn sgx_thread_mutex_lock(mutex: * mut sgx_thread_mutex_t) -> int32_t;
    pub fn sgx_thread_mutex_trylock(mutex: * mut sgx_thread_mutex_t) -> int32_t;
    pub fn sgx_thread_mutex_unlock(mutex: * mut sgx_thread_mutex_t) -> int32_t;

    pub fn sgx_thread_cond_init(cond: * mut sgx_thread_cond_t, unused: * const sgx_thread_condattr_t) -> int32_t;
    pub fn sgx_thread_cond_destroy(cond: * mut sgx_thread_cond_t) -> int32_t;

    pub fn sgx_thread_cond_wait(cond: * mut sgx_thread_cond_t, mutex: * mut sgx_thread_mutex_t) -> int32_t;
    pub fn sgx_thread_cond_signal(cond: * mut sgx_thread_cond_t) -> int32_t;
    pub fn sgx_thread_cond_broadcast(cond: * mut sgx_thread_cond_t) -> int32_t;

    pub fn sgx_thread_self() -> sgx_thread_t;
    pub fn sgx_thread_equal(a: sgx_thread_t, b: sgx_thread_t)  -> int32_t;
}


//#[link(name = "sgx_tservice")]
extern {

    //
    // sgx_dh.h
    //
    pub fn sgx_dh_init_session(role: sgx_dh_session_role_t, session: * mut sgx_dh_session_t) -> sgx_status_t;

    pub fn sgx_dh_responder_gen_msg1(msg1: * mut sgx_dh_msg1_t,
                                     dh_session: * mut sgx_dh_session_t) -> sgx_status_t;

    pub fn sgx_dh_initiator_proc_msg1(msg1: * const sgx_dh_msg1_t,
                                      msg2: * mut sgx_dh_msg2_t,
                                      dh_session: * mut sgx_dh_session_t) -> sgx_status_t;

    pub fn sgx_dh_responder_proc_msg2(msg2: * const sgx_dh_msg2_t,
                                      msg3: * mut sgx_dh_msg3_t,
                                      dh_session: * mut sgx_dh_session_t,
                                      aek: * mut sgx_key_128bit_t,
                                      initiator_identity: * mut sgx_dh_session_enclave_identity_t) -> sgx_status_t;

    pub fn sgx_dh_initiator_proc_msg3(msg3: * const sgx_dh_msg3_t,
                                      dh_session: * mut sgx_dh_session_t,
                                      aek: * mut sgx_key_128bit_t,
                                      responder_identity: * mut sgx_dh_session_enclave_identity_t) -> sgx_status_t;

    //
    // sgx_tae_service.h
    //
    pub fn sgx_create_pse_session() -> sgx_status_t;
    pub fn sgx_close_pse_session() -> sgx_status_t;
    pub fn sgx_get_ps_sec_prop(security_property: * mut sgx_ps_sec_prop_desc_t) -> sgx_status_t;
    /* intel sgx sdk 1.8 */
    pub fn sgx_get_ps_sec_prop_ex(security_property: * mut sgx_ps_sec_prop_desc_ex_t) -> sgx_status_t;
    pub fn sgx_get_trusted_time(current_time: * mut sgx_time_t, time_source_nonce: * mut sgx_time_source_nonce_t) -> sgx_status_t;

    pub fn sgx_create_monotonic_counter_ex(owner_policy: uint16_t,
                                           owner_attribute_mask: * const sgx_attributes_t,
                                           counter_uuid: * mut sgx_mc_uuid_t,
                                           counter_value: * mut uint32_t) -> sgx_status_t;

    pub fn sgx_create_monotonic_counter(counter_uuid: * mut sgx_mc_uuid_t, counter_value: * mut uint32_t) -> sgx_status_t;
    pub fn sgx_destroy_monotonic_counter(counter_uuid: * const sgx_mc_uuid_t) -> sgx_status_t;
    pub fn sgx_increment_monotonic_counter(counter_uuid: * const sgx_mc_uuid_t, counter_value: * mut uint32_t) -> sgx_status_t;
    pub fn sgx_read_monotonic_counter(counter_uuid: * const sgx_mc_uuid_t, counter_value: * mut uint32_t) -> sgx_status_t;


    //
    // sgx_tseal.h
    //
    pub fn sgx_calc_sealed_data_size(add_mac_txt_size: uint32_t, txt_encrypt_size: uint32_t) -> uint32_t;
    pub fn sgx_get_add_mac_txt_len(p_sealed_data: * const sgx_sealed_data_t) -> uint32_t;
    pub fn sgx_get_encrypt_txt_len(p_sealed_data: * const sgx_sealed_data_t) -> uint32_t;

    pub fn sgx_seal_data(additional_MACtext_length: uint32_t,
                         p_additional_MACtext: * const uint8_t,
                         text2encrypt_length: uint32_t,
                         p_text2encrypt: * const uint8_t,
                         sealed_data_size: uint32_t,
                         p_sealed_data: * mut sgx_sealed_data_t) -> sgx_status_t;

    pub fn sgx_seal_data_ex(key_policy: uint16_t,
                            attribute_mask: sgx_attributes_t,
                            misc_mask: sgx_misc_select_t,
                            additional_MACtext_length: uint32_t,
                            p_additional_MACtext: * const uint8_t,
                            text2encrypt_length: uint32_t,
                            p_text2encrypt: * const uint8_t,
                            sealed_data_size: uint32_t,
                            p_sealed_data: * mut sgx_sealed_data_t) -> sgx_status_t;

    pub fn sgx_unseal_data(p_sealed_data: * const sgx_sealed_data_t,
                           p_additional_MACtext: * mut uint8_t,
                           p_additional_MACtext_length: * mut uint32_t,
                           p_decrypted_text: * mut uint8_t,
                           p_decrypted_text_length: * mut uint32_t) -> sgx_status_t;

    pub fn sgx_mac_aadata(additional_MACtext_length: uint32_t,
                          p_additional_MACtext: * const uint8_t,
                          sealed_data_size: uint32_t,
                          p_sealed_data: * mut sgx_sealed_data_t) -> sgx_status_t;

    pub fn sgx_mac_aadata_ex(key_policy: uint16_t,
                             attribute_mask: sgx_attributes_t,
                             misc_mask: sgx_misc_select_t,
                             additional_MACtext_length: uint32_t,
                             p_additional_MACtext: * const uint8_t,
                             sealed_data_size: uint32_t,
                             p_sealed_data: * mut sgx_sealed_data_t) -> sgx_status_t;

    pub fn sgx_unmac_aadata(p_sealed_data: * const sgx_sealed_data_t,
                            p_additional_MACtext: * mut uint8_t,
                            p_additional_MACtext_length: * mut uint32_t) -> sgx_status_t;


    //
    // sgx_utils.h
    //
    pub fn sgx_create_report(target_info : * const sgx_target_info_t,
                             report_data: * const sgx_report_data_t,
                             report: * mut sgx_report_t) -> sgx_status_t;
    /* intel sgx sdk 2.4 */
    pub fn sgx_self_report() -> * const sgx_report_t;
    pub fn sgx_self_target(target_info: * mut sgx_target_info_t) -> sgx_status_t;

    pub fn sgx_verify_report(report: * const sgx_report_t) -> sgx_status_t;
    pub fn sgx_get_key(key_request: * const sgx_key_request_t, key: * mut sgx_key_128bit_t) -> sgx_status_t;
}


//#[link(name = "sgx_tcrypto")]
extern {

    //
    // sgx_tcrypto.h
    //
    pub fn sgx_sha256_msg(p_src: * const uint8_t, src_len: uint32_t, p_hash: * mut sgx_sha256_hash_t) -> sgx_status_t;
    pub fn sgx_sha256_init(p_sha_handle: * mut sgx_sha_state_handle_t) -> sgx_status_t;
    pub fn sgx_sha256_update(p_src: * const uint8_t, src_len: uint32_t, sha_handle: sgx_sha_state_handle_t) -> sgx_status_t;
    pub fn sgx_sha256_get_hash(sha_handle: sgx_sha_state_handle_t, p_hash: * mut sgx_sha256_hash_t) -> sgx_status_t;
    pub fn sgx_sha256_close(sha_handle: sgx_sha_state_handle_t) -> sgx_status_t;

    /* instel sgx sdk 2.4 */
    pub fn sgx_sha1_msg(p_src: * const uint8_t, src_len: uint32_t, p_hash: * mut sgx_sha1_hash_t) -> sgx_status_t;
    pub fn sgx_sha1_init(p_sha_handle: * mut sgx_sha_state_handle_t) -> sgx_status_t;
    pub fn sgx_sha1_update(p_src: * const uint8_t, src_len: uint32_t, sha_handle: sgx_sha_state_handle_t) -> sgx_status_t;
    pub fn sgx_sha1_get_hash(sha_handle: sgx_sha_state_handle_t, p_hash: * mut sgx_sha1_hash_t) -> sgx_status_t;
    pub fn sgx_sha1_close(sha_handle: sgx_sha_state_handle_t) -> sgx_status_t;

    pub fn sgx_rijndael128GCM_encrypt(p_key: * const sgx_aes_gcm_128bit_key_t,
                                      p_src: * const uint8_t,
                                      src_len: uint32_t,
                                      p_dst: * mut uint8_t,
                                      p_iv: * const uint8_t,
                                      iv_len: uint32_t,
                                      p_aad: * const uint8_t,
                                      aad_len: uint32_t,
                                      p_out_mac: * mut sgx_aes_gcm_128bit_tag_t) -> sgx_status_t;

    pub fn sgx_rijndael128GCM_decrypt(p_key: * const sgx_aes_gcm_128bit_key_t,
                                      p_src: * const uint8_t,
                                      src_len: uint32_t,
                                      p_dst: * mut uint8_t,
                                      p_iv: * const uint8_t,
                                      iv_len: uint32_t,
                                      p_aad: * const uint8_t,
                                      aad_len: uint32_t,
                                      p_in_mac: * const sgx_aes_gcm_128bit_tag_t) -> sgx_status_t;

    pub fn sgx_rijndael128_cmac_msg(p_key: * const sgx_cmac_128bit_key_t, p_src: * const uint8_t, src_len: uint32_t, p_mac: * mut sgx_cmac_128bit_tag_t) -> sgx_status_t;
    pub fn sgx_cmac128_init(p_key: * const sgx_cmac_128bit_key_t, p_cmac_handle: * mut sgx_cmac_state_handle_t) -> sgx_status_t;
    pub fn sgx_cmac128_update(p_src: * const uint8_t, src_len: uint32_t, cmac_handle: sgx_cmac_state_handle_t) -> sgx_status_t;
    pub fn sgx_cmac128_final(cmac_handle: sgx_cmac_state_handle_t, p_hash: * mut sgx_cmac_128bit_tag_t) -> sgx_status_t;
    pub fn sgx_cmac128_close(cmac_handle: sgx_cmac_state_handle_t) -> sgx_status_t;

    /* intel sgx sdk 2.4 */
    pub fn sgx_hmac_sha256_msg(p_src: * const uint8_t,
                               src_len: int32_t,
                               p_key: * const uint8_t,
                               key_len: int32_t,
                               p_mac: * mut uint8_t,
                               mac_len: int32_t) -> sgx_status_t;

    pub fn sgx_hmac256_init(p_key: * const uint8_t, key_len: int32_t, p_hmac_handle: * mut sgx_hmac_state_handle_t) -> sgx_status_t;
    pub fn sgx_hmac256_update(p_src: * const uint8_t, src_len: int32_t, hmac_handle: sgx_hmac_state_handle_t) -> sgx_status_t;
    pub fn sgx_hmac256_final(p_hash: * mut uint8_t, hash_len: int32_t, hmac_handle: sgx_hmac_state_handle_t) -> sgx_status_t;
    pub fn sgx_hmac256_close(hmac_handle: sgx_hmac_state_handle_t) -> sgx_status_t;

    pub fn sgx_aes_ctr_encrypt(p_key: * const sgx_aes_ctr_128bit_key_t,
                               p_src: * const uint8_t,
                               src_len: uint32_t,
                               p_ctr: * const uint8_t,
                               ctr_inc_bits: uint32_t,
                               p_dst: * mut uint8_t) -> sgx_status_t;

    pub fn sgx_aes_ctr_decrypt(p_key: * const sgx_aes_ctr_128bit_key_t,
                               p_src: * const uint8_t,
                               src_len: uint32_t,
                               p_ctr: * const uint8_t,
                               ctr_inc_bits: uint32_t,
                               p_dst: * mut uint8_t) -> sgx_status_t;

    pub fn sgx_ecc256_open_context(p_ecc_handle: * mut sgx_ecc_state_handle_t) -> sgx_status_t;
    pub fn sgx_ecc256_close_context(ecc_handle: sgx_ecc_state_handle_t) -> sgx_status_t;

    pub fn sgx_ecc256_create_key_pair(p_private: * mut sgx_ec256_private_t, p_public: * mut sgx_ec256_public_t, ecc_handle: sgx_ecc_state_handle_t) -> sgx_status_t;
    pub fn sgx_ecc256_check_point(p_point: * const sgx_ec256_public_t, ecc_handle: sgx_ecc_state_handle_t, p_valid: * mut int32_t) -> sgx_status_t;

    pub fn sgx_ecc256_compute_shared_dhkey(p_private_b: * mut sgx_ec256_private_t,
                                           p_public_ga: * mut sgx_ec256_public_t,
                                           p_shared_key: * mut sgx_ec256_dh_shared_t,
                                           ecc_handle: sgx_ecc_state_handle_t) -> sgx_status_t;
    /* intel sgx sdk 1.8 */
    /* delete (intel sgx sdk 2.0)
    pub fn sgx_ecc256_compute_shared_dhkey512(p_private_b: * mut sgx_ec256_private_t,
                                              p_public_ga: * mut sgx_ec256_public_t,
                                              p_shared_key: * mut sgx_ec256_dh_shared512_t,
                                              ecc_handle: sgx_ecc_state_handle_t) -> sgx_status_t;
    */

    pub fn sgx_ecdsa_sign(p_data: * const uint8_t,
                          data_size: uint32_t,
                          p_private: * mut sgx_ec256_private_t,
                          p_signature: * mut sgx_ec256_signature_t,
                          ecc_handle: sgx_ecc_state_handle_t) -> sgx_status_t;

    pub fn sgx_ecdsa_verify(p_data: * const uint8_t,
                            data_size:  uint32_t,
                            p_public: * const sgx_ec256_public_t,
                            p_signature: * mut sgx_ec256_signature_t,
                            p_result: * mut uint8_t,
                            ecc_handle: sgx_ecc_state_handle_t) -> sgx_status_t;

    /* intel sgx sdk 2.4 */
    pub fn sgx_ecdsa_verify_hash(hash: * const uint8_t,
                                 p_public: * const sgx_ec256_public_t,
                                 p_signature: * mut sgx_ec256_signature_t,
                                 p_result: * mut uint8_t,
                                 ecc_handle: sgx_ecc_state_handle_t) -> sgx_status_t;

    /* intel sgx sdk 1.9 */
    /*
    pub fn sgx_rsa3072_sign(p_data: * const uint8_t,
                            data_size: uint32_t,
                            p_private: * const sgx_rsa3072_private_key_t,
                            p_signature: * mut sgx_rsa3072_signature_t) -> sgx_status_t;
    */

    /* intel sgx sdk 2.0 */
    pub fn sgx_rsa3072_sign(p_data: * const uint8_t,
                            data_size: uint32_t,
                            p_key: * const sgx_rsa3072_key_t,
                            p_signature: * mut sgx_rsa3072_signature_t) -> sgx_status_t;

    pub fn sgx_rsa3072_verify(p_data: * const uint8_t,
                              data_size: uint32_t,
                              p_public: * const sgx_rsa3072_public_key_t,
                              p_signature: * const sgx_rsa3072_signature_t,
                              p_result: * mut sgx_rsa_result_t) -> sgx_status_t;

    /* intel sgx sdk 2.1.3 */
    pub fn sgx_create_rsa_key_pair(n_byte_size: int32_t,
                                   e_byte_size: int32_t,
                                   p_n: * mut uint8_t,
                                   p_d: * mut uint8_t,
                                   p_e: * mut uint8_t,
                                   p_p: * mut uint8_t,
                                   p_q: * mut uint8_t,
                                   p_dmp1: * mut uint8_t,
                                   p_dmq1: * mut uint8_t,
                                   p_iqmp: * mut uint8_t) -> sgx_status_t;

    pub fn sgx_rsa_priv_decrypt_sha256(rsa_key: * const c_void,
                                       pout_data: * mut uint8_t,
                                       pout_len: * mut size_t,
                                       pin_data: * const uint8_t,
                                       pin_len: size_t) -> sgx_status_t;

    pub fn sgx_rsa_pub_encrypt_sha256(rsa_key: * const c_void,
                                      pout_data: * mut uint8_t,
                                      pout_len: * mut size_t,
                                      pin_data: * const uint8_t,
                                      pin_len: size_t) -> sgx_status_t;

    pub fn sgx_create_rsa_priv2_key(mod_size: int32_t,
                                    exp_size: int32_t,
                                    p_rsa_key_e: * const uint8_t,
                                    p_rsa_key_p: * const uint8_t,
                                    p_rsa_key_q: * const uint8_t,
                                    p_rsa_key_dmp1: * const uint8_t,
                                    p_rsa_key_dmq1: * const uint8_t,
                                    p_rsa_key_iqmp: * const uint8_t,
                                    new_pri_key2: * mut * mut c_void) -> sgx_status_t;

    pub fn sgx_create_rsa_pub1_key(mod_size: int32_t,
                                   exp_size: int32_t,
                                   le_n: * const uint8_t,
                                   le_e: * const uint8_t,
                                   new_pub_key1: * mut * mut c_void) -> sgx_status_t;

    pub fn sgx_free_rsa_key(p_rsa_key: * const c_void,
                            key_type: sgx_rsa_key_type_t,
                            mod_size: int32_t,
                            exp_size: int32_t) -> sgx_status_t;

    pub fn sgx_calculate_ecdsa_priv_key(hash_drg: * const uint8_t,
                                        hash_drg_len: int32_t,
                                        sgx_nistp256_r_m1: * const uint8_t,
                                        sgx_nistp256_r_m1_len: int32_t,
                                        out_key: * mut uint8_t,
                                        out_key_len: int32_t) -> sgx_status_t;

    /* intel sgx sdk 2.3 */
    pub fn sgx_ecc256_calculate_pub_from_priv(p_att_priv_key: * const sgx_ec256_private_t,
                                              p_att_pub_key: * mut sgx_ec256_public_t) -> sgx_status_t;


    /* intel sgx sdk 2.4 */
    pub fn sgx_aes_gcm128_enc_init(p_key: * const uint8_t,
                                   p_iv: * const uint8_t,
                                   iv_len: uint32_t,
                                   p_aad: * const uint8_t,
                                   aad_len: uint32_t,
                                   aes_gcm_state: * mut sgx_aes_state_handle_t) -> sgx_status_t;
    pub fn sgx_aes_gcm128_enc_get_mac(mac: * mut uint8_t, aes_gcm_state: sgx_aes_state_handle_t) -> sgx_status_t;
    pub fn sgx_aes_gcm_close(aes_gcm_state: sgx_aes_state_handle_t) -> sgx_status_t;
    pub fn sgx_aes_gcm128_enc_update(p_src: * const uint8_t,
                                     src_len: uint32_t,
                                     p_dst: * mut uint8_t,
                                     aes_gcm_state: sgx_aes_state_handle_t) -> sgx_status_t;
}


//#[link(name = "sgx_tkey_exchange")]
extern {

    //
    // sgx_tkey_exchange.h
    //
    pub fn sgx_ra_init(p_pub_key: * const sgx_ec256_public_t, b_pse: int32_t, p_context: * mut sgx_ra_context_t) -> sgx_status_t;

    pub fn sgx_ra_init_ex(p_pub_key: * const sgx_ec256_public_t,
                          b_pse: int32_t,
                          derive_key_cb: sgx_ra_derive_secret_keys_t,
                          p_context: * mut sgx_ra_context_t) -> sgx_status_t;

    pub fn sgx_ra_get_keys(context: sgx_ra_context_t,
                           keytype: sgx_ra_key_type_t,
                           p_key: * mut sgx_ra_key_128_t) -> sgx_status_t;

    pub fn sgx_ra_close(context: sgx_ra_context_t) -> sgx_status_t;

    pub fn sgx_ra_get_ga(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
                         context: sgx_ra_context_t, g_a: *mut sgx_ec256_public_t) -> sgx_status_t;
}


//#[link(name = "sgx_trts")]
extern {

    //
    // sgx_trts.h
    //
    pub fn sgx_is_within_enclave(addr: * const c_void, size: size_t) -> int32_t;
    pub fn sgx_is_outside_enclave(addr: * const c_void, size: size_t) -> int32_t;
    pub fn sgx_read_rand(rand: * mut u8, length_in_bytes: size_t) -> sgx_status_t;
    /* intel sgx sdk 2.1.2 */
    pub fn sgx_is_enclave_crashed() -> int32_t;


    //
    // sgx_trts_exception.h
    //
    pub fn sgx_register_exception_handler(is_first_handler: uint32_t,
                                          exception_handler: sgx_exception_handler_t) -> * const c_void;

    pub fn sgx_unregister_exception_handler(handler: * const c_void) -> uint32_t;

    //
    // sgx_edger8r.h
    //
    pub fn sgx_ocalloc(size: size_t) -> * mut c_void;
    pub fn sgx_ocfree();
}


//#[link(name = "sgx_uae_service")]
extern {

    //
    // sgx_uae_service.h
    //
    pub fn sgx_init_quote(p_target_info: * mut sgx_target_info_t, p_gid: * mut sgx_epid_group_id_t) -> sgx_status_t;

    /* intel sgx sdk 1.9 */
    pub fn sgx_calc_quote_size(p_sig_rl: * const uint8_t, sig_rl_size: uint32_t, p_quote_size: * mut uint32_t) -> sgx_status_t;
    pub fn sgx_get_quote_size(p_sig_rl: * const uint8_t, p_quote_size: * mut uint32_t) -> sgx_status_t;

    pub fn sgx_get_quote(p_report: * const sgx_report_t,
                         quote_type: sgx_quote_sign_type_t,
                         p_spid: * const sgx_spid_t,
                         p_nonce: * const sgx_quote_nonce_t,
                         p_sig_rl: * const uint8_t,
                         sig_rl_size: uint32_t,
                         p_qe_report: * mut sgx_report_t,
                         p_quote: * mut sgx_quote_t,
                         quote_size: uint32_t) -> sgx_status_t;

    pub fn sgx_get_ps_cap(p_sgx_ps_cap: * mut sgx_ps_cap_t) -> sgx_status_t;
    pub fn sgx_get_whitelist_size(p_whitelist_size: * mut uint32_t) -> sgx_status_t;
    pub fn sgx_get_whitelist(p_whitelist: * mut uint8_t, whitelist_size: uint32_t) -> sgx_status_t;
    pub fn sgx_get_extended_epid_group_id(p_extended_epid_group_id: * mut uint32_t) -> sgx_status_t;

    pub fn sgx_report_attestation_status(p_platform_info: * const sgx_platform_info_t,
                                         attestation_status: i32,
                                         p_update_info: * mut sgx_update_info_bit_t) -> sgx_status_t;

    /* intel sgx sdk 2.1 */
    pub fn sgx_register_wl_cert_chain(p_wl_cert_chain: * const uint8_t,
                                      wl_cert_chain_size: uint32_t) -> sgx_status_t;

    /* intel sgx sdk 2.5 */
    pub fn sgx_select_att_key_id(p_att_key_id_list: * const uint8_t,
                                 att_key_id_list_size: uint32_t,
                                 pp_selected_key_id: * mut * mut sgx_att_key_id_t) -> sgx_status_t;

    pub fn sgx_init_quote_ex(p_att_key_id: * const sgx_att_key_id_t,
                             p_qe_target_info: * mut sgx_target_info_t,
                             refresh_att_key: bool,
                             p_pub_key_id_size: * mut size_t,
                             p_pub_key_id: * mut uint8_t) -> sgx_status_t;

    pub fn sgx_get_quote_size_ex(p_att_key_id: * const sgx_att_key_id_t, p_quote_size: * mut uint32_t) -> sgx_status_t;

    pub fn sgx_get_quote_ex(p_app_report: * const sgx_report_t,
                            p_att_key_id: * const sgx_att_key_id_t,
                            p_qe_report_info: * mut sgx_qe_report_info_t,
                            p_quote: * mut uint8_t,
                            quote_size: uint32_t) -> sgx_status_t;
}


//#[link(name = "sgx_ukey_exchange")]
extern {

    //
    // sgx_ukey_exchange.h
    //
    pub fn sgx_ra_get_msg1(context: sgx_ra_context_t,
                           eid: sgx_enclave_id_t,
                           p_get_ga: sgx_ecall_get_ga_trusted_t,
                           p_msg1: * mut sgx_ra_msg1_t) -> sgx_status_t;

    pub fn sgx_ra_proc_msg2(context: sgx_ra_context_t,
                            eid: sgx_enclave_id_t,
                            p_proc_msg2: sgx_ecall_proc_msg2_trusted_t,
                            p_get_msg3: sgx_ecall_get_msg3_trusted_t,
                            p_msg2: * const sgx_ra_msg2_t,
                            msg2_size: uint32_t,
                            pp_msg3: * mut * mut sgx_ra_msg3_t,
                            p_msg3_size: * mut uint32_t) -> sgx_status_t;

    /* intel sgx sdk 2.5 */
    pub fn sgx_ra_get_msg1_ex(p_att_key_id: * const sgx_att_key_id_t,
                              context: sgx_ra_context_t,
                              eid: sgx_enclave_id_t,
                              p_get_ga: sgx_ecall_get_ga_trusted_t,
                              p_msg1: * mut sgx_ra_msg1_t) -> sgx_status_t;

    pub fn sgx_ra_proc_msg2_ex(p_att_key_id: * const sgx_att_key_id_t,
                               context: sgx_ra_context_t,
                               eid: sgx_enclave_id_t,
                               p_proc_msg2: sgx_ecall_proc_msg2_trusted_t,
                               p_get_msg3: sgx_ecall_get_msg3_trusted_t,
                               p_msg2: * const sgx_ra_msg2_t,
                               msg2_size: uint32_t,
                               pp_msg3: * mut * mut sgx_ra_msg3_t,
                               p_msg3_size: * mut uint32_t) -> sgx_status_t;
}


//#[link(name = "sgx_urts")]
extern {

    //
    // sgx_urts.h
    //
    pub fn sgx_create_enclave(file_name: * const c_char,
                              debug: int32_t,
                              launch_token: * mut sgx_launch_token_t,
                              launch_token_updated: * mut int32_t,
                              enclave_id: * mut sgx_enclave_id_t,
                              misc_attr: * mut sgx_misc_attribute_t) -> sgx_status_t;

    /* intel sgx sdk 2.1.3 */
    pub fn sgx_create_encrypted_enclave(file_name: * const c_char,
                                        debug: int32_t,
                                        launch_token: * mut sgx_launch_token_t,
                                        launch_token_updated: * mut int32_t,
                                        enclave_id: * mut sgx_enclave_id_t,
                                        misc_attr: * mut sgx_misc_attribute_t,
                                        sealed_key: * const uint8_t) -> sgx_status_t;

    /* intel sgx sdk 2.2 */
    pub fn sgx_create_enclave_ex(file_name: * const c_char,
                                 debug: int32_t,
                                 launch_token: * mut sgx_launch_token_t,
                                 launch_token_updated: * mut int32_t,
                                 enclave_id: * mut sgx_enclave_id_t,
                                 misc_attr: * mut sgx_misc_attribute_t,
                                 ex_features: uint32_t,
                                 ex_features_p: * const [* const c_void; 32]) -> sgx_status_t;

    /* intel sgx sdk 2.4 */
    pub fn sgx_create_enclave_from_buffer_ex(buffer: * const uint8_t,
                                             buffer_size: size_t,
                                             debug: int32_t,
                                             enclave_id: * mut sgx_enclave_id_t,
                                             misc_attr: * mut sgx_misc_attribute_t,
                                             ex_features: uint32_t,
                                             ex_features_p: * const [* const c_void; 32]) -> sgx_status_t;

    pub fn sgx_destroy_enclave(enclave_id: sgx_enclave_id_t) -> sgx_status_t;

    /* intel sgx sdk 2.4 */
    pub fn sgx_get_target_info(enclave_id: sgx_enclave_id_t, target_info: * mut sgx_target_info_t) -> sgx_status_t;
}

/* intel sgx sdk 1.9 */
//#[link(name = "sgx_tprotected_fs")]
extern {

    //
    // sgx_tprotected_fs.h
    //
    pub fn sgx_fopen(filename: * const c_char,
                     mode: * const c_char,
                     key: * const sgx_key_128bit_t) -> SGX_FILE;

    pub fn sgx_fopen_auto_key(filename: * const c_char, mode: * const c_char) -> SGX_FILE;

    pub fn sgx_fwrite(ptr: * const c_void,
                      size: size_t,
                      count: size_t,
                      stream: SGX_FILE) -> size_t;

    pub fn sgx_fread(ptr: * mut c_void,
                     size: size_t,
                     count: size_t,
                     stream: SGX_FILE) -> size_t;

    pub fn sgx_ftell(stream: SGX_FILE) -> int64_t;

    pub fn sgx_fseek(stream: SGX_FILE, offset: int64_t, origin: c_int) -> int32_t;

    pub fn sgx_fflush(stream: SGX_FILE) -> int32_t;

    pub fn sgx_ferror(stream: SGX_FILE) -> int32_t;

    pub fn sgx_feof(stream: SGX_FILE) -> int32_t;

    pub fn sgx_clearerr(stream: SGX_FILE);

    pub fn sgx_fclose(stream: SGX_FILE) -> int32_t;

    pub fn sgx_remove(filename: * const c_char) -> int32_t;

    pub fn sgx_fexport_auto_key(filename: * const c_char, key: * mut sgx_key_128bit_t) -> int32_t;

    pub fn sgx_fimport_auto_key(filename: * const c_char, key: * const sgx_key_128bit_t) -> int32_t;

    pub fn sgx_fclear_cache(stream: SGX_FILE) -> int32_t;
}

/* intel sgx sdk 2.0 */
//#[link(name = "sgx_capable")]
extern {

    pub fn sgx_is_capable(sgx_capable: * mut int32_t) -> sgx_status_t;
    pub fn sgx_cap_enable_device(sgx_device_status: * mut sgx_device_status_t) -> sgx_status_t;
    pub fn sgx_cap_get_status(sgx_device_status: * mut sgx_device_status_t) -> sgx_status_t;
}
