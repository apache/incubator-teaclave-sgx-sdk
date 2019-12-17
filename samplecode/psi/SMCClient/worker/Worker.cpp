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

#include "Worker.h"
#include "sample_libcrypto.h"
#include "../GeneralSettings.h"
#include "sha256.h"

// This is the private EC key of SP, the corresponding public EC key is
// hard coded in isv_enclave. It is based on NIST P-256 curve.
static const sample_ec256_private_t g_sp_priv_key = {
    {
        0x90, 0xe7, 0x6c, 0xbb, 0x2d, 0x52, 0xa1, 0xce,
        0x3b, 0x66, 0xde, 0x11, 0x43, 0x9c, 0x87, 0xec,
        0x1f, 0x86, 0x6a, 0x3b, 0x65, 0xb6, 0xae, 0xea,
        0xad, 0x57, 0x34, 0x53, 0xd1, 0x03, 0x8c, 0x01
    }
};

PSIWorker::PSIWorker(WebService *ws) : ws(ws) {}

PSIWorker::~PSIWorker() {}


int PSIWorker::sp_ra_proc_msg0_req(const uint32_t id) {
    int ret = -1;

    if (!this->g_is_sp_registered || (this->extended_epid_group_id != id)) {
        Log("Received extended EPID group ID: %d", id);

        extended_epid_group_id = id;
        this->g_is_sp_registered = true;
        ret = SP_OK;
    }

    return ret;
}


int PSIWorker::sp_ra_proc_msg1_req(Messages::MessageMSG1 msg1, Messages::MessageMSG2 *msg2) {
    int ret = 0;
    ra_samp_response_header_t* p_msg2_full = NULL;
    sgx_ra_msg2_t *p_msg2 = NULL;
    sample_ecc_state_handle_t ecc_state = NULL;
    sample_status_t sample_ret = SAMPLE_SUCCESS;
    bool derive_ret = false;

    if (!g_is_sp_registered) {
        return SP_UNSUPPORTED_EXTENDED_EPID_GROUP;
    }

    do {
        //=====================  RETRIEVE SIGRL FROM IAS =======================
        uint8_t GID[4];

        for (int i=0; i<4; i++)
            GID[i] = msg1.gid(i);

        reverse(begin(GID), end(GID));

        string sigRl;
        bool error = false;
        error = this->ws->getSigRL(ByteArrayToString(GID, 4), &sigRl);

        if (error)
            return SP_RETRIEVE_SIGRL_ERROR;

        uint8_t *sig_rl;
        uint32_t sig_rl_size = StringToByteArray(sigRl, &sig_rl);
        //=====================================================================

        uint8_t gaXLittleEndian[32];
        uint8_t gaYLittleEndian[32];

        for (int i=0; i<32; i++) {
            gaXLittleEndian[i] = msg1.gax(i);
            gaYLittleEndian[i] = msg1.gay(i);
        }

        sample_ec256_public_t client_pub_key = {{0},{0}};

        for (int x=0; x<DH_SHARED_KEY_LEN; x++) {
            client_pub_key.gx[x] = gaXLittleEndian[x];
            client_pub_key.gy[x] = gaYLittleEndian[x];
        }

        // Need to save the client's public ECCDH key to local storage
        if (memcpy_s(&g_sp_db.g_a, sizeof(g_sp_db.g_a), &client_pub_key, sizeof(client_pub_key))) {
            Log("Error, cannot do memcpy", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        // Generate the Service providers ECCDH key pair.
        sample_ret = sample_ecc256_open_context(&ecc_state);
        if(SAMPLE_SUCCESS != sample_ret) {
            Log("Error, cannot get ECC context", log::error);
            ret = -1;
            break;
        }


        sample_ec256_public_t pub_key = {{0},{0}};
        sample_ec256_private_t priv_key = {{0}};
        sample_ret = sample_ecc256_create_key_pair(&priv_key, &pub_key, ecc_state);

        if (SAMPLE_SUCCESS != sample_ret) {
            Log("Error, cannot get key pair", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        // Need to save the SP ECCDH key pair to local storage.
        if (memcpy_s(&g_sp_db.b, sizeof(g_sp_db.b), &priv_key,sizeof(priv_key)) ||
                memcpy_s(&g_sp_db.g_b, sizeof(g_sp_db.g_b), &pub_key,sizeof(pub_key))) {
            Log("Error, cannot do memcpy", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        // Generate the client/SP shared secret
        sample_ec_dh_shared_t dh_key = {{0}};

        sample_ret = sample_ecc256_compute_shared_dhkey(&priv_key, (sample_ec256_public_t *)&client_pub_key,
                     (sample_ec256_dh_shared_t *)&dh_key,
                     ecc_state);

        if (SAMPLE_SUCCESS != sample_ret) {
            Log("Error, compute share key fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }


        // smk is only needed for msg2 generation.
        derive_ret = derive_key(&dh_key, SAMPLE_DERIVE_KEY_SMK, &g_sp_db.smk_key);
        if (derive_ret != true) {
            Log("Error, derive key fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        // The rest of the keys are the shared secrets for future communication.
        derive_ret = derive_key(&dh_key, SAMPLE_DERIVE_KEY_MK, &g_sp_db.mk_key);
        if (derive_ret != true) {
            Log("Error, derive key fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        derive_ret = derive_key(&dh_key, SAMPLE_DERIVE_KEY_SK, &g_sp_db.sk_key);
        if (derive_ret != true) {
            Log("Error, derive key fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        derive_ret = derive_key(&dh_key, SAMPLE_DERIVE_KEY_VK, &g_sp_db.vk_key);
        if (derive_ret != true) {
            Log("Error, derive key fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }


        uint32_t msg2_size = sizeof(sgx_ra_msg2_t) + sig_rl_size;
        p_msg2_full = (ra_samp_response_header_t*)malloc(msg2_size + sizeof(ra_samp_response_header_t));

        if (!p_msg2_full) {
            Log("Error, Error, out of memory", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        memset(p_msg2_full, 0, msg2_size + sizeof(ra_samp_response_header_t));
        p_msg2_full->type = RA_MSG2;
        p_msg2_full->size = msg2_size;

        p_msg2_full->status[0] = 0;
        p_msg2_full->status[1] = 0;
        p_msg2 = (sgx_ra_msg2_t *) p_msg2_full->body;


        uint8_t *spidBa;
        HexStringToByteArray(Settings::spid, &spidBa);

        for (int i=0; i<16; i++)
            p_msg2->spid.id[i] = spidBa[i];


        // Assemble MSG2
        if(memcpy_s(&p_msg2->g_b, sizeof(p_msg2->g_b), &g_sp_db.g_b, sizeof(g_sp_db.g_b))) {
            Log("Error, memcpy failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        p_msg2->quote_type = SAMPLE_QUOTE_LINKABLE_SIGNATURE;
        p_msg2->kdf_id = AES_CMAC_KDF_ID;

        // Create gb_ga
        sgx_ec256_public_t gb_ga[2];
        if (memcpy_s(&gb_ga[0], sizeof(gb_ga[0]), &g_sp_db.g_b, sizeof(g_sp_db.g_b)) ||
                memcpy_s(&gb_ga[1], sizeof(gb_ga[1]), &g_sp_db.g_a, sizeof(g_sp_db.g_a))) {
            Log("Error, memcpy failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        // Sign gb_ga
        sample_ret = sample_ecdsa_sign((uint8_t *)&gb_ga, sizeof(gb_ga),
                                       (sample_ec256_private_t *)&g_sp_priv_key,
                                       (sample_ec256_signature_t *)&p_msg2->sign_gb_ga,
                                       ecc_state);

        if (SAMPLE_SUCCESS != sample_ret) {
            Log("Error, sign ga_gb fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }


        // Generate the CMACsmk for gb||SPID||TYPE||KDF_ID||Sigsp(gb,ga)
        uint8_t mac[SAMPLE_EC_MAC_SIZE] = {0};
        uint32_t cmac_size = offsetof(sgx_ra_msg2_t, mac);
        sample_ret = sample_rijndael128_cmac_msg(&g_sp_db.smk_key, (uint8_t *)&p_msg2->g_b, cmac_size, &mac);

        if (SAMPLE_SUCCESS != sample_ret) {
            Log("Error, cmac fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        if (memcpy_s(&p_msg2->mac, sizeof(p_msg2->mac), mac, sizeof(mac))) {
            Log("Error, memcpy failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        if (memcpy_s(&p_msg2->sig_rl[0], sig_rl_size, sig_rl, sig_rl_size)) {
            Log("Error, memcpy failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        p_msg2->sig_rl_size = sig_rl_size;

    } while(0);


    if (ret) {
        SafeFree(p_msg2_full);
    } else {

        //=================   SET MSG2 Fields   ================
        msg2->set_size(p_msg2_full->size);

        for (auto x : p_msg2->g_b.gx)
            msg2->add_public_key_gx(x);

        for (auto x : p_msg2->g_b.gy)
            msg2->add_public_key_gy(x);

        for (auto x : p_msg2->spid.id)
            msg2->add_spid(x);

        msg2->set_quote_type(SAMPLE_QUOTE_LINKABLE_SIGNATURE);
        msg2->set_cmac_kdf_id(AES_CMAC_KDF_ID);

        for (auto x : p_msg2->sign_gb_ga.x) {
            msg2->add_signature_x(x);
        }

        for (auto x : p_msg2->sign_gb_ga.y)
            msg2->add_signature_y(x);

        for (auto x : p_msg2->mac)
            msg2->add_smac(x);

        msg2->set_size_sigrl(p_msg2->sig_rl_size);

        for (int i=0; i<p_msg2->sig_rl_size; i++)
            msg2->add_sigrl(p_msg2->sig_rl[i]);
        //=====================================================
    }

    if (ecc_state) {
        sample_ecc256_close_context(ecc_state);
    }

    return ret;
}


sgx_ra_msg3_t* PSIWorker::assembleMSG3(Messages::MessageMSG3 msg) {
    sgx_ra_msg3_t *p_msg3 = (sgx_ra_msg3_t*) malloc(msg.size());

    for (int i=0; i<SGX_MAC_SIZE; i++)
        p_msg3->mac[i] = msg.sgx_mac(i);

    for (int i=0; i<SGX_ECP256_KEY_SIZE; i++) {
        p_msg3->g_a.gx[i] = msg.gax_msg3(i);
        p_msg3->g_a.gy[i] = msg.gay_msg3(i);
    }

    for (int i=0; i<256; i++)
        p_msg3->ps_sec_prop.sgx_ps_sec_prop_desc[i] = msg.sec_property(i);

    for (int i=0; i<1116; i++)
        p_msg3->quote[i] = msg.quote(i);

    return p_msg3;
}



// Process remote attestation message 3
int PSIWorker::sp_ra_proc_msg3_req(Messages::MessageMSG3 msg, Messages::AttestationMessage *att_msg) {
    int ret = 0;
    sample_status_t sample_ret = SAMPLE_SUCCESS;
    const uint8_t *p_msg3_cmaced = NULL;
    sgx_quote_t *p_quote = NULL;
    sample_sha_state_handle_t sha_handle = NULL;
    sample_report_data_t report_data = {0};
    sample_ra_att_result_msg_t *p_att_result_msg = NULL;
    ra_samp_response_header_t* p_att_result_msg_full = NULL;
    uint32_t i;
    sgx_ra_msg3_t *p_msg3 = NULL;
    uint32_t att_result_msg_size;
    int len_hmac_nonce = 0;

    p_msg3 = assembleMSG3(msg);

    // Check to see if we have registered?
    if (!g_is_sp_registered) {
        Log("Unsupported extended EPID group", log::error);
        return -1;
    }

    do {
        // Compare g_a in message 3 with local g_a.
        if (memcmp(&g_sp_db.g_a, &p_msg3->g_a, sizeof(sgx_ec256_public_t))) {
            Log("Error, g_a is not same", log::error);
            ret = SP_PROTOCOL_ERROR;
            break;
        }

        //Make sure that msg3_size is bigger than sample_mac_t.
        uint32_t mac_size = msg.size() - sizeof(sample_mac_t);
        p_msg3_cmaced = reinterpret_cast<const uint8_t*>(p_msg3);
        p_msg3_cmaced += sizeof(sample_mac_t);

        // Verify the message mac using SMK
        sample_cmac_128bit_tag_t mac = {0};
        sample_ret = sample_rijndael128_cmac_msg(&g_sp_db.smk_key, p_msg3_cmaced, mac_size, &mac);

        if (SAMPLE_SUCCESS != sample_ret) {
            Log("Error, cmac fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        if (memcmp(&p_msg3->mac, mac, sizeof(mac))) {
            Log("Error, verify cmac fail", log::error);
            ret = SP_INTEGRITY_FAILED;
            break;
        }

        if (memcpy_s(&g_sp_db.ps_sec_prop, sizeof(g_sp_db.ps_sec_prop), &p_msg3->ps_sec_prop, sizeof(p_msg3->ps_sec_prop))) {
            Log("Error, memcpy fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        p_quote = (sgx_quote_t *) p_msg3->quote;


        // Verify the report_data in the Quote matches the expected value.
        // The first 32 bytes of report_data are SHA256 HASH of {ga|gb|vk}.
        // The second 32 bytes of report_data are set to zero.
        sample_ret = sample_sha256_init(&sha_handle);
        if (sample_ret != SAMPLE_SUCCESS) {
            Log("Error, init hash failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        sample_ret = sample_sha256_update((uint8_t *)&(g_sp_db.g_a), sizeof(g_sp_db.g_a), sha_handle);
        if (sample_ret != SAMPLE_SUCCESS) {
            Log("Error, udpate hash failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        sample_ret = sample_sha256_update((uint8_t *)&(g_sp_db.g_b), sizeof(g_sp_db.g_b), sha_handle);
        if (sample_ret != SAMPLE_SUCCESS) {
            Log("Error, udpate hash failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        sample_ret = sample_sha256_update((uint8_t *)&(g_sp_db.vk_key), sizeof(g_sp_db.vk_key), sha_handle);
        if (sample_ret != SAMPLE_SUCCESS) {
            Log("Error, udpate hash failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        sample_ret = sample_sha256_get_hash(sha_handle, (sample_sha256_hash_t *)&report_data);
        if (sample_ret != SAMPLE_SUCCESS) {
            Log("Error, Get hash failed", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        if (memcmp((uint8_t *)&report_data, (uint8_t *)&(p_quote->report_body.report_data), sizeof(report_data))) {
            Log("Error, verify hash failed", log::error);
            ret = SP_INTEGRITY_FAILED;
            break;
        }

        // Verify quote with attestation server.
        ias_att_report_t attestation_report = {0};
        ret = ias_verify_attestation_evidence(p_msg3->quote, p_msg3->ps_sec_prop.sgx_ps_sec_prop_desc, &attestation_report, ws);

        if (0 != ret) {
            ret = SP_IAS_FAILED;
            break;
        }

        Log("Attestation Report:");
        Log("\tid: %s", attestation_report.id);
        Log("\tstatus: %d", attestation_report.status);
        Log("\trevocation_reason: %u", attestation_report.revocation_reason);
        Log("\tpse_status: %d",  attestation_report.pse_status);

        Log("Enclave Report:");
        Log("\tSignature Type: 0x%x", p_quote->sign_type);
        Log("\tSignature Basename: %s", ByteArrayToString(p_quote->basename.name, 32));
        Log("\tattributes.flags: 0x%0lx", p_quote->report_body.attributes.flags);
        Log("\tattributes.xfrm: 0x%0lx", p_quote->report_body.attributes.xfrm);
        Log("\tmr_enclave: %s", ByteArrayToString(p_quote->report_body.mr_enclave.m, SGX_HASH_SIZE));
        Log("\tmr_signer: %s", ByteArrayToString(p_quote->report_body.mr_signer.m, SGX_HASH_SIZE));
        Log("\tisv_prod_id: 0x%0x", p_quote->report_body.isv_prod_id);
        Log("\tisv_svn: 0x%0x", p_quote->report_body.isv_svn);


        // Respond the client with the results of the attestation.
        att_result_msg_size = sizeof(sample_ra_att_result_msg_t);

        p_att_result_msg_full = (ra_samp_response_header_t*) malloc(att_result_msg_size + sizeof(ra_samp_response_header_t) + sizeof(validation_result));
        if (!p_att_result_msg_full) {
            Log("Error, out of memory", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        memset(p_att_result_msg_full, 0, att_result_msg_size + sizeof(ra_samp_response_header_t) + sizeof(validation_result));
        p_att_result_msg_full->type = RA_ATT_RESULT;
        p_att_result_msg_full->size = att_result_msg_size;

        if (IAS_QUOTE_OK != attestation_report.status) {
            p_att_result_msg_full->status[0] = 0xFF;
        }

        if (IAS_PSE_OK != attestation_report.pse_status) {
            p_att_result_msg_full->status[1] = 0xFF;
        }

        p_att_result_msg = (sample_ra_att_result_msg_t *)p_att_result_msg_full->body;

        bool isv_policy_passed = true;

        p_att_result_msg->platform_info_blob = attestation_report.info_blob;

        // Generate mac based on the mk key.
        mac_size = sizeof(ias_platform_info_blob_t);
        sample_ret = sample_rijndael128_cmac_msg(&g_sp_db.mk_key,
                     (const uint8_t*)&p_att_result_msg->platform_info_blob,
                     mac_size,
                     &p_att_result_msg->mac);

        if (SAMPLE_SUCCESS != sample_ret) {
            Log("Error, cmac fail", log::error);
            ret = SP_INTERNAL_ERROR;
            break;
        }

        // Generate shared secret and encrypt it with SK, if attestation passed.
        uint8_t aes_gcm_iv[SAMPLE_SP_IV_SIZE] = {0};
        p_att_result_msg->secret.payload_size = MAX_VERIFICATION_RESULT;

        if ((IAS_QUOTE_OK == attestation_report.status ||
             IAS_QUOTE_GROUP_OUT_OF_DATE == attestation_report.status ||
             IAS_QUOTE_CONFIGURATION_NEEDED == attestation_report.status) &&
                (IAS_PSE_OK == attestation_report.pse_status) &&
                (isv_policy_passed == true)) {
            if (IAS_QUOTE_GROUP_OUT_OF_DATE == attestation_report.status)
                Log("GROUP_OUT_OF_DATE detected!!! Your CPU is vulnerable to recent CPU BUGs");
            if (IAS_QUOTE_CONFIGURATION_NEEDED == attestation_report.status)
                Log("CONFIGURATION_NEEDED detected!!! Your CPU has turned on hyper-threading and is vulnerable to recent CPU BUGs");
            memset(validation_result, '\0', MAX_VERIFICATION_RESULT);
            validation_result[0] = 0;
            validation_result[1] = 1;

            ret = sample_rijndael128GCM_encrypt(&g_sp_db.sk_key,
                                                &validation_result[0],
                                                p_att_result_msg->secret.payload_size,
                                                p_att_result_msg->secret.payload,
                                                &aes_gcm_iv[0],
                                                SAMPLE_SP_IV_SIZE,
                                                NULL,
                                                0,
                                                &p_att_result_msg->secret.payload_tag);
        }

    } while(0);

    if (ret) {
        SafeFree(p_att_result_msg_full);
        return -1;
    } else {
        att_msg->set_size(att_result_msg_size);

        ias_platform_info_blob_t platform_info_blob = p_att_result_msg->platform_info_blob;
        att_msg->set_epid_group_status(platform_info_blob.sample_epid_group_status);
        att_msg->set_tcb_evaluation_status(platform_info_blob.sample_tcb_evaluation_status);
        att_msg->set_pse_evaluation_status(platform_info_blob.pse_evaluation_status);

        for (int i=0; i<PSVN_SIZE; i++)
            att_msg->add_latest_equivalent_tcb_psvn(platform_info_blob.latest_equivalent_tcb_psvn[i]);

        for (int i=0; i<ISVSVN_SIZE; i++)
            att_msg->add_latest_pse_isvsvn(platform_info_blob.latest_pse_isvsvn[i]);

        for (int i=0; i<PSDA_SVN_SIZE; i++)
            att_msg->add_latest_psda_svn(platform_info_blob.latest_psda_svn[i]);

        for (int i=0; i<GID_SIZE; i++)
            att_msg->add_performance_rekey_gid(platform_info_blob.performance_rekey_gid[i]);

        for (int i=0; i<SAMPLE_NISTP256_KEY_SIZE; i++) {
            att_msg->add_ec_sign256_x(platform_info_blob.signature.x[i]);
            att_msg->add_ec_sign256_y(platform_info_blob.signature.y[i]);
        }

        for (int i=0; i<SAMPLE_MAC_SIZE; i++)
            att_msg->add_mac_smk(p_att_result_msg->mac[i]);

        att_msg->set_result_size(p_att_result_msg->secret.payload_size);

        for (int i=0; i<12; i++)
            att_msg->add_reserved(p_att_result_msg->secret.reserved[i]);

        for (int i=0; i<16; i++)
            att_msg->add_payload_tag(p_att_result_msg->secret.payload_tag[i]);

        for (int i=0; i<p_att_result_msg->secret.payload_size; i++)
            att_msg->add_payload(p_att_result_msg->secret.payload[i]);
    }

    return ret;
}

void PSIWorker::set_hash_path(string path) {
    this->hash_path = path;
}

int PSIWorker::set_hash_salt(Messages::MessagePsiSalt msg) {
    uint8_t salt[SALT_SIZE];
    uint8_t ciphertext[SALT_SIZE];
    sample_aes_gcm_128bit_tag_t in_mac = {0};

    for (int i=0; i<SALT_SIZE; i++) {
        ciphertext[i] = (uint8_t)msg.salt(i);
    }

    for (int i=0; i<sizeof(sample_aes_gcm_128bit_tag_t); i++) {
        in_mac[i] = (uint8_t)msg.mac(i);
    }

    uint8_t aes_gcm_iv[SAMPLE_SP_IV_SIZE] = {0};

    int ret = sample_rijndael128GCM_decrypt(&g_sp_db.sk_key,
                                            &ciphertext[0],
                                            SALT_SIZE,
                                            &salt[0],
                                            &aes_gcm_iv[0],
                                            SAMPLE_SP_IV_SIZE,
                                            NULL,
                                            0,
                                            &in_mac);
    if (ret != SAMPLE_SUCCESS) {
        Log("decrypt salt failed! %d", ret);
        return -1;
    }

    string psi_salt = ByteArrayToString(salt, SALT_SIZE);
    this->psi_salt = psi_salt;

    //printf_array("salt: ", salt, SALT_SIZE);
    Log("Received SALT: %s", psi_salt);

    return 0;
}

bool PSIWorker::sp_psi_is_finish_get_data() {
    return this->hash_vector_cursor >= this->hash_vector.size();
}

int PSIWorker::sp_psi_get_data_hash(Messages::MessagePsiHashData *data) {
    int ret = 0;

    if (this->hash_vector.size() <= 0) {
        //read file
        uint8_t * file_data = NULL;
        int file_size = 0;

        //No duplicate data by default
        file_size = ReadFileToBuffer(this->hash_path, &file_data);
        if (file_size <= 0) {
            return -1;
        }

        char * p = (char*)file_data;
        const char * s = p;
        char* n = (char*)p;
        for( ; p - s < file_size; p = n + 1) {
            n = strchr(p, '\n');
            if (n == NULL) {
                //only one line or last line
                n = p + strlen(p);
            } else {
                n[0] = '\0';
            }
            if (strlen(p) <= 0) {//ignore null line
                continue;
            }

            sample_sha256_hash_t report_data = {0};
            Sha256 sha256;
            sha256.update((uint8_t*)p, strlen(p));
            sha256.update((uint8_t*)this->psi_salt.c_str(), this->psi_salt.size());
            sha256.hash((sample_sha256_hash_t * )&report_data);

            string hash = ByteArrayToString(report_data, sizeof(sample_sha256_hash_t));

            this->hash_vector.push_back(hash);
            this->data_map[hash] = p;

            Log("[PSI] Init data: %s, hash: %s", p, ByteArrayToString((const uint8_t*)&report_data, sizeof(sample_sha256_hash_t)));
        }

        Log("[PSI] Init all data, size: %d", this->hash_vector.size());

        std::sort(this->hash_vector.begin(), this->hash_vector.end());
        this->hash_vector_cursor = 0;
    }

    if (this->hash_vector_cursor >= this->hash_vector.size()) {
        return -1;
    }

    int count = PSI_HASH_DATA_COUNT;

    if (this->hash_vector_cursor + PSI_HASH_DATA_COUNT > this->hash_vector.size()) {
        count = this->hash_vector.size() - this->hash_vector_cursor;
    }

    int payload_size = count * SAMPLE_SHA256_HASH_SIZE;
    uint8_t payload[payload_size] = {0};
    uint8_t enc_data[payload_size] = {0};

    for (int i = 0; i < count; i++) {
        uint8_t * arr = NULL;
        int size = HexStringToByteArray(this->hash_vector[this->hash_vector_cursor + i], &arr);
        if (size != sizeof(sample_sha256_hash_t)) {
            Log("[PSI] Get hash vector , something error: %d, %d, %s", size, sizeof(sample_sha256_hash_t), this->hash_vector[this->hash_vector_cursor + i]);
            return -1;
        }
        memcpy(payload + i*sizeof(sample_sha256_hash_t), arr, size);
    }

    this->hash_vector_cursor += count;

    uint8_t aes_gcm_iv[SAMPLE_SP_IV_SIZE] = {0};
    sample_aes_gcm_128bit_tag_t out_mac = {0};

    ret = sample_rijndael128GCM_encrypt(&g_sp_db.sk_key,
                                        payload,
                                        payload_size,
                                        enc_data,
                                        &aes_gcm_iv[0],
                                        SAMPLE_SP_IV_SIZE,
                                        NULL,
                                        0,
                                        &out_mac);

    if (ret == -1) {
        Log("sample_rijndael128GCM_encrypt failed");
        return -1;
    }

    int data_size = sizeof(uint32_t) + sizeof(sample_aes_gcm_128bit_tag_t) + payload_size;
    data->set_size(data_size);
    for (int i = 0; i < sizeof(sample_aes_gcm_128bit_tag_t); i++) {
        data->add_mac(out_mac[i]);
    }

    for (int i = 0; i < payload_size; i++) {
        data->add_data((uint32_t)enc_data[i]);
    }

    return 0;
}

int PSIWorker::sp_psi_intersect(Messages::MessagePsiIntersect msg) {

    sample_aes_gcm_128bit_tag_t in_mac = {0};
    for (int i = 0; i < sizeof(sample_aes_gcm_128bit_tag_t); i++) {
        in_mac[i] = (uint8_t)msg.mac(i);
    }

    int data_size = msg.data_size();
    uint8_t* data = (uint8_t*)malloc(data_size);
    uint8_t* dec_data = (uint8_t*)malloc(data_size);

    for (int i = 0; i < data_size; i++) {
        data[i] = (uint8_t)msg.data(i);
    }

    uint8_t aes_gcm_iv[SAMPLE_SP_IV_SIZE] = {0};

    int ret = sample_rijndael128GCM_decrypt(&g_sp_db.sk_key,
                                            &data[0],
                                            data_size,
                                            &dec_data[0],
                                            &aes_gcm_iv[0],
                                            SAMPLE_SP_IV_SIZE,
                                            NULL,
                                            0,
                                            &in_mac);

    if (ret != SAMPLE_SUCCESS) {
        Log("sample_rijndael128GCM_decrypt failed, %d", ret);
        SafeFree(data);
        SafeFree(dec_data);
        return -1;
    }

    // int hash_cnt = data_size / SAMPLE_SHA256_HASH_SIZE;
    // for (int i = 0; i < hash_cnt; i++) {
    //     sample_sha256_hash_t hash = {0};
    //     memcpy(hash, &dec_data[i*SAMPLE_SHA256_HASH_SIZE], SAMPLE_SHA256_HASH_SIZE);
    //     string hash_str = ByteArrayToString(hash, SAMPLE_SHA256_HASH_SIZE);
    //     Log("[PSI] Intersect result: %s", this->data_map[hash_str]);
    // }

    int hash_cnt = 0;
    for (int i = 0; i < data_size && i < this->hash_vector.size(); i++) {
        if (dec_data[i]) {
            hash_cnt++;
            string hash_str = this->hash_vector[i];
            Log("[PSI] Intersect result: %s", this->data_map[hash_str]);
        }
    }

    Log("[PSI] Intersect result count: %d", hash_cnt);

    SafeFree(data);
    SafeFree(dec_data);

    return 0;
}
