/*
 * Copyright (C) 2011-2021 Intel Corporation. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 *   * Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 *   * Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in
 *     the documentation and/or other materials provided with the
 *     distribution.
 *   * Neither the name of Intel Corporation nor the names of its
 *     contributors may be used to endorse or promote products derived
 *     from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 */

#include "ipp_wrapper.h"
#include "sgx_sm2_internal.h"

#define ECC_FIELD_SIZE 256

sgx_status_t sgx_sm2_digest_preprocess(const uint8_t *p_data,
                                    int data_size,
                                    const sgx_ec256_public_t *p_att_pub_key,
                                    sgx_sm3_hash_t *p_hash)
{
    if ((p_data == NULL) || (data_size < 1) || (p_att_pub_key == NULL) || (p_hash == NULL)) {
        return SGX_ERROR_INVALID_PARAMETER;
    }

    sgx_status_t ret = SGX_SUCCESS;
    IppStatus ipp_ret = ippStsNoErr;
    IppsSM3State* p_sm3_state = NULL;
    int ctx_size = 0;
    uint8_t hash_z[SGX_SM3_HASH_SIZE] = { 0 };

    ret = sgx_sm2_digest_z(p_att_pub_key, (sgx_sm3_hash_t *)hash_z);
    if (SGX_SUCCESS != ret) {
        return ret;
    }

    do {
        ipp_ret = ippsSM3GetSize(&ctx_size);
        ERROR_BREAK(ipp_ret);

        p_sm3_state = (IppsSM3State*)(malloc(ctx_size));
        if (!p_sm3_state) {
            ipp_ret = ippStsNoMemErr;
            break;
        }

        ipp_ret = ippsSM3Init(p_sm3_state);
        ERROR_BREAK(ipp_ret);

        ipp_ret = ippsSM3Update(hash_z, SGX_SM3_HASH_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);
        ipp_ret = ippsSM3Update(p_data, data_size, p_sm3_state);
        ERROR_BREAK(ipp_ret);

        ipp_ret = ippsSM3GetTag((Ipp8u*)p_hash, SGX_SM3_HASH_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);

    } while(0);

    SAFE_FREE(p_sm3_state);

    switch (ipp_ret)
    {
    case ippStsNoErr: return SGX_SUCCESS;
    case ippStsNoMemErr:
    case ippStsMemAllocErr: return SGX_ERROR_OUT_OF_MEMORY;
    case ippStsNullPtrErr:
    case ippStsLengthErr:
    case ippStsSizeErr:
    case ippStsBadArgErr: return SGX_ERROR_INVALID_PARAMETER;
    default: return SGX_ERROR_UNEXPECTED;
    }
}

sgx_status_t sgx_sm2_digest_z(const sgx_ec256_public_t *p_att_pub_key, sgx_sm3_hash_t *p_hash)
{
    if ((p_att_pub_key == NULL) || (p_hash == NULL)) {
        return SGX_ERROR_INVALID_PARAMETER;
    }

    uint8_t sm2_user_id[16] = {
        0x00, 0x70, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
    };
    uint8_t sm2_param_a[SGX_ECP256_KEY_SIZE] = {
        0xff, 0xff, 0xff, 0xfe, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfc};
    uint8_t sm2_param_b[SGX_ECP256_KEY_SIZE] = {
        0x28, 0xe9, 0xfa, 0x9e, 0x9d, 0x9f, 0x5e, 0x34,
        0x4d, 0x5a, 0x9e, 0x4b, 0xcf, 0x65, 0x09, 0xa7,
        0xf3, 0x97, 0x89, 0xf5, 0x15, 0xab, 0x8f, 0x92,
        0xdd, 0xbc, 0xbd, 0x41, 0x4d, 0x94, 0x0e, 0x93};
    uint8_t sm2_param_x_g[SGX_ECP256_KEY_SIZE] = {
        0x32, 0xc4, 0xae, 0x2c, 0x1f, 0x19, 0x81, 0x19,
        0x5f, 0x99, 0x04, 0x46, 0x6a, 0x39, 0xc9, 0x94,
        0x8f, 0xe3, 0x0b, 0xbf, 0xf2, 0x66, 0x0b, 0xe1,
        0x71, 0x5a, 0x45, 0x89, 0x33, 0x4c, 0x74, 0xc7};
    uint8_t sm2_param_y_g[SGX_ECP256_KEY_SIZE] = {
        0xbc, 0x37, 0x36, 0xa2, 0xf4, 0xf6, 0x77, 0x9c,
        0x59, 0xbd, 0xce, 0xe3, 0x6b, 0x69, 0x21, 0x53,
        0xd0, 0xa9, 0x87, 0x7c, 0xc6, 0x2a, 0x47, 0x40,
        0x02, 0xdf, 0x32, 0xe5, 0x21, 0x39, 0xf0, 0xa0};
    uint8_t pub_gx[SGX_ECP256_KEY_SIZE] = { 0 };
    uint8_t pub_gy[SGX_ECP256_KEY_SIZE] = { 0 };

    for (int i = 0; i < SGX_ECP256_KEY_SIZE; i++)
    {
        pub_gx[i] = p_att_pub_key->gx[SGX_ECP256_KEY_SIZE-1-i];
        pub_gy[i] = p_att_pub_key->gy[SGX_ECP256_KEY_SIZE-1-i];
    }

    IppStatus ipp_ret = ippStsNoErr;
    IppsSM3State* p_sm3_state = NULL;
    int ctx_size = 0;

    do {
        ipp_ret = ippsSM3GetSize(&ctx_size);
        ERROR_BREAK(ipp_ret);

        p_sm3_state = (IppsSM3State*)(malloc(ctx_size));
        if (!p_sm3_state) {
            ipp_ret = ippStsNoMemErr;
            break;
        }

        ipp_ret = ippsSM3Init(p_sm3_state);
        ERROR_BREAK(ipp_ret);

        ipp_ret = ippsSM3Update((Ipp8u *)sm2_user_id, 16, p_sm3_state);
        ERROR_BREAK(ipp_ret);
        ipp_ret = ippsSM3Update(sm2_param_a, SGX_ECP256_KEY_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);
        ipp_ret = ippsSM3Update(sm2_param_b, SGX_ECP256_KEY_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);
        ipp_ret = ippsSM3Update(sm2_param_x_g, SGX_ECP256_KEY_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);
        ipp_ret = ippsSM3Update(sm2_param_y_g, SGX_ECP256_KEY_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);
        ipp_ret = ippsSM3Update(pub_gx, SGX_ECP256_KEY_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);
        ipp_ret = ippsSM3Update(pub_gy, SGX_ECP256_KEY_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);

        ipp_ret = ippsSM3GetTag((Ipp8u*)p_hash, SGX_SM3_HASH_SIZE, p_sm3_state);
        ERROR_BREAK(ipp_ret);

    } while(0);

    SAFE_FREE(p_sm3_state);

    switch (ipp_ret)
    {
    case ippStsNoErr: return SGX_SUCCESS;
    case ippStsNoMemErr:
    case ippStsMemAllocErr: return SGX_ERROR_OUT_OF_MEMORY;
    case ippStsNullPtrErr:
    case ippStsLengthErr:
    case ippStsSizeErr:
    case ippStsBadArgErr: return SGX_ERROR_INVALID_PARAMETER;
    default: return SGX_ERROR_UNEXPECTED;
    }
}

sgx_status_t sgx_sm2_pub_from_priv(const sgx_ec256_private_t *p_att_priv_key, sgx_ec256_public_t *p_att_pub_key, sgx_ecc_state_handle_t ecc_handle)
{
    if ((ecc_handle == NULL) || (p_att_priv_key == NULL) || (p_att_pub_key == NULL)) {
        return SGX_ERROR_INVALID_PARAMETER;
    }

    IppsECCPState* p_ecc_state = (IppsECCPState*)ecc_handle;;
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    int point_size = 0;
    IppsECCPPointState* public_key = NULL;
    IppsBigNumState*    bn_o = NULL;
    IppsBigNumState*    bn_x = NULL;
    IppsBigNumState*    bn_y = NULL;
    sgx_ec256_private_t att_priv_key_be;
    uint8_t* p_temp;
    int size = 0;
    IppsBigNumSGN sgn;

    do {
        //get point (public key) size
        //
        if (ippsECCPPointGetSize(ECC_FIELD_SIZE, &point_size) != ippStsNoErr) {
            break;
        }

        //allocate point of point_size size
        //
        public_key = (IppsECCPPointState*)(malloc(point_size));
        if (NULL == public_key) {
            ret = SGX_ERROR_OUT_OF_MEMORY;
            break;
        }

        //init point
        //
        if (ippsECCPPointInit(ECC_FIELD_SIZE, public_key) != ippStsNoErr) {
            break;
        }

        //allocate bn_o, will be used for private key
        //
        if (sgx_ipp_newBN(NULL, sizeof(sgx_ec256_private_t), &bn_o) != ippStsNoErr) {
            break;
        }

        //convert private key into big endian
        //
        p_temp = (uint8_t*)p_att_priv_key;
        for (uint32_t i = 0; i<sizeof(att_priv_key_be); i++) {
            att_priv_key_be.r[i] = *(p_temp + sizeof(att_priv_key_be) - 1 - i);
        }

        //assign private key into bn_o
        //
        if (ippsSetOctString_BN(reinterpret_cast<Ipp8u *>(&att_priv_key_be), sizeof(sgx_ec256_private_t), bn_o) != ippStsNoErr) {
            break;
        }

        //compute public key from the given private key (bn_o) of the elliptic cryptosystem (p_ecc_state) over GF(p).
        //
        if (ippsECCPPublicKey(bn_o, public_key, p_ecc_state) != ippStsNoErr) {
            break;
        }

        //allocate BNs
        //
        if (sgx_ipp_newBN(NULL, sizeof(sgx_ec256_private_t), &bn_x) != ippStsNoErr) {
            break;
        }

        if (sgx_ipp_newBN(NULL, sizeof(sgx_ec256_private_t), &bn_y) != ippStsNoErr) {
            break;
        }
        //assign public key into BNs
        //
        if (ippsECCPGetPoint(bn_x, bn_y, public_key, p_ecc_state) != ippStsNoErr) {
            break;
        }

        //output key in little endian order
        //
        //gx value
        if (ippsGetSize_BN(bn_x, &size) != ippStsNoErr) {
            break;
        }
        if (ippsGet_BN(&sgn, &size, reinterpret_cast<Ipp32u *>(p_att_pub_key->gx), bn_x) != ippStsNoErr) {
            break;
        }
        //gy value
        //
        if (ippsGetSize_BN(bn_y, &size) != ippStsNoErr) {
            break;
        }
        if (ippsGet_BN(&sgn, &size, reinterpret_cast<Ipp32u *>(p_att_pub_key->gy), bn_y) != ippStsNoErr) {
            break;
        }

        ret = SGX_SUCCESS;
    } while (0);

    //in case of failure clear public key
    //
    if (ret != SGX_SUCCESS) {
        (void)memset_s(p_att_pub_key, sizeof(sgx_ec256_public_t), 0, sizeof(sgx_ec256_public_t));
    }

    CLEAR_FREE_MEM(public_key, point_size);
    sgx_ipp_secure_free_BN(bn_o, sizeof(sgx_ec256_private_t));
    sgx_ipp_secure_free_BN(bn_x, sizeof(sgx_ec256_private_t));
    sgx_ipp_secure_free_BN(bn_y, sizeof(sgx_ec256_private_t));

    return ret;
}
