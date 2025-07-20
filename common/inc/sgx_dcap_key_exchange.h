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

#ifndef _SGX_DCAP_KEY_EXCHANGE_H_
#define _SGX_DCAP_KEY_EXCHANGE_H_

#include <stdint.h>
#include "sgx_key_exchange.h"

#ifdef  __cplusplus
extern "C" {
#endif

typedef struct _dcap_ra_msg1_t
{
    sgx_ec256_public_t       g_a;         /* the Endian-ness of Ga is Little-Endian */
} sgx_dcap_ra_msg1_t;


typedef struct _dcap_ura_msg2_t
{
    sgx_ec256_public_t       g_b;         /* the Endian-ness of Gb is Little-Endian */
    uint32_t                 kdf_id;      /* key derivation function id in little endian. */
    sgx_ec256_signature_t    sign_gb_ga;  /* In little endian */
    sgx_mac_t                mac;         /* mac_smk(g_b||kdf_id||sign_gb_ga) */
} sgx_dcap_ura_msg2_t;

typedef struct _dcap_mra_msg2_t
{
    sgx_mac_t                mac;         /* mac_smk(g_b||kdf_id||quote_size||quote) */
    sgx_ec256_public_t       g_b;         /* the Endian-ness of Gb is Little-Endian */
    uint32_t                 kdf_id;      /* key derivation function id in little endian. */
    uint32_t                 quote_size;
    uint8_t                  quote[];
} sgx_dcap_mra_msg2_t;

typedef struct _dcap_ra_msg3_t
{
    sgx_mac_t                mac;         /* mac_smk(g_a||quote_size||quote) */
    sgx_ec256_public_t       g_a;         /* the Endian-ness of Ga is Little-Endian */
    uint32_t                 quote_size;
    uint8_t                  quote[];
} sgx_dcap_ra_msg3_t;

typedef struct _sgx_dcap_enclave_identity_t
{
    sgx_cpu_svn_t     cpu_svn;
    sgx_misc_select_t misc_select;
    uint8_t           reserved_1[28];
    sgx_attributes_t  attributes;
    sgx_measurement_t mr_enclave;
    uint8_t           reserved_2[32];
    sgx_measurement_t mr_signer;
    uint8_t           reserved_3[96];
    sgx_prod_id_t     isv_prod_id;
    sgx_isv_svn_t     isv_svn;
} sgx_dcap_enclave_identity_t;

#ifdef  __cplusplus
}
#endif

#endif
