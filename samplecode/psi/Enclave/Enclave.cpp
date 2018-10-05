// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

#include "Enclave.h"

#include <iostream>

using namespace util;
using namespace std;

Enclave* Enclave::instance = NULL;

Enclave::Enclave() {}

Enclave* Enclave::getInstance() {
    if (instance == NULL) {
        instance = new Enclave();
    }

    return instance;
}


Enclave::~Enclave() {
    sgx_destroy_enclave(enclave_id);
}


sgx_status_t Enclave::createEnclave() {
    sgx_status_t ret;
    int launch_token_update = 0;
    sgx_launch_token_t launch_token = {0};

    memset(&launch_token, 0, sizeof(sgx_launch_token_t));

    ret = sgx_create_enclave(this->enclave_path,
                             SGX_DEBUG_FLAG,
                             &launch_token,
                             &launch_token_update,
                             &this->enclave_id, NULL);

    if (SGX_SUCCESS != ret) {
        Log("Error, call sgx_create_enclave fail", log::error);
        print_error_message(ret);
    } else {
        Log("Enclave created, ID: %llx", this->enclave_id);
    }

    return ret;
}

sgx_status_t Enclave::raInit(sgx_ra_context_t *ra_context) {
    sgx_status_t ret;
    sgx_status_t status;
    sgx_ra_context_t context = INT_MAX;
    int enclave_lost_retry_time = 1;

    if (ra_context == NULL) {
        Log("Error, call raInit fail", log::error);
        return SGX_ERROR_INVALID_PARAMETER;
    }

    do {
        ret = enclave_init_ra(this->enclave_id,
                              &status,
                              false,
                              &context);
    } while (SGX_ERROR_ENCLAVE_LOST == ret && enclave_lost_retry_time--);

    if (SGX_SUCCESS != ret || status) {
        Log("Error, call enclave_ra_init fail", log::error);
    } else {
        Log("Call enclave_ra_init success");
        *ra_context = context;
    }
    return ret;
}

void Enclave::raClose(sgx_ra_context_t ra_context) {
    int ret = -1;

    if (INT_MAX != ra_context) {
        int ret_save = -1;
        sgx_status_t status;
        ret = enclave_ra_close(enclave_id, &status, ra_context);
        if (SGX_SUCCESS != ret || status) {
            ret = -1;
            Log("Error, call enclave_ra_close fail", log::error);
        } else {
            // enclave_ra_close was successful, let's restore the value that
            // led us to this point in the code.
            ret = ret_save;
            Log("Call enclave_ra_close success");
        }
    }
}

sgx_enclave_id_t Enclave::getID() {
    return this->enclave_id;
}
