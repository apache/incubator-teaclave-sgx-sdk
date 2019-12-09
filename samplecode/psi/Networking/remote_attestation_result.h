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

#ifndef _REMOTE_ATTESTATION_RESULT_H_
#define _REMOTE_ATTESTATION_RESULT_H_

#include <stdint.h>

#ifdef  __cplusplus
extern "C" {
#endif

#define SAMPLE_MAC_SIZE             16  /* Message Authentication Code*/
/* - 16 bytes*/
typedef uint8_t                     sample_mac_t[SAMPLE_MAC_SIZE];

#ifndef SAMPLE_FEBITSIZE
#define SAMPLE_FEBITSIZE        256
#endif

#define SAMPLE_NISTP256_KEY_SIZE    (SAMPLE_FEBITSIZE/ 8 /sizeof(uint32_t))

typedef struct sample_ec_sign256_t {
    uint32_t x[SAMPLE_NISTP256_KEY_SIZE];
    uint32_t y[SAMPLE_NISTP256_KEY_SIZE];
} sample_ec_sign256_t;

#pragma pack(push,1)

#define SAMPLE_SP_TAG_SIZE          16

typedef struct sp_aes_gcm_data_t {
    uint32_t        payload_size;       /*  0: Size of the payload which is*/
    /*     encrypted*/
    uint8_t         reserved[12];       /*  4: Reserved bits*/
    uint8_t         payload_tag[SAMPLE_SP_TAG_SIZE];
    /* 16: AES-GMAC of the plain text,*/
    /*     payload, and the sizes*/
    uint8_t         payload[];          /* 32: Ciphertext of the payload*/
    /*     followed by the plain text*/
} sp_aes_gcm_data_t;


#define ISVSVN_SIZE 2
#define PSDA_SVN_SIZE 4
#define GID_SIZE 4
#define PSVN_SIZE 18

/* @TODO: Modify at production to use the values specified by an Production*/
/* attestation server API*/
typedef struct ias_platform_info_blob_t {
    uint8_t sample_epid_group_status;
    uint16_t sample_tcb_evaluation_status;
    uint16_t pse_evaluation_status;
    uint8_t latest_equivalent_tcb_psvn[PSVN_SIZE];
    uint8_t latest_pse_isvsvn[ISVSVN_SIZE];
    uint8_t latest_psda_svn[PSDA_SVN_SIZE];
    uint8_t performance_rekey_gid[GID_SIZE];
    sample_ec_sign256_t signature;
} ias_platform_info_blob_t;


typedef struct sample_ra_att_result_msg_t {
    ias_platform_info_blob_t    platform_info_blob;
    sample_mac_t                mac;    /* mac_smk(attestation_status)*/
    sp_aes_gcm_data_t           secret;
} sample_ra_att_result_msg_t;

#pragma pack(pop)

#ifdef  __cplusplus
}
#endif

#endif
