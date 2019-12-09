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

#ifndef _ECP_H
#define _ECP_H

#include <stdint.h>
#include <stdlib.h>

#include "remote_attestation_result.h"

#ifndef SAMPLE_FEBITSIZE
#define SAMPLE_FEBITSIZE                    256
#endif

#define SAMPLE_ECP_KEY_SIZE                     (SAMPLE_FEBITSIZE/8)

typedef struct sample_ec_priv_t {
    uint8_t r[SAMPLE_ECP_KEY_SIZE];
} sample_ec_priv_t;

typedef struct sample_ec_dh_shared_t {
    uint8_t s[SAMPLE_ECP_KEY_SIZE];
} sample_ec_dh_shared_t;

typedef uint8_t sample_ec_key_128bit_t[16];

#define SAMPLE_EC_MAC_SIZE 16

#ifdef  __cplusplus
extern "C" {
#endif


#ifndef _ERRNO_T_DEFINED
#define _ERRNO_T_DEFINED
typedef int errno_t;
#endif
errno_t memcpy_s(void *dest, size_t numberOfElements, const void *src,
                 size_t count);


#ifdef SUPPLIED_KEY_DERIVATION

typedef enum _sample_derive_key_type_t {
    SAMPLE_DERIVE_KEY_SMK_SK = 0,
    SAMPLE_DERIVE_KEY_MK_VK,
} sample_derive_key_type_t;

bool derive_key(
    const sample_ec_dh_shared_t *p_shared_key,
    uint8_t key_id,
    sample_ec_key_128bit_t *first_derived_key,
    sample_ec_key_128bit_t *second_derived_key);

#else

typedef enum _sample_derive_key_type_t {
    SAMPLE_DERIVE_KEY_SMK = 0,
    SAMPLE_DERIVE_KEY_SK,
    SAMPLE_DERIVE_KEY_MK,
    SAMPLE_DERIVE_KEY_VK,
} sample_derive_key_type_t;

bool derive_key(
    const sample_ec_dh_shared_t *p_shared_key,
    uint8_t key_id,
    sample_ec_key_128bit_t *derived_key);

#endif

bool verify_cmac128(
    sample_ec_key_128bit_t mac_key,
    const uint8_t *p_data_buf,
    uint32_t buf_size,
    const uint8_t *p_mac_buf);
#ifdef  __cplusplus
}
#endif

#endif

