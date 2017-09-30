// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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


class ServiceProvider {

public:
    ServiceProvider(WebService *ws);
    virtual ~ServiceProvider();
    int sp_ra_proc_msg0_req(const uint32_t extended_epid_group_id);
    int sp_ra_proc_msg1_req(Messages::MessageMSG1 msg1, Messages::MessageMSG2 *msg2);
    int sp_ra_proc_msg3_req(Messages::MessageMSG3 msg, Messages::AttestationMessage *att_msg);
    sgx_ra_msg3_t* assembleMSG3(Messages::MessageMSG3 msg);
    int sp_ra_proc_app_att_hmac(Messages::SecretMessage *new_msg, string hmac_key, string hmac_key_filename);

private:
    WebService *ws = NULL;
    bool g_is_sp_registered = false;
    uint32_t extended_epid_group_id;
    sp_db_item_t g_sp_db;
    const uint16_t AES_CMAC_KDF_ID = 0x0001;
    uint8_t validation_result[MAX_VERIFICATION_RESULT];
};

#endif










