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

//! # Cryptography Library
//!
//! The Intel(R) Software Guard Extensions SDK includes a trusted cryptography library named sgx_tcrypto.
//! It includes the cryptographic functions used by other trusted libraries included in the SDK
//!

#![no_std]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]

extern crate sgx_types;

pub use self::bindings::*;

mod bindings {
    use sgx_types::error::SgxStatus;
    use sgx_types::types::*;

    extern "C" {
        pub fn sgx_init_crypto_lib(cpu_features: u64, cpuid_table: *const u32) -> SgxStatus;
    }

    extern "C" {
        //
        // sgx_tcrypto.h
        //
        /* instel sgx sdk 2.16 */
        pub fn sgx_sha384_msg(p_src: *const u8, src_len: u32, p_hash: *mut Sha384Hash)
            -> SgxStatus;
        pub fn sgx_sha384_init(p_sha_handle: *mut ShaHandle) -> SgxStatus;
        pub fn sgx_sha384_update(
            p_src: *const u8,
            src_len: u32,
            sha_handle: ShaHandle,
        ) -> SgxStatus;
        pub fn sgx_sha384_get_hash(sha_handle: ShaHandle, p_hash: *mut Sha384Hash) -> SgxStatus;
        pub fn sgx_sha384_close(sha_handle: ShaHandle) -> SgxStatus;

        pub fn sgx_sha256_msg(p_src: *const u8, src_len: u32, p_hash: *mut Sha256Hash)
            -> SgxStatus;
        pub fn sgx_sha256_init(p_sha_handle: *mut ShaHandle) -> SgxStatus;
        pub fn sgx_sha256_update(
            p_src: *const u8,
            src_len: u32,
            sha_handle: ShaHandle,
        ) -> SgxStatus;
        pub fn sgx_sha256_get_hash(sha_handle: ShaHandle, p_hash: *mut Sha256Hash) -> SgxStatus;
        pub fn sgx_sha256_close(sha_handle: ShaHandle) -> SgxStatus;

        /* instel sgx sdk 2.4 */
        pub fn sgx_sha1_msg(p_src: *const u8, src_len: u32, p_hash: *mut Sha1Hash) -> SgxStatus;
        pub fn sgx_sha1_init(p_sha_handle: *mut ShaHandle) -> SgxStatus;
        pub fn sgx_sha1_update(p_src: *const u8, src_len: u32, sha_handle: ShaHandle) -> SgxStatus;
        pub fn sgx_sha1_get_hash(sha_handle: ShaHandle, p_hash: *mut Sha1Hash) -> SgxStatus;
        pub fn sgx_sha1_close(sha_handle: ShaHandle) -> SgxStatus;

        pub fn sgx_rijndael128GCM_encrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            p_out_mac: *mut Mac128bit,
        ) -> SgxStatus;

        pub fn sgx_rijndael128GCM_decrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            p_in_mac: *const Mac128bit,
        ) -> SgxStatus;

        pub fn sgx_rijndael128_cmac_msg(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_mac: *mut Mac128bit,
        ) -> SgxStatus;

        pub fn sgx_cmac128_init(
            p_key: *const Key128bit,
            p_cmac_handle: *mut CMacHandle,
        ) -> SgxStatus;

        pub fn sgx_cmac128_update(
            p_src: *const u8,
            src_len: u32,
            cmac_handle: CMacHandle,
        ) -> SgxStatus;

        pub fn sgx_cmac128_final(cmac_handle: CMacHandle, p_hash: *mut Mac128bit) -> SgxStatus;

        pub fn sgx_cmac128_close(cmac_handle: CMacHandle) -> SgxStatus;

        /* intel sgx sdk 2.4 */
        pub fn sgx_hmac_sha256_msg(
            p_src: *const u8,
            src_len: i32,
            p_key: *const u8,
            key_len: i32,
            p_mac: *mut u8,
            mac_len: i32,
        ) -> SgxStatus;

        pub fn sgx_hmac_sha256_init(
            p_key: *const u8,
            key_len: i32,
            p_hmac_handle: *mut HMacHandle,
        ) -> SgxStatus;

        pub fn sgx_hmac_sha256_update(
            p_src: *const u8,
            src_len: i32,
            hmac_handle: HMacHandle,
        ) -> SgxStatus;

        pub fn sgx_hmac_sha256_final(
            p_hash: *mut u8,
            hash_len: i32,
            hmac_handle: HMacHandle,
        ) -> SgxStatus;

        pub fn sgx_hmac_sha256_close(hmac_handle: HMacHandle) -> SgxStatus;

        pub fn sgx_aes_ctr_encrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_ctr: *mut u8,
            ctr_inc_bits: u32,
            p_dst: *mut u8,
        ) -> SgxStatus;

        pub fn sgx_aes_ctr_decrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_ctr: *mut u8,
            ctr_inc_bits: u32,
            p_dst: *mut u8,
        ) -> SgxStatus;

        pub fn sgx_ecc256_open_context(p_ecc_handle: *mut EccHandle) -> SgxStatus;
        pub fn sgx_ecc256_close_context(ecc_handle: EccHandle) -> SgxStatus;

        pub fn sgx_ecc256_create_key_pair(
            p_private: *mut Ec256PrivateKey,
            p_public: *mut Ec256PublicKey,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        pub fn sgx_ecc256_check_point(
            p_point: *const Ec256PublicKey,
            ecc_handle: EccHandle,
            p_valid: *mut i32,
        ) -> SgxStatus;

        pub fn sgx_ecc256_compute_shared_dhkey(
            p_private_b: *const Ec256PrivateKey,
            p_public_ga: *const Ec256PublicKey,
            p_shared_key: *mut Ec256SharedKey,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        /* intel sgx sdk 1.8 */
        // delete (intel sgx sdk 2.0)
        // pub fn sgx_ecc256_compute_shared_dhkey512(
        //     p_private_b: *mut Ec256PrivateKey,
        //     p_public_ga: *mut Ec256PublicKey,
        //     p_shared_key: *mut Ec256DhShared512Key,
        //     ecc_handle: EccHandle,
        // ) -> SgxStatus;

        pub fn sgx_ecdsa_sign(
            p_data: *const u8,
            data_size: u32,
            p_private: *const Ec256PrivateKey,
            p_signature: *mut Ec256Signature,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        pub fn sgx_ecdsa_verify(
            p_data: *const u8,
            data_size: u32,
            p_public: *const Ec256PublicKey,
            p_signature: *const Ec256Signature,
            p_result: *mut u8,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        /* intel sgx sdk 2.4 */
        pub fn sgx_ecdsa_verify_hash(
            hash: *const u8,
            p_public: *const Ec256PublicKey,
            p_signature: *const Ec256Signature,
            p_result: *mut u8,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        pub fn sgx_calculate_ecdsa_priv_key(
            hash_drg: *const u8,
            hash_drg_len: i32,
            sgx_nistp256_r_m1: *const u8,
            sgx_nistp256_r_m1_len: i32,
            out_key: *mut u8,
            out_key_len: i32,
        ) -> SgxStatus;

        /* intel sgx sdk 2.3 */
        pub fn sgx_ecc256_calculate_pub_from_priv(
            p_att_priv_key: *const Ec256PrivateKey,
            p_att_pub_key: *mut Ec256PublicKey,
        ) -> SgxStatus;

        pub fn sgx_rsa2048_sign(
            p_data: *const u8,
            data_size: u32,
            p_key: *const Rsa2048Key,
            p_signature: *mut Rsa2048Signature,
        ) -> SgxStatus;

        pub fn sgx_rsa2048_sign_ex(
            p_data: *const u8,
            data_size: u32,
            p_key: *const Rsa2048Key,
            p_public: *const Rsa2048PubKey,
            p_signature: *mut Rsa2048Signature,
        ) -> SgxStatus;

        pub fn sgx_rsa2048_verify(
            p_data: *const u8,
            data_size: u32,
            p_public: *const Rsa2048PubKey,
            p_signature: *const Rsa2048Signature,
            p_result: *mut RsaResult,
        ) -> SgxStatus;

        /* intel sgx sdk 2.0 */
        pub fn sgx_rsa3072_sign(
            p_data: *const u8,
            data_size: u32,
            p_key: *const Rsa3072Key,
            p_signature: *mut Rsa3072Signature,
        ) -> SgxStatus;

        /* intel sgx sdk 2.15 */
        pub fn sgx_rsa3072_sign_ex(
            p_data: *const u8,
            data_size: u32,
            p_key: *const Rsa3072Key,
            p_public: *const Rsa3072PubKey,
            p_signature: *mut Rsa3072Signature,
        ) -> SgxStatus;

        pub fn sgx_rsa3072_verify(
            p_data: *const u8,
            data_size: u32,
            p_public: *const Rsa3072PubKey,
            p_signature: *const Rsa3072Signature,
            p_result: *mut RsaResult,
        ) -> SgxStatus;

        /* intel sgx sdk 2.1.3 */
        pub fn sgx_create_rsa_key_pair(
            n_byte_size: i32,
            e_byte_size: i32,
            p_n: *mut u8,
            p_d: *mut u8,
            p_e: *mut u8,
            p_p: *mut u8,
            p_q: *mut u8,
            p_dmp1: *mut u8,
            p_dmq1: *mut u8,
            p_iqmp: *mut u8,
        ) -> SgxStatus;

        pub fn sgx_rsa_priv_decrypt_sha256(
            rsa_key: *const c_void,
            pout_data: *mut u8,
            pout_len: *mut usize,
            pin_data: *const u8,
            pin_len: usize,
        ) -> SgxStatus;

        pub fn sgx_rsa_pub_encrypt_sha256(
            rsa_key: *const c_void,
            pout_data: *mut u8,
            pout_len: *mut usize,
            pin_data: *const u8,
            pin_len: usize,
        ) -> SgxStatus;

        pub fn sgx_create_rsa_priv2_key(
            mod_size: i32,
            exp_size: i32,
            p_rsa_key_e: *const u8,
            p_rsa_key_p: *const u8,
            p_rsa_key_q: *const u8,
            p_rsa_key_dmp1: *const u8,
            p_rsa_key_dmq1: *const u8,
            p_rsa_key_iqmp: *const u8,
            new_pri_key2: *mut *mut c_void,
        ) -> SgxStatus;

        /* intel sgx sdk 2.6 */
        pub fn sgx_create_rsa_priv1_key(
            n_byte_size: i32,
            e_byte_size: i32,
            d_byte_size: i32,
            le_n: *const u8,
            le_e: *const u8,
            le_d: *const u8,
            new_pri_key1: *mut *mut c_void,
        ) -> SgxStatus;

        pub fn sgx_create_rsa_pub1_key(
            mod_size: i32,
            exp_size: i32,
            le_n: *const u8,
            le_e: *const u8,
            new_pub_key1: *mut *mut c_void,
        ) -> SgxStatus;

        pub fn sgx_free_rsa_key(
            p_rsa_key: *const c_void,
            key_type: RsaKeyType,
            mod_size: i32,
            exp_size: i32,
        ) -> SgxStatus;

        /* intel sgx sdk 2.4 */
        pub fn sgx_aes_gcm128_init(
            p_key: *const u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            aes_gcm_state: *mut AesHandle,
        ) -> SgxStatus;

        pub fn sgx_aes_gcm128_enc_update(
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            aes_gcm_state: AesHandle,
        ) -> SgxStatus;

        pub fn sgx_aes_gcm128_dec_update(
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            aes_gcm_state: AesHandle,
        ) -> SgxStatus;

        pub fn sgx_aes_gcm128_enc_get_mac(mac: *mut u8, aes_gcm_state: AesHandle) -> SgxStatus;

        pub fn sgx_aes_gcm128_dec_verify_mac(mac: *const u8, aes_gcm_state: AesHandle)
            -> SgxStatus;

        pub fn sgx_aes_gcm_close(aes_gcm_state: AesHandle) -> SgxStatus;

        pub fn sgx_aes_ccm128_encrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            p_out_mac: *mut Mac128bit,
        ) -> SgxStatus;

        pub fn sgx_aes_ccm128_decrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            p_in_mac: *const Mac128bit,
        ) -> SgxStatus;

        pub fn sgx_aes_ccm128_init(
            p_key: *const u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            aes_ccm_state: *mut AesHandle,
        ) -> SgxStatus;

        pub fn sgx_aes_ccm128_enc_update(
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            aes_ccm_state: AesHandle,
        ) -> SgxStatus;

        pub fn sgx_aes_ccm128_dec_update(
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            aes_ccm_state: AesHandle,
        ) -> SgxStatus;

        pub fn sgx_aes_ccm128_enc_get_mac(mac: *mut u8, aes_ccm_state: AesHandle) -> SgxStatus;

        pub fn sgx_aes_ccm128_dec_verify_mac(mac: *const u8, aes_ccm_state: AesHandle)
            -> SgxStatus;

        pub fn sgx_aes_ccm_close(aes_ccm_state: AesHandle) -> SgxStatus;

        pub fn sgx_aes_cbc_encrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
        ) -> SgxStatus;

        pub fn sgx_aes_cbc_decrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
        ) -> SgxStatus;

        pub fn sgx_sm3_msg(p_src: *const u8, src_len: u32, p_hash: *mut Sm3Hash) -> SgxStatus;
        pub fn sgx_sm3_init(p_sm3_handle: *mut Sm3Handle) -> SgxStatus;
        pub fn sgx_sm3_update(p_src: *const u8, src_len: u32, sm3_handle: Sm3Handle) -> SgxStatus;
        pub fn sgx_sm3_get_hash(sm3_handle: Sm3Handle, p_hash: *mut Sm3Hash) -> SgxStatus;
        pub fn sgx_sm3_close(sm3_handle: Sm3Handle) -> SgxStatus;

        pub fn sgx_sm4_ccm128_encrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            p_out_mac: *mut Mac128bit,
        ) -> SgxStatus;

        pub fn sgx_sm4_ccm128_decrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            p_in_mac: *const Mac128bit,
        ) -> SgxStatus;

        pub fn sgx_sm4_ccm128_init(
            p_key: *const u8,
            p_iv: *const u8,
            iv_len: u32,
            p_aad: *const u8,
            aad_len: u32,
            sm4_ccm_state: *mut Sm4Handle,
        ) -> SgxStatus;

        pub fn sgx_sm4_ccm128_enc_update(
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            sm4_ccm_state: Sm4Handle,
        ) -> SgxStatus;

        pub fn sgx_sm4_ccm128_dec_update(
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            sm4_ccm_state: Sm4Handle,
        ) -> SgxStatus;

        pub fn sgx_sm4_ccm128_enc_get_mac(mac: *mut u8, sm4_ccm_state: Sm4Handle) -> SgxStatus;

        pub fn sgx_sm4_ccm128_dec_verify_mac(mac: *const u8, sm4_ccm_state: Sm4Handle)
            -> SgxStatus;

        pub fn sgx_sm4_ccm_close(sm4_ccm_state: Sm4Handle) -> SgxStatus;

        pub fn sgx_sm4_cbc_encrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
        ) -> SgxStatus;

        pub fn sgx_sm4_cbc_decrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_dst: *mut u8,
            p_iv: *const u8,
            iv_len: u32,
        ) -> SgxStatus;

        pub fn sgx_sm4_ctr_encrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_ctr: *mut u8,
            ctr_inc_bits: u32,
            p_dst: *mut u8,
        ) -> SgxStatus;

        pub fn sgx_sm4_ctr_decrypt(
            p_key: *const Key128bit,
            p_src: *const u8,
            src_len: u32,
            p_ctr: *mut u8,
            ctr_inc_bits: u32,
            p_dst: *mut u8,
        ) -> SgxStatus;

        pub fn sgx_hmac_sm3_msg(
            p_src: *const u8,
            src_len: i32,
            p_key: *const u8,
            key_len: i32,
            p_mac: *mut u8,
            mac_len: i32,
        ) -> SgxStatus;

        pub fn sgx_hmac_sm3_init(
            p_key: *const u8,
            key_len: i32,
            p_hmac_handle: *mut HMacHandle,
        ) -> SgxStatus;

        pub fn sgx_hmac_sm3_update(
            p_src: *const u8,
            src_len: i32,
            hmac_handle: HMacHandle,
        ) -> SgxStatus;

        pub fn sgx_hmac_sm3_final(
            p_hash: *mut u8,
            hash_len: i32,
            hmac_handle: HMacHandle,
        ) -> SgxStatus;

        pub fn sgx_hmac_sm3_close(hmac_handle: HMacHandle) -> SgxStatus;

        pub fn sgx_sm2_open_context(p_ecc_handle: *mut EccHandle) -> SgxStatus;
        pub fn sgx_sm2_close_context(ecc_handle: EccHandle) -> SgxStatus;

        pub fn sgx_sm2_create_key_pair(
            p_private: *mut Ec256PrivateKey,
            p_public: *mut Ec256PublicKey,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        pub fn sgx_sm2_check_point(
            p_point: *const Ec256PublicKey,
            ecc_handle: EccHandle,
            p_valid: *mut i32,
        ) -> SgxStatus;

        pub fn sgx_sm2_compute_shared_dhkey(
            p_private_b: *const Ec256PrivateKey,
            p_public_ga: *const Ec256PublicKey,
            p_shared_key: *mut Ec256SharedKey,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        pub fn sgx_sm2_sign(
            p_data: *const u8,
            data_size: u32,
            p_private: *const Ec256PrivateKey,
            p_signature: *mut Ec256Signature,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        pub fn sgx_sm2_verify(
            p_data: *const u8,
            data_size: u32,
            p_public: *const Ec256PublicKey,
            p_signature: *const Ec256Signature,
            p_result: *mut u8,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        pub fn sgx_sm2_verify_hash(
            hash: *const u8,
            p_public: *const Ec256PublicKey,
            p_signature: *const Ec256Signature,
            p_result: *mut u8,
            ecc_handle: EccHandle,
        ) -> SgxStatus;

        pub fn sgx_calculate_sm2_priv_key(
            hash_drg: *const u8,
            hash_drg_len: i32,
            sgx_sm2_order: *const u8,
            sgx_sm2_order_len: i32,
            out_key: *mut u8,
            out_key_len: i32,
        ) -> SgxStatus;

        pub fn sgx_sm2_calculate_pub_from_priv(
            p_att_priv_key: *const Ec256PrivateKey,
            p_att_pub_key: *mut Ec256PublicKey,
        ) -> SgxStatus;
    }
}
