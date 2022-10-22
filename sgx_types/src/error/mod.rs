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

use core::error::Error;
use core::fmt;
use core::result;

pub mod errno;

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum SgxStatus {
        Success                 = 0x0000_0000,

        Unexpected              = 0x0000_0001,      /* Unexpected error. */
        InvalidParameter        = 0x0000_0002,      /* The parameter is incorrect. */
        OutOfMemory             = 0x0000_0003,      /* Not enough memory is available to complete this operation. */
        EnclaveLost             = 0x0000_0004,      /* Enclave lost after power transition or used in child process created by linux:fork(). */
        InvalidState            = 0x0000_0005,      /* SGX API is invoked in incorrect order or state. */
        UnsupportedFeature      = 0x0000_0008,      /* Feature is not supported on this platform. */
        ThreadExit              = 0x0000_0009,      /* Enclave is exited with pthread_exit(). */
        MemoryMapFailure        = 0x0000_000A,      /* Failed to reserve memory for the enclave. */

        InvalidFunction         = 0x0000_1001,      /* The ecall/ocall index is invalid. */
        OutOfTcs                = 0x0000_1003,      /* The enclave is out of TCS. */
        EnclaveCrashed          = 0x0000_1006,      /* The enclave is crashed. */
        ECallNotAllowed         = 0x0000_1007,      /* The ECALL is not allowed at this time, e.g. ecall is blocked by the dynamic entry table, or nested ecall is not allowed during initialization. */
        OCallNotAllowed         = 0x0000_1008,      /* The OCALL is not allowed at this time, e.g. ocall is not allowed during exception handling. */
        StackOverRun            = 0x0000_1009,      /* The enclave is running out of stack. */

        UndefinedSymbol         = 0x0000_2000,      /* The enclave image has undefined symbol. */
        InvalidEnclave          = 0x0000_2001,      /* The enclave image is not correct. */
        InvalidEcnalveId        = 0x0000_2002,      /* The enclave id is invalid. */
        InvalidSignature        = 0x0000_2003,      /* The signature is invalid. */
        NotDebugEnclave         = 0x0000_2004,      /* The enclave is signed as product enclave, and can not be created as debuggable enclave. */
        OutOfEPC                = 0x0000_2005,      /* Not enough EPC is available to load the enclave. */
        NoDevice                = 0x0000_2006,      /* Can't open SGX device. */
        MemoryMapConflict       = 0x0000_2007,      /* Page mapping failed in driver. */
        InvalidMetadata         = 0x0000_2009,      /* The metadata is incorrect. */
        DeviceBusy              = 0x0000_200C,      /* Device is busy, mostly EINIT failed. */
        InvalidVersion          = 0x0000_200D,      /* Metadata version is inconsistent between uRTS and sgx_sign or uRTS is incompatible with current platform. */
        ModeIncompatible        = 0x0000_200E,      /* The target enclave 32/64 bit mode or sim/hw mode is incompatible with the mode of current uRTS. */
        EnclaveFileAccess       = 0x0000_200F,      /* Can't open enclave file. */
        InvalidMisc             = 0x0000_2010,      /* The MiscSelct/MiscMask settings are not correct. */
        InvalidLaunchToken      = 0x0000_2011,      /* The launch token is not correct. */

        MacMismatch             = 0x0000_3001,      /* Indicates verification error for reports, sealed datas, etc. */
        InvalidAttribute        = 0x0000_3002,      /* The enclave is not authorized, e.g., requesting invalid attribute or launch key access on legacy SGX platform without FLC. */
        InvalidCpusvn           = 0x0000_3003,      /* The cpu svn is beyond platform's cpu svn value. */
        InvalidIsvsvn           = 0x0000_3004,      /* The isv svn is greater than the enclave's isv svn. */
        InvalidKeyname          = 0x0000_3005,      /* The key name is an unsupported value. */

        ServiceUnavailable      = 0x0000_4001,      /* Indicates aesm didn't respond or the requested service is not supported. */
        ServiceTimeout          = 0x0000_4002,      /* The request to aesm timed out. */
        InvalidEpidBlob         = 0x0000_4003,      /* Indicates epid blob verification error. */
        ServiceInvalidPrivilege = 0x0000_4004,      /* Enclave not authorized to run, .e.g. provisioning enclave hosted in an app without access rights to /dev/sgx_provision. */
        EpidMemoryRevoked       = 0x0000_4005,      /* The EPID group membership is revoked. */
        UpdateNeeded            = 0x0000_4006,      /* SGX needs to be updated. */
        NetworkFailure          = 0x0000_4007,      /* Network connecting or proxy setting issue is encountered. */
        InvalidAeSession        = 0x0000_4008,      /* Session is invalid or ended by server. */
        ServiceBusy             = 0x0000_400A,      /* The requested service is temporarily not availabe. */
        McNotFound              = 0x0000_400C,      /* The Monotonic Counter doesn't exist or has been invalided. */
        McNoAccess              = 0x0000_400D,      /* Caller doesn't have the access right to specified VMC. */
        McUsedUp                = 0x0000_400E,      /* Monotonic counters are used out. */
        McOverQuota             = 0x0000_400F,      /* Monotonic counters exceeds quota limitation. */
        KdfMismatch             = 0x0000_4011,      /* Key derivation function doesn't match during key exchange. */
        UnrecognizedPlatform    = 0x0000_4012,      /* EPID Provisioning failed due to platform not recognized by backend server. */
        UnsupportedConfig       = 0x0000_4013,      /* The config for trigging EPID Provisiong or PSE Provisiong&LTP is invalid. */

        NoPrivilege             = 0x0000_5002,      /* Not enough privilege to perform the operation. */

        /* SGX Protected Code Loader Error codes*/
        PclEncrypted            = 0x0000_6001,      /* trying to encrypt an already encrypted enclave. */
        PclNotEncrypted         = 0x0000_6002,      /* trying to load a plain enclave using sgx_create_encrypted_enclave. */
        PclMacMismatch          = 0x0000_6003,      /* section mac result does not match build time mac. */
        PclShaMismatch          = 0x0000_6004,      /* Unsealed key MAC does not match MAC of key hardcoded in enclave binary. */
        PclGuidMismatch         = 0x0000_6005,      /* GUID in sealed blob does not match GUID hardcoded in enclave binary. */

        /* SGX errors are only used in the file API when there is no appropriate EXXX (EINVAL, EIO etc.) error code. */
        BadStatus               = 0x0000_7001,	    /* The file is in bad status, run sgx_clearerr to try and fix it. */
        NoKeyId                 = 0x0000_7002,	    /* The Key ID field is all zeros, can't re-generate the encryption key. */
        NameMismatch            = 0x0000_7003,	    /* The current file name is different then the original file name (not allowed, substitution attack). */
        NotSgxFile              = 0x0000_7004,      /* The file is not an SGX file. */
        CantOpenRecoveryFile    = 0x0000_7005,	    /* A recovery file can't be opened, so flush operation can't continue (only used when no EXXX is returned). */
        CantWriteRecoveryFile   = 0x0000_7006,      /* A recovery file can't be written, so flush operation can't continue (only used when no EXXX is returned). */
        RecoveryNeeded          = 0x0000_7007,	    /* When openeing the file, recovery is needed, but the recovery process failed. */
        FluchFailed             = 0x0000_7008,	    /* fflush operation (to disk) failed (only used when no EXXX is returned). */
        CloseFailed             = 0x0000_7009,	    /* fclose operation (to disk) failed (only used when no EXXX is returned). */

        UnsupportedAttKeyid     = 0x0000_8001,      /* platform quoting infrastructure does not support the key. */
        AttKeyCertFailed        = 0x0000_8002,      /* Failed to generate and certify the attestation key. */
        AttKeyUninitialized     = 0x0000_8003,      /* The platform quoting infrastructure does not have the attestation key available to generate quote. */
        InvaliedAttKeyCertData  = 0x0000_8004,      /* TThe data returned by the platform library's sgx_get_quote_config() is invalid. */
        INvaliedPlatfromCert    = 0x0000_8005,      /* The PCK Cert for the platform is not available. */

        EnclaveCreateInterrupted = 0x0000_F001,     /* The ioctl for enclave_create unexpectedly failed with EINTR. */
    }
}

impl SgxStatus {
    #[inline]
    pub fn is_success(&self) -> bool {
        *self == SgxStatus::Success
    }
}

impl SgxStatus {
    pub fn __description(&self) -> &'static str {
        match *self {
            SgxStatus::Success => "Success.",
            SgxStatus::Unexpected => "Unexpected error occurred.",
            SgxStatus::InvalidParameter => "The parameter is incorrect.",
            SgxStatus::OutOfMemory => "Not enough memory is available to complete this operation.",
            SgxStatus::EnclaveLost => "Enclave lost after power transition or used in child process created.",
            SgxStatus::InvalidState => "SGX API is invoked in incorrect order or state.",
            SgxStatus::UnsupportedFeature => "Feature is not supported on this platform.",
            SgxStatus::ThreadExit => "Enclave is exited with pthread_exit.",
            SgxStatus::MemoryMapFailure => "Failed to reserve memory for the enclave.",

            SgxStatus::InvalidFunction => "The ecall/ocall index is invalid.",
            SgxStatus::OutOfTcs => "The enclave is out of TCS.",
            SgxStatus::EnclaveCrashed => "The enclave is crashed.",
            SgxStatus::ECallNotAllowed => "The ECALL is not allowed at this time.",
            SgxStatus::OCallNotAllowed => "The OCALL is not allowed at this time.",
            SgxStatus::StackOverRun => "The enclave is running out of stack.",

            SgxStatus::UndefinedSymbol => "The enclave image has undefined symbol.",
            SgxStatus::InvalidEnclave => "The enclave image is not correct.",
            SgxStatus::InvalidEcnalveId => "The enclave id is invalid.",
            SgxStatus::InvalidSignature => "The signature is invalid.",
            SgxStatus::NotDebugEnclave => "The enclave can not be created as debuggable enclave.",
            SgxStatus::OutOfEPC => "Not enough EPC is available to load the enclave.",
            SgxStatus::NoDevice => "Can't open SGX device.",
            SgxStatus::MemoryMapConflict => "Page mapping failed in driver.",
            SgxStatus::InvalidMetadata => "The metadata is incorrect.",
            SgxStatus::DeviceBusy => "Device is busy, mostly EINIT failed.",
            SgxStatus::InvalidVersion => "Enclave version was invalid.",
            SgxStatus::ModeIncompatible => "The target enclave mode is incompatible with the mode of current uRTS.",
            SgxStatus::EnclaveFileAccess => "Can't open enclave file.",
            SgxStatus::InvalidMisc => "The MiscSelct/MiscMask settings are not correct.",
            SgxStatus::InvalidLaunchToken => "The launch token is not correct.",

            SgxStatus::MacMismatch => "Indicates verification error.",
            SgxStatus::InvalidAttribute => "The enclave is not authorized.",
            SgxStatus::InvalidCpusvn => "The cpu svn is beyond platform's cpu svn value.",
            SgxStatus::InvalidIsvsvn => "The isv svn is greater than the enclave's isv svn.",
            SgxStatus::InvalidKeyname => "The key name is an unsupported value.",

            SgxStatus::ServiceUnavailable => "Indicates aesm didn't response or the requested service is not supported.",
            SgxStatus::ServiceTimeout => "The request to aesm time out.",
            SgxStatus::InvalidEpidBlob => "Indicates epid blob verification error.",
            SgxStatus::ServiceInvalidPrivilege => "Enclave has no privilege to get launch token.",
            SgxStatus::EpidMemoryRevoked => "The EPID group membership is revoked.",
            SgxStatus::UpdateNeeded => "SGX needs to be updated.",
            SgxStatus::NetworkFailure => "Network connecting or proxy setting issue is encountered.",
            SgxStatus::InvalidAeSession => "Session is invalid or ended by server.",
            SgxStatus::ServiceBusy => "The requested service is temporarily not availabe.",
            SgxStatus::McNotFound => "The Monotonic Counter doesn't exist or has been invalided.",
            SgxStatus::McNoAccess => "Caller doesn't have the access right to specified VMC.",
            SgxStatus::McUsedUp => "Monotonic counters are used out.",
            SgxStatus::McOverQuota => "Monotonic counters exceeds quota limitation.",
            SgxStatus::KdfMismatch => "Key derivation function doesn't match during key exchange.",
            SgxStatus::UnrecognizedPlatform => "EPID Provisioning failed due to platform not recognized by backend server.",
            SgxStatus::UnsupportedConfig => "The config for trigging EPID Provisiong or PSE Provisiong&LTP is invalid.",
            SgxStatus::NoPrivilege => "Not enough privilege to perform the operation.",

            SgxStatus::PclEncrypted => "Trying to encrypt an already encrypted enclave.",
            SgxStatus::PclNotEncrypted => "Trying to load a plain enclave using sgx_create_encrypted_enclave.",
            SgxStatus::PclMacMismatch => "Section mac result does not match build time mac.",
            SgxStatus::PclShaMismatch => "Unsealed key MAC does not match MAC of key hardcoded in enclave binary.",
            SgxStatus::PclGuidMismatch => "GUID in sealed blob does not match GUID hardcoded in enclave binary.",

            SgxStatus::BadStatus => "The file is in bad status.",
            SgxStatus::NoKeyId => "The Key ID field is all zeros, can't regenerate the encryption key.",
            SgxStatus::NameMismatch => "The current file name is different then the original file name.",
            SgxStatus::NotSgxFile => "The file is not an SGX file.",
            SgxStatus::CantOpenRecoveryFile => "A recovery file can't be opened, so flush operation can't continue.",
            SgxStatus::CantWriteRecoveryFile => "A recovery file can't be written, so flush operation can't continue.",
            SgxStatus::RecoveryNeeded => "When openeing the file, recovery is needed, but the recovery process failed.",
            SgxStatus::FluchFailed => "fflush operation failed.",
            SgxStatus::CloseFailed => "fclose operation failed.",

            SgxStatus::UnsupportedAttKeyid => "platform quoting infrastructure does not support the key.",
            SgxStatus::AttKeyCertFailed => "Failed to generate and certify the attestation key.",
            SgxStatus::AttKeyUninitialized => "The platform quoting infrastructure does not have the attestation key available to generate quote.",
            SgxStatus::InvaliedAttKeyCertData => "The data returned by the platform library is invalid.",
            SgxStatus::INvaliedPlatfromCert => "The PCK Cert for the platform is not available.",

            SgxStatus::EnclaveCreateInterrupted => "The ioctl for enclave_create unexpectedly failed with EINTR.",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            SgxStatus::Success => "Success.",
            SgxStatus::Unexpected => "Unexpected",
            SgxStatus::InvalidParameter => "InvalidParameter",
            SgxStatus::OutOfMemory => "OutOfMemory",
            SgxStatus::EnclaveLost => "EnclaveLost",
            SgxStatus::InvalidState => "InvalidState",
            SgxStatus::UnsupportedFeature => "UnsupportedFeature",
            SgxStatus::ThreadExit => "ThreadExit",
            SgxStatus::MemoryMapFailure => "MemoryMapFailure",

            SgxStatus::InvalidFunction => "InvalidFunction",
            SgxStatus::OutOfTcs => "OutOfTcs",
            SgxStatus::EnclaveCrashed => "EnclaveCrashed",
            SgxStatus::ECallNotAllowed => "ECallNotAllowed",
            SgxStatus::OCallNotAllowed => "OCallNotAllowed",
            SgxStatus::StackOverRun => "StackOverRun",

            SgxStatus::UndefinedSymbol => "UndefinedSymbol",
            SgxStatus::InvalidEnclave => "InvalidEnclave",
            SgxStatus::InvalidEcnalveId => "InvalidEcnalveId",
            SgxStatus::InvalidSignature => "InvalidSignature",
            SgxStatus::NotDebugEnclave => "NotDebugEnclave",
            SgxStatus::OutOfEPC => "OutOfEPC",
            SgxStatus::NoDevice => "NoDevice",
            SgxStatus::MemoryMapConflict => "MemoryMapConflict",
            SgxStatus::InvalidMetadata => "InvalidMetadata",
            SgxStatus::DeviceBusy => "DeviceBusy",
            SgxStatus::InvalidVersion => "InvalidVersion",
            SgxStatus::ModeIncompatible => "ModeIncompatible",
            SgxStatus::EnclaveFileAccess => "EnclaveFileAccess",
            SgxStatus::InvalidMisc => "InvalidMisc",
            SgxStatus::InvalidLaunchToken => "InvalidLaunchToken",

            SgxStatus::MacMismatch => "MacMismatch",
            SgxStatus::InvalidAttribute => "InvalidAttribute",
            SgxStatus::InvalidCpusvn => "InvalidCpusvn",
            SgxStatus::InvalidIsvsvn => "InvalidIsvsvn",
            SgxStatus::InvalidKeyname => "InvalidKeyname",

            SgxStatus::ServiceUnavailable => "ServiceUnavailable",
            SgxStatus::ServiceTimeout => "ServiceTimeout",
            SgxStatus::InvalidEpidBlob => "InvalidEpidBlob",
            SgxStatus::ServiceInvalidPrivilege => "ServiceInvalidPrivilege",
            SgxStatus::EpidMemoryRevoked => "EpidMemoryRevoked",
            SgxStatus::UpdateNeeded => "UpdateNeeded",
            SgxStatus::NetworkFailure => "NetworkFailure",
            SgxStatus::InvalidAeSession => "InvalidAeSession",
            SgxStatus::ServiceBusy => "ServiceBusy",
            SgxStatus::McNotFound => "McNotFound",
            SgxStatus::McNoAccess => "McNoAccess",
            SgxStatus::McUsedUp => "McUsedUp",
            SgxStatus::McOverQuota => "McOverQuota",
            SgxStatus::KdfMismatch => "KdfMismatch",
            SgxStatus::UnrecognizedPlatform => "UnrecognizedPlatform",
            SgxStatus::UnsupportedConfig => "UnsupportedConfig",
            SgxStatus::NoPrivilege => "NoPrivilege",

            SgxStatus::PclEncrypted => "PclEncrypted",
            SgxStatus::PclNotEncrypted => "PclNotEncrypted",
            SgxStatus::PclMacMismatch => "PclMacMismatch",
            SgxStatus::PclShaMismatch => "PclShaMismatch",
            SgxStatus::PclGuidMismatch => "PclGuidMismatch",

            SgxStatus::BadStatus => "BadStatus",
            SgxStatus::NoKeyId => "NoKeyId",
            SgxStatus::NameMismatch => "NameMismatch",
            SgxStatus::NotSgxFile => "NotSgxFile",
            SgxStatus::CantOpenRecoveryFile => "CantOpenRecoveryFile",
            SgxStatus::CantWriteRecoveryFile => "CantWriteRecoveryFile",
            SgxStatus::RecoveryNeeded => "RecoveryNeeded",
            SgxStatus::FluchFailed => "FluchFailed",
            SgxStatus::CloseFailed => "CloseFailed",

            SgxStatus::UnsupportedAttKeyid => "UnsupportedAttKeyid",
            SgxStatus::AttKeyCertFailed => "AttKeyCertFailed",
            SgxStatus::AttKeyUninitialized => "AttKeyUninitialized",
            SgxStatus::InvaliedAttKeyCertData => "InvaliedAttKeyCertData",
            SgxStatus::INvaliedPlatfromCert => "INvaliedPlatfromCert",

            SgxStatus::EnclaveCreateInterrupted => "EnclaveCreateInterrupted",
        }
    }
}

impl fmt::Display for SgxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Error for SgxStatus {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.__description()
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum PceError {
        Success                 = 0x0000_F000,
        Unexpected              = 0x0000_F001,
        InvalidParameter        = 0x0000_F002,
        OutOfEPC                = 0x0000_F003,
        InterfaceUnavailable    = 0x0000_F004,
        InvalidReport           = 0x0000_F005,
        CryptoError             = 0x0000_F006,
        InvalidPrivilege        = 0x0000_F007,
        InvalidTCB              = 0x0000_F008,
    }
}

impl PceError {
    pub fn __description(&self) -> &'static str {
        match *self {
            PceError::Success => "Success.",
            PceError::Unexpected => "Unexpected error.",
            PceError::InvalidParameter => "The parameter is incorrect.",
            PceError::OutOfEPC => "Not enough memory is available to complete this operation.",
            PceError::InterfaceUnavailable => "SGX API is unavailable.",
            PceError::InvalidReport => "The report cannot be verified.",
            PceError::CryptoError => "Cannot decrypt or verify ciphertext.",
            PceError::InvalidPrivilege => "Not enough privilege to perform the operation.",
            PceError::InvalidTCB => "PCE could not sign at the requested TCB.",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            PceError::Success => "Success.",
            PceError::Unexpected => "Unexpected",
            PceError::InvalidParameter => "InvalidParameter",
            PceError::OutOfEPC => "OutOfEPC",
            PceError::InterfaceUnavailable => "InterfaceUnavailable",
            PceError::InvalidReport => "InvalidReport",
            PceError::CryptoError => "CryptoError",
            PceError::InvalidPrivilege => "InvalidPrivilege",
            PceError::InvalidTCB => "InvalidTCB",
        }
    }
}

impl fmt::Display for PceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Quote3Error {
        Success                                 = 0x0000_0000,
        Unexpected                              = 0x0000_E001,
        InvalidParameter                        = 0x0000_E002,
        OutOfMemory                             = 0x0000_E003,
        EcdsaIdMisMatch                         = 0x0000_E004,
        PathNameBufferOverflow                  = 0x0000_E005,
        FileAccessError                         = 0x0000_E006,
        StoredKey                               = 0x0000_E007,
        PubkeyIdMismatch                        = 0x0000_E008,
        InvalidPceSigScheme                     = 0x0000_E009,
        AttKeyBlobError                         = 0x0000_E00A,
        UnsupportedAttKeyId                     = 0x0000_E00B,
        UnsupportedLoadingPolicy                = 0x0000_E00C,
        InterfaceUnavailable                    = 0x0000_E00D,
        PlatformLibUnavailable                  = 0x0000_E00E,
        AttKeyNotInitialized                    = 0x0000_E00F,
        AttKeyCertDataInvalid                   = 0x0000_E010,
        NoPlatformCertData                      = 0x0000_E011,
        OutOfEpc                                = 0x0000_E012,
        ErrorReport                             = 0x0000_E013,
        EnclaveLost                             = 0x0000_E014,
        InvalidReport                           = 0x0000_E015,
        EnclaveLoadError                        = 0x0000_E016,
        UnableToGenerateQeReport                = 0x0000_E017,
        KeyCertifcationError                    = 0x0000_E018,
        NetworkError                            = 0x0000_E019,
        MessageError                            = 0x0000_E01A,
        NoQuoteCollateralData                   = 0x0000_E01B,
        QuoteCertificationDataUnsupported       = 0x0000_E01C,
        QuoteFormatUnsupported                  = 0x0000_E01D,
        UnableToGenerateReport                  = 0x0000_E01E,
        QeReportInvalidSignature                = 0x0000_E01F,
        QeReportUnsupportedFormat               = 0x0000_E020,
        PckCertUnsupportedFormat                = 0x0000_E021,
        PckCertChainError                       = 0x0000_E022,
        TcbInfoUnsupportedFormat                = 0x0000_E023,
        TcbInfoMismatch                         = 0x0000_E024,
        QeIdentityUnsupportedFormat             = 0x0000_E025,
        QeIdentityMismatch                      = 0x0000_E026,
        TcbOutOfDate                            = 0x0000_E027,
        TcbOutOfDateConfigurationNeeded         = 0x0000_E028,
        EnclaveIdentityOutOfDate                = 0x0000_E029,
        EnclaveReportIsvsvnOutOfDate            = 0x0000_E02A,
        QeIdentityOutOfDate                     = 0x0000_E02B,
        TcbInfoExpired                          = 0x0000_E02C,
        PckCertChainExpired                     = 0x0000_E02D,
        CrlExpired                              = 0x0000_E02E,
        SigningCertChainExpired                 = 0x0000_E02F,
        EnclaveIdentityExpired                  = 0x0000_E030,
        PckRevoked                              = 0x0000_E031,
        TcbRevoked                              = 0x0000_E032,
        TcbConfigurationNeeded                  = 0x0000_E033,
        UnableToGetCollateral                   = 0x0000_E034,
        InvalidPrivilege                        = 0x0000_E035,
        NoQveIdentityData                       = 0x0000_E037,
        CrlUnsupportedFormat                    = 0x0000_E038,
        QeIdentityChainError                    = 0x0000_E039,
        TcbInfoChainError                       = 0x0000_E03A,
        QvlQveMismatch                          = 0x0000_E03B,
        TcbSwHardeningNeeded                    = 0x0000_E03C,
        TcbConfigurationAndSwHardeningNeeded    = 0x0000_E03D,
        UnsupportedMode                         = 0x0000_E03E,
        NoDevice                                = 0x0000_E03F,
        ServiceUnavailable                      = 0x0000_E040,
        NetworkFailure                          = 0x0000_E041,
        ServiceTimeout                          = 0x0000_E042,
        ServiceBusy                             = 0x0000_E043,
        UnknownMessageResponse                  = 0x0000_E044,
        PersistentStorageError                  = 0x0000_E045,
        MessageParsingError                     = 0x0000_E046,
        PlatformUnknown                         = 0x0000_E047,
        UnknownApiVersion                       = 0x0000_E048,
        CertsUnavailable                        = 0x0000_E049,
        QveIdentityMismatch                     = 0x0000_E050,
        QveOutOfDate                            = 0x0000_E051,
        PswNotAvailable                         = 0x0000_E052,
        CollateralVersionNotSupported           = 0x0000_E053,
        TdxModuleMismatch                       = 0x0000_E060,
        ErrorMax                                = 0x0000_E0FF,
    }
}

impl Quote3Error {
    pub fn __description(&self) -> &'static str {
        match *self {
            Quote3Error::Success => "Success.",
            Quote3Error::Unexpected => "Unexpected error.",
            Quote3Error::InvalidParameter => "The parameter is incorrect.",
            Quote3Error::OutOfMemory => {
                "Not enough memory is available to complete this operation."
            }
            Quote3Error::EcdsaIdMisMatch => {
                "Expected ECDSA_ID does not match the value stored in the ECDSA Blob."
            }
            Quote3Error::PathNameBufferOverflow => "The ECDSA blob pathname is too large.",
            Quote3Error::FileAccessError => "Error accessing ECDSA blob.",
            Quote3Error::StoredKey => "Cached ECDSA key is invalid.",
            Quote3Error::PubkeyIdMismatch => "Cached ECDSA key does not match requested key.",
            Quote3Error::InvalidPceSigScheme => "PCE use the incorrect signature scheme.",
            Quote3Error::AttKeyBlobError => "There is a problem with the attestation key blob.",
            Quote3Error::UnsupportedAttKeyId => "Unsupported attestation key ID.",
            Quote3Error::UnsupportedLoadingPolicy => "Unsupported enclave loading policy",
            Quote3Error::InterfaceUnavailable => "Unable to load the QE enclave.",
            Quote3Error::PlatformLibUnavailable => {
                "Unable to find the platform library with the dependent APIs"
            }
            Quote3Error::AttKeyNotInitialized => {
                "he attestation key doesn't exist or has not been certified."
            }
            Quote3Error::AttKeyCertDataInvalid => {
                "The certification data retrieved from the platform library is invalid."
            }
            Quote3Error::NoPlatformCertData => {
                "The platform library doesn't have any platfrom cert data."
            }
            Quote3Error::OutOfEpc => "Not enough memory in the EPC to load the enclave.",
            Quote3Error::ErrorReport => "There was a problem verifying an SGX REPORT.",
            Quote3Error::EnclaveLost => {
                "Interfacing to the enclave failed due to a power transition."
            }
            Quote3Error::InvalidReport => "Error verifying the application enclave's report.",
            Quote3Error::EnclaveLoadError => "Unable to load the enclaves.",
            Quote3Error::UnableToGenerateQeReport => {
                "The QE was unable to generate its own report targeting the application enclave."
            }
            Quote3Error::KeyCertifcationError => {
                "Caused when the provider library returns an invalid TCB."
            }
            Quote3Error::NetworkError => "Network error when retrieving PCK certs.",
            Quote3Error::MessageError => "Message error when retrieving PCK certs.",
            Quote3Error::NoQuoteCollateralData => {
                "The platform does not have the quote verification collateral data available."
            }
            Quote3Error::QuoteCertificationDataUnsupported => "",
            Quote3Error::QuoteFormatUnsupported => "",
            Quote3Error::UnableToGenerateReport => "",
            Quote3Error::QeReportInvalidSignature => "",
            Quote3Error::QeReportUnsupportedFormat => "",
            Quote3Error::PckCertUnsupportedFormat => "",
            Quote3Error::PckCertChainError => "",
            Quote3Error::TcbInfoUnsupportedFormat => "",
            Quote3Error::TcbInfoMismatch => "",
            Quote3Error::QeIdentityUnsupportedFormat => "",
            Quote3Error::QeIdentityMismatch => "",
            Quote3Error::TcbOutOfDate => "",
            Quote3Error::TcbOutOfDateConfigurationNeeded => "",
            Quote3Error::EnclaveIdentityOutOfDate => "",
            Quote3Error::EnclaveReportIsvsvnOutOfDate => "",
            Quote3Error::QeIdentityOutOfDate => "",
            Quote3Error::TcbInfoExpired => "",
            Quote3Error::PckCertChainExpired => "",
            Quote3Error::CrlExpired => "",
            Quote3Error::SigningCertChainExpired => "",
            Quote3Error::EnclaveIdentityExpired => "",
            Quote3Error::PckRevoked => "",
            Quote3Error::TcbRevoked => "",
            Quote3Error::TcbConfigurationNeeded => "",
            Quote3Error::UnableToGetCollateral => "",
            Quote3Error::InvalidPrivilege => "No enough privilege to perform the operation.",
            Quote3Error::NoQveIdentityData => {
                "The platform does not have the QVE identity data available."
            }
            Quote3Error::CrlUnsupportedFormat => "",
            Quote3Error::QeIdentityChainError => "",
            Quote3Error::TcbInfoChainError => "",
            Quote3Error::QvlQveMismatch => {
                "QvE returned supplemental data version mismatched between QVL and QvE."
            }
            Quote3Error::TcbSwHardeningNeeded => "TCB up to date but SW Hardening needed.",
            Quote3Error::TcbConfigurationAndSwHardeningNeeded => {
                "TCB up to date but Configuration and SW Hardening needed."
            }
            Quote3Error::UnsupportedMode => "",
            Quote3Error::NoDevice => "",
            Quote3Error::ServiceUnavailable => "",
            Quote3Error::NetworkFailure => "",
            Quote3Error::ServiceTimeout => "",
            Quote3Error::ServiceBusy => "",
            Quote3Error::UnknownMessageResponse => "Unexpected error from the cache service.",
            Quote3Error::PersistentStorageError => {
                "Error storing the retrieved cached data in persistent memory."
            }
            Quote3Error::MessageParsingError => "Message parsing error.",
            Quote3Error::PlatformUnknown => "Platform was not found in the cache",
            Quote3Error::UnknownApiVersion => "The current PCS API version configured is unknown.",
            Quote3Error::CertsUnavailable => "Certificates are not available for this platform",
            Quote3Error::QveIdentityMismatch => {
                "QvE Identity is NOT match to Intel signed QvE identity."
            }
            Quote3Error::QveOutOfDate => "QvE ISVSVN is smaller then the ISVSVN threshold.",
            Quote3Error::PswNotAvailable => {
                "SGX PSW library cannot be loaded, could be due to file I/O error."
            }
            Quote3Error::CollateralVersionNotSupported => {
                "SGX quote verification collateral version not supported by QVL/QvE."
            }
            Quote3Error::TdxModuleMismatch => {
                "TDX SEAM module identity is NOT match to Intel signed TDX SEAM module"
            }
            Quote3Error::ErrorMax => "Indicate max error to allow better translation.",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            Quote3Error::Success => "Success.",
            Quote3Error::Unexpected => "Unexpected",
            Quote3Error::InvalidParameter => "InvalidParameter",
            Quote3Error::OutOfMemory => "OutOfMemory",
            Quote3Error::EcdsaIdMisMatch => "EcdsaIdMisMatch",
            Quote3Error::PathNameBufferOverflow => "PathNameBufferOverflow",
            Quote3Error::FileAccessError => "FileAccessError",
            Quote3Error::StoredKey => "StoredKey",
            Quote3Error::PubkeyIdMismatch => "PubkeyIdMismatch",
            Quote3Error::InvalidPceSigScheme => "InvalidPceSigScheme",
            Quote3Error::AttKeyBlobError => "AttKeyBlobError",
            Quote3Error::UnsupportedAttKeyId => "UnsupportedAttKeyId",
            Quote3Error::UnsupportedLoadingPolicy => "UnsupportedLoadingPolicy",
            Quote3Error::InterfaceUnavailable => "InterfaceUnavailable",
            Quote3Error::PlatformLibUnavailable => "PlatformLibUnavailable",
            Quote3Error::AttKeyNotInitialized => "AttKeyNotInitialized",
            Quote3Error::AttKeyCertDataInvalid => "AttKeyCertDataInvalid",
            Quote3Error::NoPlatformCertData => "NoPlatformCertData",
            Quote3Error::OutOfEpc => "OutOfEpc",
            Quote3Error::ErrorReport => "ErrorReport",
            Quote3Error::EnclaveLost => "EnclaveLost",
            Quote3Error::InvalidReport => "InvalidReport",
            Quote3Error::EnclaveLoadError => "EnclaveLoadError",
            Quote3Error::UnableToGenerateQeReport => "UnableToGenerateQeReport",
            Quote3Error::KeyCertifcationError => "KeyCertifcationError",
            Quote3Error::NetworkError => "NetworkError",
            Quote3Error::MessageError => "MessageError",
            Quote3Error::NoQuoteCollateralData => "NoQuoteCollateralData",
            Quote3Error::QuoteCertificationDataUnsupported => "QuoteCertificationDataUnsupported",
            Quote3Error::QuoteFormatUnsupported => "QuoteFormatUnsupported",
            Quote3Error::UnableToGenerateReport => "UnableToGenerateReport",
            Quote3Error::QeReportInvalidSignature => "QeReportInvalidSignature",
            Quote3Error::QeReportUnsupportedFormat => "QeReportUnsupportedFormat",
            Quote3Error::PckCertUnsupportedFormat => "PckCertUnsupportedFormat",
            Quote3Error::PckCertChainError => "PckCertChainError",
            Quote3Error::TcbInfoUnsupportedFormat => "TcbInfoUnsupportedFormat",
            Quote3Error::TcbInfoMismatch => "TcbInfoMismatch",
            Quote3Error::QeIdentityUnsupportedFormat => "QeIdentityUnsupportedFormat",
            Quote3Error::QeIdentityMismatch => "QeIdentityMismatch",
            Quote3Error::TcbOutOfDate => "TcbOutOfDate",
            Quote3Error::TcbOutOfDateConfigurationNeeded => "TcbOutOfDateConfigurationNeeded",
            Quote3Error::EnclaveIdentityOutOfDate => "EnclaveIdentityOutOfDate",
            Quote3Error::EnclaveReportIsvsvnOutOfDate => "EnclaveReportIsvsvnOutOfDate",
            Quote3Error::QeIdentityOutOfDate => "QeIdentityOutOfDate",
            Quote3Error::TcbInfoExpired => "TcbInfoExpired",
            Quote3Error::PckCertChainExpired => "PckCertChainExpired",
            Quote3Error::CrlExpired => "CrlExpired",
            Quote3Error::SigningCertChainExpired => "SigningCertChainExpired",
            Quote3Error::EnclaveIdentityExpired => "EnclaveIdentityExpired",
            Quote3Error::PckRevoked => "PckRevoked",
            Quote3Error::TcbRevoked => "TcbRevoked",
            Quote3Error::TcbConfigurationNeeded => "TcbConfigurationNeeded",
            Quote3Error::UnableToGetCollateral => "UnableToGetCollateral",
            Quote3Error::InvalidPrivilege => "InvalidPrivilege",
            Quote3Error::NoQveIdentityData => "NoQveIdentityData",
            Quote3Error::CrlUnsupportedFormat => "CrlUnsupportedFormat",
            Quote3Error::QeIdentityChainError => "QeIdentityChainError",
            Quote3Error::TcbInfoChainError => "TcbInfoChainError",
            Quote3Error::QvlQveMismatch => "QvlQveMismatch",
            Quote3Error::TcbSwHardeningNeeded => "TcbSwHardeningNeeded",
            Quote3Error::TcbConfigurationAndSwHardeningNeeded => {
                "TcbConfigurationAndSwHardeningNeeded"
            }
            Quote3Error::UnsupportedMode => "UnsupportedMode",
            Quote3Error::NoDevice => "NoDevice",
            Quote3Error::ServiceUnavailable => "ServiceUnavailable",
            Quote3Error::NetworkFailure => "NetworkFailure",
            Quote3Error::ServiceTimeout => "ServiceTimeout",
            Quote3Error::ServiceBusy => "ServiceBusy",
            Quote3Error::UnknownMessageResponse => "UnknownMessageResponse",
            Quote3Error::PersistentStorageError => "PersistentStorageError",
            Quote3Error::MessageParsingError => "MessageParsingError",
            Quote3Error::PlatformUnknown => "PlatformUnknown",
            Quote3Error::UnknownApiVersion => "UnknownApiVersion",
            Quote3Error::CertsUnavailable => "CertsUnavailable",
            Quote3Error::QveIdentityMismatch => "QveIdentityMismatch",
            Quote3Error::QveOutOfDate => "QveOutOfDate",
            Quote3Error::PswNotAvailable => "PswNotAvailable",
            Quote3Error::CollateralVersionNotSupported => "CollateralVersionNotSupported",
            Quote3Error::TdxModuleMismatch => "TdxModuleMismatch",
            Quote3Error::ErrorMax => "ErrorMax",
        }
    }
}

impl fmt::Display for Quote3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum QcnlError {
        Success                     = 0x0000_0000,
        Unexpected                  = 0x0000_B001,
        InvalidParameter            = 0x0000_B002,
        NetworkError                = 0x0000_B003,
        NetworkProxyFail            = 0x0000_B004,
        NetworkHostFail             = 0x0000_B005,
        NetworkNotConnected         = 0x0000_B006,
        NetworkHttp2Error           = 0x0000_B007,
        NetworkWriteError           = 0x0000_B008,
        NetworkTimeout              = 0x0000_B009,
        NetworkHttpsError           = 0x0000_B00A,
        NetworkUnknownOption        = 0x0000_B00B,
        NetworkInitError            = 0x0000_B00C,
        MsgError                    = 0x0000_B00D,
        OutOfMemory                 = 0x0000_B00E,
        StatusNoCacheData           = 0x0000_B00F,
        StatusPlatformUnknown       = 0x0000_B010,
        StatusUnexpected            = 0x0000_B011,
        StatusCertsUnavaliable      = 0x0000_B012,
        StatusServiceUnavaliable    = 0x0000_B013,
        InvalidConfig               = 0x0000_B030,
    }
}

impl QcnlError {
    pub fn __description(&self) -> &'static str {
        match *self {
            QcnlError::Success => "Success.",
            QcnlError::Unexpected => "Unexpected error.",
            QcnlError::InvalidParameter => "The parameter is incorrect.",
            QcnlError::NetworkError => "Network error.",
            QcnlError::NetworkProxyFail => "Network error : Couldn't resolve proxy.",
            QcnlError::NetworkHostFail => "Network error : Couldn't resolve host.",
            QcnlError::NetworkNotConnected => {
                "Network error : Failed to connect() to host or proxy."
            }
            QcnlError::NetworkHttp2Error => {
                "Network error : A problem was detected in the HTTP2 framing layer."
            }
            QcnlError::NetworkWriteError => {
                "Network error : an error was returned to libcurl from a write callback."
            }
            QcnlError::NetworkTimeout => "Network error : Operation timeout.",
            QcnlError::NetworkHttpsError => {
                "Network error : A problem occurred somewhere in the SSL/TLS handshake."
            }
            QcnlError::NetworkUnknownOption => {
                "Network error : An option passed to libcurl is not recognized/known."
            }
            QcnlError::NetworkInitError => "Failed to initialize CURL library.",
            QcnlError::MsgError => "HTTP message error.",
            QcnlError::OutOfMemory => "Out of memory error.",
            QcnlError::StatusNoCacheData => "No cache data.",
            QcnlError::StatusPlatformUnknown => "Platform unknown.",
            QcnlError::StatusUnexpected => "Unexpected cache error.",
            QcnlError::StatusCertsUnavaliable => "Certs not available.",
            QcnlError::StatusServiceUnavaliable => "Service is currently not available.",
            QcnlError::InvalidConfig => "Error in configuration file.",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            QcnlError::Success => "Success.",
            QcnlError::Unexpected => "Unexpected",
            QcnlError::InvalidParameter => "InvalidParameter",
            QcnlError::NetworkError => "NetworkError",
            QcnlError::NetworkProxyFail => "NetworkProxyFail",
            QcnlError::NetworkHostFail => "NetworkHostFail",
            QcnlError::NetworkNotConnected => "NetworkNotConnected",
            QcnlError::NetworkHttp2Error => "NetworkHttp2Error",
            QcnlError::NetworkWriteError => "NetworkWriteError",
            QcnlError::NetworkTimeout => "NetworkTimeout",
            QcnlError::NetworkHttpsError => "NetworkHttpsError",
            QcnlError::NetworkUnknownOption => "NetworkUnknownOption",
            QcnlError::NetworkInitError => "NetworkInitError",
            QcnlError::MsgError => "MsgError",
            QcnlError::OutOfMemory => "OutOfMemory",
            QcnlError::StatusNoCacheData => "StatusNoCacheData",
            QcnlError::StatusPlatformUnknown => "StatusPlatformUnknown",
            QcnlError::StatusUnexpected => "StatusUnexpected",
            QcnlError::StatusCertsUnavaliable => "StatusCertsUnavaliable",
            QcnlError::StatusServiceUnavaliable => "StatusServiceUnavaliable",
            QcnlError::InvalidConfig => "InvalidConfig",
        }
    }
}

impl fmt::Display for QcnlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub type SgxResult<T = ()> = result::Result<T, SgxStatus>;
pub type SgxPceResult<T = ()> = result::Result<T, PceError>;
pub type SgxQcnlResult<T = ()> = result::Result<T, QcnlError>;
pub type SgxQuote3Result<T = ()> = result::Result<T, Quote3Error>;

pub type OsError = i32;
pub type OsResult<T = ()> = result::Result<T, OsError>;

#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($e);
    };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            bail!($e);
        }
    };
}
