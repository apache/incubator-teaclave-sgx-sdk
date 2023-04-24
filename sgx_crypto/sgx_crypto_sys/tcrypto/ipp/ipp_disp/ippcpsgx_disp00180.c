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

#include "ippcp.h"

#define IPPFUN(type,name,arg) extern type IPP_STDCALL name arg

#ifndef NULL
#ifdef  __cplusplus
#define NULL    0
#else
#define NULL    ((void *)0)
#endif
#endif

#if defined (_M_AMD64) || defined (__x86_64__)


#define AVX3I_FEATURES ( ippCPUID_SHA|ippCPUID_AVX512VBMI|ippCPUID_AVX512VBMI2|ippCPUID_AVX512IFMA|ippCPUID_AVX512GFNI|ippCPUID_AVX512VAES|ippCPUID_AVX512VCLMUL )
#define AVX3X_FEATURES ( ippCPUID_AVX512F|ippCPUID_AVX512CD|ippCPUID_AVX512VL|ippCPUID_AVX512BW|ippCPUID_AVX512DQ )
#define AVX3M_FEATURES ( ippCPUID_AVX512F|ippCPUID_AVX512CD|ippCPUID_AVX512PF|ippCPUID_AVX512ER )


IPPAPI( const IppsHashMethod*, y8_ippsHashMethod_SHA256, (void) )
IPPAPI( const IppsHashMethod*, l9_ippsHashMethod_SHA256, (void) )
IPPAPI( const IppsHashMethod*, k1_ippsHashMethod_SHA256, (void) )

IPPFUN( const IppsHashMethod*,sgx_disp_ippsHashMethod_SHA256, (void) )
{
  Ipp64u features;
  ippcpGetCpuFeatures( &features );

      if( AVX3I_FEATURES  == ( features & AVX3I_FEATURES  )) {
        return k1_ippsHashMethod_SHA256(  );
      } else 
      if( ippCPUID_AVX2  == ( features & ippCPUID_AVX2  )) {
        return l9_ippsHashMethod_SHA256(  );
      } else 
      if( ippCPUID_SSE42 == ( features & ippCPUID_SSE42 )) {
        return y8_ippsHashMethod_SHA256(  );
      } else 
        return NULL;
}
#else


IPPAPI( const IppsHashMethod*, p8_ippsHashMethod_SHA256, (void) )
IPPAPI( const IppsHashMethod*, h9_ippsHashMethod_SHA256, (void) )

IPPFUN( const IppsHashMethod*,sgx_disp_ippsHashMethod_SHA256, (void) )
{
  Ipp64u features;
  ippcpGetCpuFeatures( &features );

      if( ippCPUID_AVX2  == ( features & ippCPUID_AVX2  )) {
        return h9_ippsHashMethod_SHA256(  );
      } else 
      if( ippCPUID_SSE42 == ( features & ippCPUID_SSE42 )) {
        return p8_ippsHashMethod_SHA256(  );
      } else 
        return NULL;
}
#endif