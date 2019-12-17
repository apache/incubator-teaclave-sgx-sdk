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

#ifndef SERVICE_PROVIDER_H
#define SERVICE_PROVIDER_H

#include <iomanip>
#include <sstream>
#include <algorithm>    // std::reverse
#include <stdio.h>
#include <stdlib.h>
#include <stddef.h>
#include <time.h>
#include <string.h>
#include <iostream>
#include <map>
#include <vector>
#include <algorithm>    // std::sort

#include "Messages.pb.h"
#include "UtilityFunctions.h"
#include "LogBase.h"
#include "Network_def.h"
#include "WebService.h"

#include "remote_attestation_result.h"
#include "sgx_key_exchange.h"
#include "ias_ra.h"

using namespace std;

#define DH_HALF_KEY_LEN 32
#define DH_SHARED_KEY_LEN 32
#define SAMPLE_SP_IV_SIZE 12

enum sp_ra_msg_status_t {
    SP_OK,
    SP_UNSUPPORTED_EXTENDED_EPID_GROUP,
    SP_INTEGRITY_FAILED,
    SP_QUOTE_VERIFICATION_FAILED,
    SP_IAS_FAILED,
    SP_INTERNAL_ERROR,
    SP_PROTOCOL_ERROR,
    SP_QUOTE_VERSION_ERROR,
    SP_RETRIEVE_SIGRL_ERROR
};

typedef struct _sp_db_item_t {
    sgx_ec256_public_t       	g_a;
    sgx_ec256_public_t       	g_b;
    sgx_ec_key_128bit_t      	vk_key;		// Shared secret key for the REPORT_DATA
    sgx_ec_key_128bit_t      	mk_key;		// Shared secret key for generating MAC's
    sgx_ec_key_128bit_t      	sk_key;		// Shared secret key for encryption
    sgx_ec_key_128bit_t      	smk_key;	// Used only for SIGMA protocol
    sample_ec_priv_t            b;
    sgx_ps_sec_prop_desc_t   ps_sec_prop;
} sp_db_item_t;

class PSIWorker {

public:
    PSIWorker(WebService *ws);
    virtual ~PSIWorker();
    int sp_ra_proc_msg0_req(const uint32_t extended_epid_group_id);
    int sp_ra_proc_msg1_req(Messages::MessageMSG1 msg1, Messages::MessageMSG2 *msg2);
    int sp_ra_proc_msg3_req(Messages::MessageMSG3 msg, Messages::AttestationMessage *att_msg);
    sgx_ra_msg3_t* assembleMSG3(Messages::MessageMSG3 msg);

    void set_hash_path(string path);
    int set_hash_salt(Messages::MessagePsiSalt msg);

    int sp_psi_get_data_hash(Messages::MessagePsiHashData *data);
    bool sp_psi_is_finish_get_data();

    int sp_psi_intersect(Messages::MessagePsiIntersect msg);

private:
    WebService *ws = NULL;
    bool g_is_sp_registered = false;
    uint32_t extended_epid_group_id;
    sp_db_item_t g_sp_db;
    const uint16_t AES_CMAC_KDF_ID = 0x0001;
    uint8_t validation_result[MAX_VERIFICATION_RESULT];

    string psi_salt;
    string hash_path;

    std::vector<string> hash_vector;
    std::map<string, string> data_map;
    uint32_t hash_vector_cursor;
};

#endif










