#!/bin/bash

# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

echo "deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu $CODENAME main" | sudo tee /etc/apt/sources.list.d/intel-sgx.list && \
wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add - && \
    apt-get update && \
    apt-get install -y \
        libsgx-headers=$SGX_SDK_VERSION \
        libsgx-ae-epid=$SGX_SDK_VERSION \
        libsgx-ae-le=$SGX_SDK_VERSION \
        libsgx-ae-pce=$SGX_SDK_VERSION \
        libsgx-aesm-ecdsa-plugin=$SGX_SDK_VERSION \
        libsgx-aesm-epid-plugin=$SGX_SDK_VERSION \
        libsgx-aesm-launch-plugin=$SGX_SDK_VERSION \
        libsgx-aesm-pce-plugin=$SGX_SDK_VERSION \
        libsgx-aesm-quote-ex-plugin=$SGX_SDK_VERSION \
        libsgx-enclave-common=$SGX_SDK_VERSION \
        libsgx-enclave-common-dev=$SGX_SDK_VERSION \
        libsgx-enclave-common-dbgsym=$SGX_SDK_VERSION \
        libsgx-epid=$SGX_SDK_VERSION \
        libsgx-epid-dev=$SGX_SDK_VERSION \
        libsgx-launch=$SGX_SDK_VERSION \
        libsgx-launch-dev=$SGX_SDK_VERSION \
        libsgx-quote-ex=$SGX_SDK_VERSION \
        libsgx-quote-ex-dev=$SGX_SDK_VERSION \
        libsgx-uae-service=$SGX_SDK_VERSION \
        libsgx-uae-service-dbgsym=$SGX_SDK_VERSION \
        libsgx-urts=$SGX_SDK_VERSION \
        libsgx-urts-dbgsym=$SGX_SDK_VERSION \
        sgx-aesm-service=$SGX_SDK_VERSION \
	    libsgx-ae-qe3=$SGX_DCAP_VERSION \
        libsgx-pce-logic=$SGX_DCAP_VERSION \
        libsgx-qe3-logic=$SGX_DCAP_VERSION \
        libsgx-dcap-ql=$SGX_DCAP_VERSION \
        libsgx-dcap-quote-verify=$SGX_DCAP_VERSION \
        libsgx-dcap-ql-dev=$SGX_DCAP_VERSION \
        libsgx-dcap-ql-dbgsym=$SGX_DCAP_VERSION \
        libsgx-dcap-default-qpl=$SGX_DCAP_VERSION \
        libsgx-dcap-default-qpl-dev=$SGX_DCAP_VERSION \
        libsgx-dcap-default-qpl-dbgsym=$SGX_DCAP_VERSION \
        libsgx-dcap-quote-verify-dev=$SGX_DCAP_VERSION \
        libsgx-dcap-quote-verify-dbgsym=$SGX_DCAP_VERSION \
        libsgx-ra-network=$SGX_DCAP_VERSION \
        libsgx-ra-uefi=$SGX_DCAP_VERSION && \
    mkdir -p /var/run/aesmd && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/cache/apt/archives/*
