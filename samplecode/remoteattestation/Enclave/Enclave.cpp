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
    int ret = -1;

    if (INT_MAX != context) {
        int ret_save = -1;
        ret = enclave_ra_close(enclave_id, &status, context);
        if (SGX_SUCCESS != ret || status) {
            ret = -1;
            Log("Error, call enclave_ra_close fail", log::error);
        } else {
            // enclave_ra_close was successful, let's restore the value that
            // led us to this point in the code.
            ret = ret_save;
        }

        Log("Call enclave_ra_close success");
    }

    sgx_destroy_enclave(enclave_id);
}



sgx_status_t Enclave::createEnclave() {
    sgx_status_t ret;
    int launch_token_update = 0;
    int enclave_lost_retry_time = 1;
    sgx_launch_token_t launch_token = {0};

    memset(&launch_token, 0, sizeof(sgx_launch_token_t));

    do {
        ret = sgx_create_enclave(this->enclave_path,
                                 SGX_DEBUG_FLAG,
                                 &launch_token,
                                 &launch_token_update,
                                 &this->enclave_id, NULL);

        if (SGX_SUCCESS != ret) {
            Log("Error, call sgx_create_enclave fail", log::error);
            print_error_message(ret);
            break;
        } else {
            Log("Call sgx_create_enclave success");

            ret = enclave_init_ra(this->enclave_id,
                                  &this->status,
                                  false,
                                  &this->context);
        }

    } while (SGX_ERROR_ENCLAVE_LOST == ret && enclave_lost_retry_time--);

    if (ret == SGX_SUCCESS)
        Log("Enclave created, ID: %llx", this->enclave_id);


    return ret;
}


sgx_enclave_id_t Enclave::getID() {
    return this->enclave_id;
}

sgx_status_t Enclave::getStatus() {
    return this->status;
}

sgx_ra_context_t Enclave::getContext() {
    return this->context;
}
