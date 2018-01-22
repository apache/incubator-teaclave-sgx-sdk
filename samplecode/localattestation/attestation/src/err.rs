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

impl_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum ATTESTATION_STATUS {
        SUCCESS                 = 0x00,
        INVALID_PARAMETER       = 0xE1,
        VALID_SESSION           = 0xE2,
        INVALID_SESSION         = 0xE3,
        ATTESTATION_ERROR       = 0xE4,
        ATTESTATION_SE_ERROR    = 0xE5,
        IPP_ERROR               = 0xE6,
        NO_AVAILABLE_SESSION_ERROR = 0xE7,
        MALLOC_ERROR            = 0xE8,
        ERROR_TAG_MISMATCH      =  0xE9,
        OUT_BUFFER_LENGTH_ERROR = 0xEA,
        INVALID_REQUEST_TYPE_ERROR = 0xEB,
        INVALID_PARAMETER_ERROR = 0xEC,
        ENCLAVE_TRUST_ERROR     = 0xED,
        ENCRYPT_DECRYPT_ERROR   = 0xEE,
        DUPLICATE_SESSION       = 0xEF,
        UNKNOWN_ERROR           = 0xF0,
    }
}