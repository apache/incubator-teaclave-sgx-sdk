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

#include "sgx_tcrypto.h"
#include "ippcp.h"
#include "ipp_wrapper.h"
#include "stdlib.h"
#include "string.h"
#include <limits.h>

/* AES-CCM 128-bit
* Parameters:
*   Return: sgx_status_t  - SGX_SUCCESS or failure as defined sgx_error.h
*   Inputs: sgx_aes_ccm_128bit_key_t *p_key - Pointer to key used in encryption/decryption operation
*           uint8_t *p_src - Pointer to input stream to be encrypted/decrypted
*           uint32_t src_len - Length of input stream to be encrypted/decrypted
*           uint8_t *p_iv - Pointer to initialization vector to use
*           uint32_t iv_len - Length of initialization vector
*           uint8_t *p_aad - Pointer to input stream of additional authentication data
*           uint32_t aad_len - Length of additional authentication data stream
*           sgx_aes_ccm_128bit_tag_t *p_in_mac - Pointer to expected MAC in decryption process
*   Output: uint8_t *p_dst - Pointer to cipher text. Size of buffer should be >= src_len.
*           sgx_aes_ccm_128bit_tag_t *p_out_mac - Pointer to MAC generated from encryption process
* NOTE: Wrapper is responsible for confirming decryption tag matches encryption tag */
sgx_status_t sgx_aes_ccm128_encrypt(const sgx_aes_ccm_128bit_key_t *p_key, const uint8_t *p_src, uint32_t src_len,
                                    uint8_t *p_dst, const uint8_t *p_iv, uint32_t iv_len, const uint8_t *p_aad, uint32_t aad_len,
                                    sgx_aes_ccm_128bit_tag_t *p_out_mac)
{
    IppStatus error_code = ippStsNoErr;
    IppsAES_CCMState* pState = NULL;
    int ippStateSize = 0;

    if ((p_key == NULL) || ((src_len > 0) && (p_dst == NULL)) || ((src_len > 0) && (p_src == NULL))
        || (p_out_mac == NULL) || (iv_len != SGX_AESCCM_IV_SIZE) || ((aad_len > 0) && (p_aad == NULL))
        || (p_iv == NULL) || ((p_src == NULL) && (p_aad == NULL)))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }
    error_code = ippsAES_CCMGetSize(&ippStateSize);
    if (error_code != ippStsNoErr)
    {
        return SGX_ERROR_UNEXPECTED;
    }
    pState = (IppsAES_CCMState*)malloc(ippStateSize);
    if (pState == NULL)
    {
        return SGX_ERROR_OUT_OF_MEMORY;
    }
    error_code = ippsAES_CCMInit((const Ipp8u *)p_key, SGX_AESCCM_KEY_SIZE, pState, ippStateSize);
    if (error_code != ippStsNoErr)
    {
        // Clear temp State before free.
        memset_s(pState, ippStateSize, 0, ippStateSize);
        free(pState);
        switch (error_code)
        {
        case ippStsMemAllocErr: return SGX_ERROR_OUT_OF_MEMORY;
        case ippStsNullPtrErr:
        case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
        default: return SGX_ERROR_UNEXPECTED;
        }
    }
    error_code = ippsAES_CCMStart(p_iv, SGX_AESCCM_IV_SIZE, p_aad, aad_len, pState);
    if (error_code != ippStsNoErr)
    {
        // Clear temp State before free.
        memset_s(pState, ippStateSize, 0, ippStateSize);
        free(pState);
        switch (error_code)
        {
        case ippStsNullPtrErr:
        case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
        default: return SGX_ERROR_UNEXPECTED;
        }
    }
    error_code = ippsAES_CCMTagLen(SGX_AESCCM_MAC_SIZE, pState);
    if (error_code != ippStsNoErr)
    {
        // Clear temp State before free.
        memset_s(pState, ippStateSize, 0, ippStateSize);
        free(pState);
        switch (error_code)
        {
        case ippStsNullPtrErr:
        case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
        default: return SGX_ERROR_UNEXPECTED;
        }
    }
    if (src_len > 0) {
        error_code = ippsAES_CCMMessageLen((Ipp64u)src_len, pState);
        if (error_code != ippStsNoErr)
        {
            // Clear temp State before free.
            memset_s(pState, ippStateSize, 0, ippStateSize);
            free(pState);
            switch (error_code)
            {
            case ippStsNullPtrErr: return SGX_ERROR_INVALID_PARAMETER;
            default: return SGX_ERROR_PCL_NOT_ENCRYPTED;
            }
        }
        error_code = ippsAES_CCMEncrypt(p_src, p_dst, src_len, pState);
        if (error_code != ippStsNoErr)
        {
            // Clear temp State before free.
            memset_s(pState, ippStateSize, 0, ippStateSize);
            free(pState);
            switch (error_code)
            {
            case ippStsNullPtrErr:
            case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
            default: return SGX_ERROR_FILE_BAD_STATUS;
            }
        }
    }
    error_code = ippsAES_CCMGetTag((Ipp8u *)p_out_mac, SGX_AESCCM_MAC_SIZE, pState);
    if (error_code != ippStsNoErr)
    {
        // Clear temp State before free.
        memset_s(p_dst, src_len, 0, src_len);
        memset_s(pState, ippStateSize, 0, ippStateSize);
        free(pState);
        switch (error_code)
        {
        case ippStsNullPtrErr:
        case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
        default: return SGX_ERROR_UNEXPECTED;
        }
    }
    // Clear temp State before free.
    memset_s(pState, ippStateSize, 0, ippStateSize);
    free(pState);
    return SGX_SUCCESS;
}

sgx_status_t sgx_aes_ccm128_decrypt(const sgx_aes_ccm_128bit_key_t *p_key, const uint8_t *p_src,
                                    uint32_t src_len, uint8_t *p_dst, const uint8_t *p_iv, uint32_t iv_len,
                                    const uint8_t *p_aad, uint32_t aad_len, const sgx_aes_ccm_128bit_tag_t *p_in_mac)
{
    IppStatus error_code = ippStsNoErr;
    uint8_t l_tag[SGX_AESCCM_MAC_SIZE];
    IppsAES_CCMState* pState = NULL;
    int ippStateSize = 0;

    if ((p_key == NULL) || ((src_len > 0) && (p_dst == NULL)) || ((src_len > 0) && (p_src == NULL))
        || (p_in_mac == NULL) || (iv_len != SGX_AESCCM_IV_SIZE) || ((aad_len > 0) && (p_aad == NULL))
        || (p_iv == NULL) || ((p_src == NULL) && (p_aad == NULL)))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }

    // Autenthication Tag returned by Decrypt to be compared with Tag created during seal
    memset(&l_tag, 0, SGX_AESCCM_MAC_SIZE);
    error_code = ippsAES_CCMGetSize(&ippStateSize);
    if (error_code != ippStsNoErr)
    {
        return SGX_ERROR_UNEXPECTED;
    }
    pState = (IppsAES_CCMState*)malloc(ippStateSize);
    if (pState == NULL)
    {
        return SGX_ERROR_OUT_OF_MEMORY;
    }
    error_code = ippsAES_CCMInit((const Ipp8u *)p_key, SGX_AESCCM_KEY_SIZE, pState, ippStateSize);
    if (error_code != ippStsNoErr)
    {
        // Clear temp State before free.
        memset_s(pState, ippStateSize, 0, ippStateSize);
        free(pState);
        switch (error_code)
        {
        case ippStsMemAllocErr: return SGX_ERROR_OUT_OF_MEMORY;
        case ippStsNullPtrErr:
        case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
        default: return SGX_ERROR_UNEXPECTED;
        }
    }
    error_code = ippsAES_CCMStart(p_iv, SGX_AESCCM_IV_SIZE, p_aad, aad_len, pState);
    if (error_code != ippStsNoErr)
    {
        // Clear temp State before free.
        memset_s(pState, ippStateSize, 0, ippStateSize);
        free(pState);
        switch (error_code)
        {
        case ippStsNullPtrErr:
        case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
        default: return SGX_ERROR_UNEXPECTED;
        }
    }
    error_code = ippsAES_CCMTagLen(SGX_AESCCM_MAC_SIZE, pState);
    if (error_code != ippStsNoErr)
    {
        // Clear temp State before free.
        memset_s(pState, ippStateSize, 0, ippStateSize);
        free(pState);
        switch (error_code)
        {
        case ippStsNullPtrErr:
        case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
        default: return SGX_ERROR_UNEXPECTED;
        }
    }
    if (src_len > 0) {
        error_code = ippsAES_CCMMessageLen((Ipp64u)src_len, pState);
        if (error_code != ippStsNoErr)
        {
            // Clear temp State before free.
            memset_s(pState, ippStateSize, 0, ippStateSize);
            free(pState);
            switch (error_code)
            {
            case ippStsNullPtrErr: return SGX_ERROR_INVALID_PARAMETER;
            default: return SGX_ERROR_UNEXPECTED;
            }
        }
        error_code = ippsAES_CCMDecrypt(p_src, p_dst, src_len, pState);
        if (error_code != ippStsNoErr)
        {
            // Clear temp State before free.
            memset_s(pState, ippStateSize, 0, ippStateSize);
            free(pState);
            switch (error_code)
            {
            case ippStsNullPtrErr: return SGX_ERROR_INVALID_PARAMETER;
            default: return SGX_ERROR_UNEXPECTED;
            }
        }
    }
    error_code = ippsAES_CCMGetTag((Ipp8u *)l_tag, SGX_AESCCM_MAC_SIZE, pState);
    if (error_code != ippStsNoErr)
    {
        // Clear temp State before free.
        memset_s(p_dst, src_len, 0, src_len);
        memset_s(pState, ippStateSize, 0, ippStateSize);
        free(pState);
        switch (error_code)
        {
        case ippStsNullPtrErr:
        case ippStsLengthErr: return SGX_ERROR_INVALID_PARAMETER;
        default: return SGX_ERROR_UNEXPECTED;
        }
    }
    // Clear temp State before free.
    memset_s(pState, ippStateSize, 0, ippStateSize);
    free(pState);

    // Verify current data tag = data tag generated when sealing the data blob
    if (consttime_memequal(p_in_mac, &l_tag, SGX_AESCCM_MAC_SIZE) == 0)
    {
        memset_s(p_dst, src_len, 0, src_len);
        memset_s(&l_tag, SGX_AESCCM_MAC_SIZE, 0, SGX_AESCCM_MAC_SIZE);
        return SGX_ERROR_MAC_MISMATCH;
    }

    memset_s(&l_tag, SGX_AESCCM_MAC_SIZE, 0, SGX_AESCCM_MAC_SIZE);
    return SGX_SUCCESS;
}

sgx_status_t sgx_aes_ccm128_init(const uint8_t *key, const uint8_t *iv, uint32_t iv_len, const uint8_t *aad,
    uint32_t aad_len, sgx_aes_state_handle_t* aes_ccm_state)
{
    if ((aad_len >= INT_MAX) || (key == NULL) || (iv_len != SGX_AESCCM_IV_SIZE) || ((aad_len > 0) && (aad == NULL))
        || (iv == NULL) || (aes_ccm_state == NULL))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }
    int state_size = 0;
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    IppStatus status = ippStsNoErr;
    IppsAES_CCMState *p_state = NULL;

    do {
        status = ippsAES_CCMGetSize(&state_size);
        ERROR_BREAK(status);

        p_state = reinterpret_cast<IppsAES_CCMState *>(malloc(state_size));
        if (p_state == NULL) {
            ret = SGX_ERROR_OUT_OF_MEMORY;
            break;
        }

        status = ippsAES_CCMInit(key, SGX_AESCCM_KEY_SIZE, p_state, state_size);
        ERROR_BREAK(status);

        status = ippsAES_CCMStart(iv, iv_len, aad, aad_len, p_state);
        ERROR_BREAK(status);

        status = ippsAES_CCMTagLen(SGX_AESCCM_MAC_SIZE, p_state);
        ERROR_BREAK(status);

        status = ippsAES_CCMMessageLen(IPP_MAX_64U, p_state);
        ERROR_BREAK(status);

        *aes_ccm_state = p_state;

        ret = SGX_SUCCESS;
    } while (0);

    if (ret != SGX_SUCCESS) {
        CLEAR_FREE_MEM(p_state, state_size);
    }

    return ret;
}

sgx_status_t sgx_aes_ccm128_enc_get_mac(uint8_t *mac, sgx_aes_state_handle_t aes_ccm_state)
{
    if ((mac == NULL) || (aes_ccm_state == NULL))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }
    sgx_status_t ret = SGX_ERROR_UNEXPECTED;
    IppStatus status = ippsAES_CCMGetTag(mac, SGX_AESCCM_MAC_SIZE, (IppsAES_CCMState*)aes_ccm_state);
    if (status == ippStsNoErr) {
        ret = SGX_SUCCESS;
    }

    //In case of error, clear output MAC buffer.
    //
    if (ret != SGX_SUCCESS)
        memset_s(mac, SGX_AESCCM_MAC_SIZE, 0, SGX_AESCCM_MAC_SIZE);

    return ret;
}

sgx_status_t sgx_aes_ccm128_dec_verify_mac(uint8_t *mac, sgx_aes_state_handle_t aes_ccm_state)
{
    if ((mac == NULL) || (aes_ccm_state == NULL))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }
    uint8_t l_tag[SGX_AESCCM_MAC_SIZE];
    IppStatus status = ippsAES_CCMGetTag(l_tag, SGX_AESCCM_MAC_SIZE, (IppsAES_CCMState*)aes_ccm_state);
    if (status != ippStsNoErr) {
        return SGX_ERROR_UNEXPECTED;
    }

    if (consttime_memequal(mac, &l_tag, SGX_AESCCM_MAC_SIZE) == 0)
    {
        memset_s(&l_tag, SGX_AESCCM_MAC_SIZE, 0, SGX_AESCCM_MAC_SIZE);
        return SGX_ERROR_MAC_MISMATCH;
    }
    
    memset_s(&l_tag, SGX_AESCCM_MAC_SIZE, 0, SGX_AESCCM_MAC_SIZE);
    return SGX_SUCCESS;
}

//aes_ccm encryption fini function
sgx_status_t sgx_aes_ccm_close(sgx_aes_state_handle_t aes_ccm_state)
{
    if (aes_ccm_state == NULL)
        return SGX_ERROR_INVALID_PARAMETER;
    
    int state_size = 0;
    if (ippsAES_CCMGetSize(&state_size) != ippStsNoErr) {
        free(aes_ccm_state);
        return SGX_SUCCESS;
    }
    CLEAR_FREE_MEM(aes_ccm_state, state_size);
    return SGX_SUCCESS;
}

sgx_status_t sgx_aes_ccm128_enc_update(uint8_t *p_src, uint32_t src_len,
    uint8_t *p_dst, sgx_aes_state_handle_t aes_ccm_state)
{
    if ((aes_ccm_state == NULL) || (p_src == NULL) || (p_dst == NULL) || (src_len >= INT_MAX) || (src_len == 0))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }
    if (ippsAES_CCMEncrypt(p_src, p_dst, src_len, (IppsAES_CCMState*)aes_ccm_state) != ippStsNoErr) {
        return SGX_ERROR_UNEXPECTED;
    }
    return SGX_SUCCESS;
}

sgx_status_t sgx_aes_ccm128_dec_update(uint8_t *p_src, uint32_t src_len,
    uint8_t *p_dst, sgx_aes_state_handle_t aes_ccm_state)
{
    if ((aes_ccm_state == NULL) || (p_src == NULL) || (p_dst == NULL) || (src_len >= INT_MAX) || (src_len == 0))
    {
        return SGX_ERROR_INVALID_PARAMETER;
    }
    if (ippsAES_CCMDecrypt(p_src, p_dst, src_len, (IppsAES_CCMState*)aes_ccm_state) != ippStsNoErr) {
        return SGX_ERROR_UNEXPECTED;
    }
    return SGX_SUCCESS;
}
