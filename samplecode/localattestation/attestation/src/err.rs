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