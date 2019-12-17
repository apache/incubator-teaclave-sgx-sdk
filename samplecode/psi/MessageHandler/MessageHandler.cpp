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

#include "MessageHandler.h"

using namespace util;

MessageHandler::MessageHandler(int port) {
    this->nm = NetworkManagerServer::getInstance(port);
}

MessageHandler::~MessageHandler() {
    delete this->enclave;
}


int MessageHandler::init() {
    this->nm->Init();
    this->nm->connectCallbackHandler([this](string v, int type) {
        return this->incomingHandler(v, type);
    });
}

void MessageHandler::start() {

    sgx_status_t ret = this->initEnclave();
    if (SGX_SUCCESS != ret) {
        Log("Error, call initEnclave fail", log::error);
        return;
    }

    sgx_status_t status;
    ret = initialize(this->enclave->getID(), &status);
    if ((SGX_SUCCESS != ret) || (SGX_SUCCESS != status)) {
        Log("Error, call generate_salt fail", log::error);
        return;
    }

    Log("Call initEnclave success");
    this->nm->startService();
}


sgx_status_t MessageHandler::initEnclave() {
    this->enclave = Enclave::getInstance();
    return this->enclave->createEnclave();
}

uint32_t MessageHandler::getExtendedEPID_GID() {
    uint32_t extended_epid_group_id = 0;
    int ret = sgx_get_extended_epid_group_id(&extended_epid_group_id);

    if (SGX_SUCCESS != ret) {
        ret = -1;
        Log("Error, call sgx_get_extended_epid_group_id fail");
        return ret;
    }

    Log("Call sgx_get_extended_epid_group_id success");

    return extended_epid_group_id;
}


string MessageHandler::generateMSG0() {
    Log("Call MSG0 generate");

    uint32_t extended_epid_group_id = this->getExtendedEPID_GID();

    Messages::MessageMsg0 msg;
    msg.set_type(RA_MSG0);
    msg.set_epid(extended_epid_group_id);

    return nm->serialize(msg);
}


string MessageHandler::generateMSG1() {
    int retGIDStatus = 0;
    int count = 0;
    sgx_status_t ret;
    sgx_ra_context_t context = INT_MAX;
    sgx_ra_msg1_t sgxMsg1Obj;

    ret = this->enclave->raInit(&context);
    if (SGX_SUCCESS != ret) {
        Log("Error, call enclave_init_ra fail", log::error);
        return "";
    }

    while (1) {
        retGIDStatus = sgx_ra_get_msg1(context,
                                       this->enclave->getID(),
                                       sgx_ra_get_ga,
                                       &sgxMsg1Obj);

        if (retGIDStatus == SGX_SUCCESS) {
            break;
        } else if (retGIDStatus == SGX_ERROR_BUSY) {
            if (count == 5) { //retried 5 times, so fail out
                Log("Error, sgx_ra_get_msg1 is busy - 5 retries failed", log::error);
                break;;
            } else {
                sleep(3);
                count++;
            }
        } else {    //error other than busy
            Log("Error, failed to generate MSG1", log::error);
            break;
        }
    }


    if (SGX_SUCCESS == retGIDStatus) {
        Log("MSG1 generated Successfully");

        Messages::MessageMSG1 msg;
        msg.set_type(RA_MSG1);
        msg.set_context(context);

        for (auto x : sgxMsg1Obj.g_a.gx)
            msg.add_gax(x);

        for (auto x : sgxMsg1Obj.g_a.gy)
            msg.add_gay(x);

        for (auto x : sgxMsg1Obj.gid) {
            msg.add_gid(x);
        }

        return nm->serialize(msg);
    }

    return "";
}


void MessageHandler::assembleMSG2(Messages::MessageMSG2 msg, sgx_ra_msg2_t **pp_msg2) {
    uint32_t size = msg.size();

    sgx_ra_msg2_t *p_msg2 = NULL;
    p_msg2 = (sgx_ra_msg2_t*) malloc(size + sizeof(sgx_ra_msg2_t));

    uint8_t pub_key_gx[32];
    uint8_t pub_key_gy[32];

    sgx_ec256_signature_t sign_gb_ga;
    sgx_spid_t spid;

    for (int i; i<32; i++) {
        pub_key_gx[i] = msg.public_key_gx(i);
        pub_key_gy[i] = msg.public_key_gy(i);
    }

    for (int i=0; i<16; i++) {
        spid.id[i] = msg.spid(i);
    }

    for (int i=0; i<8; i++) {
        sign_gb_ga.x[i] = msg.signature_x(i);
        sign_gb_ga.y[i] = msg.signature_y(i);
    }

    memcpy(&p_msg2->g_b.gx, &pub_key_gx, sizeof(pub_key_gx));
    memcpy(&p_msg2->g_b.gy, &pub_key_gy, sizeof(pub_key_gy));
    memcpy(&p_msg2->sign_gb_ga, &sign_gb_ga, sizeof(sign_gb_ga));
    memcpy(&p_msg2->spid, &spid, sizeof(spid));

    p_msg2->quote_type = (uint16_t)msg.quote_type();
    p_msg2->kdf_id = msg.cmac_kdf_id();

    uint8_t smac[16];
    for (int i=0; i<16; i++)
        smac[i] = msg.smac(i);

    memcpy(&p_msg2->mac, &smac, sizeof(smac));

    p_msg2->sig_rl_size = msg.size_sigrl();

    for (int i=0; i<msg.size_sigrl(); i++)
        p_msg2->sig_rl[i] = msg.sigrl(i);

    *pp_msg2 = p_msg2;
}


string MessageHandler::handleMSG2(Messages::MessageMSG2 msg) {
    Log("Received MSG2");

    uint32_t size = msg.size();
    sgx_ra_context_t context = msg.context();

    sgx_ra_msg2_t *p_msg2;
    this->assembleMSG2(msg, &p_msg2);

    sgx_ra_msg3_t *p_msg3 = NULL;
    uint32_t msg3_size;
    int ret = 0;

    do {
        ret = sgx_ra_proc_msg2(context,
                               this->enclave->getID(),
                               sgx_ra_proc_msg2_trusted,
                               sgx_ra_get_msg3_trusted,
                               p_msg2,
                               size,
                               &p_msg3,
                               &msg3_size);
    } while (SGX_ERROR_BUSY == ret && busy_retry_time--);

    SafeFree(p_msg2);

    if (SGX_SUCCESS != (sgx_status_t)ret) {
        Log("Error, call sgx_ra_proc_msg2 fail, error code: 0x%x", ret);
    } else {
        Log("Call sgx_ra_proc_msg2 success");

        Messages::MessageMSG3 msg3;

        msg3.set_type(RA_MSG3);
        msg3.set_size(msg3_size);
        msg3.set_context(context);

        for (int i=0; i<SGX_MAC_SIZE; i++)
            msg3.add_sgx_mac(p_msg3->mac[i]);

        for (int i=0; i<SGX_ECP256_KEY_SIZE; i++) {
            msg3.add_gax_msg3(p_msg3->g_a.gx[i]);
            msg3.add_gay_msg3(p_msg3->g_a.gy[i]);
        }

        for (int i=0; i<256; i++) {
            msg3.add_sec_property(p_msg3->ps_sec_prop.sgx_ps_sec_prop_desc[i]);
        }


        for (int i=0; i<1116; i++) {
            msg3.add_quote(p_msg3->quote[i]);
        }

        SafeFree(p_msg3);

        return nm->serialize(msg3);
    }

    SafeFree(p_msg3);

    return "";
}


void MessageHandler::assembleAttestationMSG(Messages::AttestationMessage msg, ra_samp_response_header_t **pp_att_msg) {
    sample_ra_att_result_msg_t *p_att_result_msg = NULL;
    ra_samp_response_header_t* p_att_result_msg_full = NULL;

    int total_size = msg.size() + sizeof(ra_samp_response_header_t) + msg.result_size();
    p_att_result_msg_full = (ra_samp_response_header_t*) malloc(total_size);

    memset(p_att_result_msg_full, 0, total_size);
    p_att_result_msg_full->type = RA_ATT_RESULT;
    p_att_result_msg_full->size = msg.size();

    p_att_result_msg = (sample_ra_att_result_msg_t *) p_att_result_msg_full->body;

    p_att_result_msg->platform_info_blob.sample_epid_group_status = msg.epid_group_status();
    p_att_result_msg->platform_info_blob.sample_tcb_evaluation_status = msg.tcb_evaluation_status();
    p_att_result_msg->platform_info_blob.pse_evaluation_status = msg.pse_evaluation_status();

    for (int i=0; i<PSVN_SIZE; i++)
        p_att_result_msg->platform_info_blob.latest_equivalent_tcb_psvn[i] = msg.latest_equivalent_tcb_psvn(i);

    for (int i=0; i<ISVSVN_SIZE; i++)
        p_att_result_msg->platform_info_blob.latest_pse_isvsvn[i] = msg.latest_pse_isvsvn(i);

    for (int i=0; i<PSDA_SVN_SIZE; i++)
        p_att_result_msg->platform_info_blob.latest_psda_svn[i] = msg.latest_psda_svn(i);

    for (int i=0; i<GID_SIZE; i++)
        p_att_result_msg->platform_info_blob.performance_rekey_gid[i] = msg.performance_rekey_gid(i);

    for (int i=0; i<SAMPLE_NISTP256_KEY_SIZE; i++) {
        p_att_result_msg->platform_info_blob.signature.x[i] = msg.ec_sign256_x(i);
        p_att_result_msg->platform_info_blob.signature.y[i] = msg.ec_sign256_y(i);
    }

    for (int i=0; i<SAMPLE_MAC_SIZE; i++)
        p_att_result_msg->mac[i] = msg.mac_smk(i);


    p_att_result_msg->secret.payload_size = msg.result_size();

    for (int i=0; i<12; i++)
        p_att_result_msg->secret.reserved[i] = msg.reserved(i);

    for (int i=0; i<SAMPLE_SP_TAG_SIZE; i++)
        p_att_result_msg->secret.payload_tag[i] = msg.payload_tag(i);

    for (int i=0; i<SAMPLE_SP_TAG_SIZE; i++)
        p_att_result_msg->secret.payload_tag[i] = msg.payload_tag(i);

    for (int i=0; i<msg.result_size(); i++) {
        p_att_result_msg->secret.payload[i] = (uint8_t)msg.payload(i);
    }

    *pp_att_msg = p_att_result_msg_full;
}

string MessageHandler::generateAttestationFailed(uint32_t id, sgx_ra_context_t context) {

    Messages::MessagePsiSalt msg;

    msg.set_type(RA_PSI_SLAT);
    msg.set_size(0);
    msg.set_state(0);
    msg.set_context(context);
    msg.add_salt(0);
    msg.add_mac(0);
    msg.set_id(id);

    return nm->serialize(msg);
}

string MessageHandler::handleAttestationResult(Messages::AttestationMessage msg) {
    Log("Received Attestation result");

    ra_samp_response_header_t *p_att_result_msg_full = NULL;
    this->assembleAttestationMSG(msg, &p_att_result_msg_full);
    sample_ra_att_result_msg_t *p_att_result_msg_body = (sample_ra_att_result_msg_t *) ((uint8_t*) p_att_result_msg_full + sizeof(ra_samp_response_header_t));

    sgx_status_t status;
    sgx_status_t ret;
    sgx_ra_context_t context = msg.context();
    uint32_t id = 0;
    uint8_t salt[SALT_SIZE];
    uint8_t mac[SGX_MAC_SIZE];

    ret = verify_att_result_mac(this->enclave->getID(),
                                &status,
                                context,
                                (uint8_t*)&p_att_result_msg_body->platform_info_blob,
                                sizeof(ias_platform_info_blob_t),
                                (uint8_t*)&p_att_result_msg_body->mac);


    if ((SGX_SUCCESS != ret) || (SGX_SUCCESS != status)) {
        Log("Error: INTEGRITY FAILED - attestation result message MK based cmac failed", log::error);
        SafeFree(p_att_result_msg_full);
        return generateAttestationFailed(context, id);
    }

    if (0 != p_att_result_msg_full->status[0] || 0 != p_att_result_msg_full->status[1]) {
        Log("Error, attestation mac result message MK based cmac failed", log::error);
        SafeFree(p_att_result_msg_full);
        return generateAttestationFailed(context, id);
    } else {
        ret = verify_secret_data(this->enclave->getID(),
                                 &status,
                                 context,
                                 p_att_result_msg_body->secret.payload,
                                 p_att_result_msg_body->secret.payload_size,
                                 p_att_result_msg_body->secret.payload_tag,
                                 MAX_VERIFICATION_RESULT,
                                 salt,
                                 mac,
                                 &id);

        SafeFree(p_att_result_msg_full);

        if (SGX_SUCCESS != ret) {
            Log("Error, attestation result message secret using SK based AESGCM failed", log::error);
            Log("Error  on ret , code : %08X\n",ret);
            print_error_message(ret);

            return generateAttestationFailed(context, id);

        } else if (SGX_SUCCESS != status) {
            Log("Error, attestation result message secret using SK based AESGCM failed", log::error);
            Log("Error  on status, code : %08X\n",status);
            print_error_message(status);

            return generateAttestationFailed(context, id);

        } else {
            Log("Send attestation okay");

            Messages::MessagePsiSalt msg;
            msg.set_type(RA_PSI_SLAT);
            msg.set_size(0);
            msg.set_state(1);
            msg.set_context(context);
            msg.set_id(id);

            for (int i = 0; i < SALT_SIZE; i++) {
                msg.add_salt(salt[i]);
            }

            for (int i = 0; i < SGX_MAC_SIZE; i++) {
                msg.add_mac(mac[i]);
            }

            return nm->serialize(msg);
        }
    }

    return "";
}


string MessageHandler::handleMSG0(Messages::MessageMsg0 msg) {
    Log("MSG0 response received");

    if (msg.status() == TYPE_OK) {
        Log("Sending msg1 to remote attestation service provider. Expecting msg2 back");
        auto ret = this->generateMSG1();
        return ret;
    } else {
        Log("MSG0 response status was not OK", log::error);
    }

    return "";
}


string MessageHandler::handleVerification() {
    Log("Verification request received");
    return this->generateMSG0();
}

string MessageHandler::createInitMsg(int type, string msg) {
    Messages::InitialMessage init_msg;
    init_msg.set_type(type);
    init_msg.set_size(0);

    return nm->serialize(init_msg);
}

string MessageHandler::handlePsiHashData(Messages::MessagePsiHashData msg) {
    Log("[PSI] Received hash data");

    uint8_t mac[SGX_MAC_SIZE] = {0};
    sgx_ra_context_t context = msg.context();
    uint32_t id = msg.id();
    int data_size = msg.data_size();
    uint8_t* data = (uint8_t*)malloc(data_size);

    for (int i = 0; i < SGX_MAC_SIZE; i++) {
        mac[i] = (uint8_t)msg.mac(i);
    }

    for (int i = 0; i < data_size; i++) {
        data[i] = (uint8_t)msg.data(i);
    }

    sgx_status_t status;
    sgx_status_t ret = add_hash_data(this->enclave->getID(),
                                    &status,
                                    id,
                                    context,
                                    data,
                                    data_size,
                                    mac);
    if (SGX_SUCCESS != ret || SGX_SUCCESS != status) {
        Log("[PSI] add_hash_data failed, %d, %d!", ret, status);
        SafeFree(data);
        return "";
    }

    SafeFree(data);

    Messages::MessagePsiResult result;
    result.set_type(RA_PSI_RESULT);
    result.set_size(0);
    result.set_state(0);
    result.set_context(msg.context());
    result.set_id(msg.id());

    return nm->serialize(result);
}

string MessageHandler::handlePsiHashDataFinished(Messages::MessagePsiHashDataFinished msg, bool* again) {

    sgx_ra_context_t context = msg.context();
    uint32_t id = msg.id();
    uint8_t mac[SGX_MAC_SIZE] = {0};
    uint8_t * data = NULL;
    size_t data_size = 0;
    sgx_status_t status;
    sgx_status_t ret;

    Log("[PSI] Received hash data finished, %d", id);

    ret = get_result_size(this->enclave->getID(), &status, id, &data_size);
    if (SGX_SUCCESS != ret) {
        Log("[PSI] get_result_size failed, %d", ret);
        return "";
    }

    if (SGX_SUCCESS != status) {
        if (status == SGX_ERROR_INVALID_STATE) {
            *again = true;

            Messages::MessagePsiResult result;
            result.set_type(RA_PSI_RESULT);
            result.set_size(0);
            result.set_state(1); //tell sp req result again
            result.set_context(msg.context());
            result.set_id(msg.id());

            Log("[PSI] has not calc result success");

            return nm->serialize(result);
        } else {
            Log("[PSI] get_result_size failed, %d, %d", ret, status);
            return "";
        }
    }

    if (data_size > 0) {
        data = (uint8_t*)malloc(data_size);
        if (data == NULL) {
            Log("[PSI] alloc buffer for intersect data failed!");
            return "";
        }

        ret = get_result(this->enclave->getID(),
                        &status,
                        id,
                        context,
                        data,
                        data_size,
                        mac);
        if (SGX_SUCCESS != ret || SGX_SUCCESS != status) {
            Log("[PSI] get_result failed, %d, %d", ret, status);
            return "";
        }
    }

    Messages::MessagePsiIntersect intersect;
    intersect.set_type(RA_PSI_INTERSECT);
    intersect.set_size(0);
    intersect.set_id(msg.id());
    intersect.set_context(context);

    for (int i = 0; i < SGX_MAC_SIZE; i++) {
        intersect.add_mac(mac[i]);
    }

    if (data && data_size) {
        for (int i = 0; i < data_size; i++) {
            intersect.add_data(data[i]);
        }
    }
    if (data) {
        SafeFree(data);
    }

    enclave_ra_close(this->enclave->getID(), &status, context);

    Log("[PSI] get result success, %d", data_size/SGX_HASH_SIZE);

    return nm->serialize(intersect);
}

vector<string> MessageHandler::incomingHandler(string v, int type) {
    vector<string> res;
    string s;
    bool ret;

    switch (type) {
    case RA_VERIFICATION: {	//Verification request
        Messages::InitialMessage init_msg;
        ret = init_msg.ParseFromString(v);
        if (ret && init_msg.type() == RA_VERIFICATION) {
            s = this->handleVerification();
            res.push_back(to_string(RA_MSG0));
        }
    }
    break;
    case RA_MSG0: {		//Reply to MSG0
        Messages::MessageMsg0 msg0;
        ret = msg0.ParseFromString(v);
        if (ret && (msg0.type() == RA_MSG0)) {
            s = this->handleMSG0(msg0);
            res.push_back(to_string(RA_MSG1));
        }
    }
    break;
    case RA_MSG2: {		//MSG2
        Messages::MessageMSG2 msg2;
        ret = msg2.ParseFromString(v);
        if (ret && (msg2.type() == RA_MSG2)) {
            s = this->handleMSG2(msg2);
            res.push_back(to_string(RA_MSG3));
        }
    }
    break;
    case RA_ATT_RESULT: {	//Reply to MSG3
        Messages::AttestationMessage att_msg;
        ret = att_msg.ParseFromString(v);
        if (ret && att_msg.type() == RA_ATT_RESULT) {
            s = this->handleAttestationResult(att_msg);
            res.push_back(to_string(RA_PSI_SLAT));
        }
    }
    break;
    case RA_PSI_HASHDATA: {
        Messages::MessagePsiHashData data_msg;
        ret = data_msg.ParseFromString(v);
        if (ret && data_msg.type() == RA_PSI_HASHDATA) {
            s = this->handlePsiHashData(data_msg);
            res.push_back(to_string(RA_PSI_RESULT));
        }
    }
    break;
    case RA_PSI_HASHDATA_FINISHED: {
        Messages::MessagePsiHashDataFinished finish_msg;
        ret = finish_msg.ParseFromString(v);
        if (ret && finish_msg.type() == RA_PSI_HASHDATA_FINISHED) {
            bool again = false;
            s = this->handlePsiHashDataFinished(finish_msg, &again);
            if (again) {
                res.push_back(to_string(RA_PSI_RESULT));
            } else {
                res.push_back(to_string(RA_PSI_INTERSECT));
            }
        }
    }
    break;
    default:
        Log("Unknown type: %d", type, log::error);
        break;
    }

    res.push_back(s);

    return res;
}
