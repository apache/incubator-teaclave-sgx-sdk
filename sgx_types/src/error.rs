// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
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

use core::result;
//
// sgx_error.h
//
impl_enum! {

    #[repr(u32)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum sgx_status_t {
        SGX_SUCCESS                  = 0x00000000,

        SGX_ERROR_UNEXPECTED         = 0x00000001,      /* Unexpected error */
        SGX_ERROR_INVALID_PARAMETER  = 0x00000002,      /* The parameter is incorrect */
        SGX_ERROR_OUT_OF_MEMORY      = 0x00000003,      /* Not enough memory is available to complete this operation */
        SGX_ERROR_ENCLAVE_LOST       = 0x00000004,      /* Enclave lost after power transition or used in child process created by linux:fork() */
        SGX_ERROR_INVALID_STATE      = 0x00000005,      /* SGX API is invoked in incorrect order or state */

        SGX_ERROR_INVALID_FUNCTION   = 0x00001001,      /* The ecall/ocall index is invalid */
        SGX_ERROR_OUT_OF_TCS         = 0x00001003,      /* The enclave is out of TCS */
        SGX_ERROR_ENCLAVE_CRASHED    = 0x00001006,      /* The enclave is crashed */
        SGX_ERROR_ECALL_NOT_ALLOWED  = 0x00001007,      /* The ECALL is not allowed at this time, e.g. ecall is blocked by the dynamic entry table, or nested ecall is not allowed during initialization */
        SGX_ERROR_OCALL_NOT_ALLOWED  = 0x00001008,      /* The OCALL is not allowed at this time, e.g. ocall is not allowed during exception handling */
        SGX_ERROR_STACK_OVERRUN      = 0x00001009,      /* The enclave is running out of stack */

        SGX_ERROR_UNDEFINED_SYMBOL   = 0x00002000,      /* The enclave image has undefined symbol. */
        SGX_ERROR_INVALID_ENCLAVE    = 0x00002001,      /* The enclave image is not correct. */
        SGX_ERROR_INVALID_ENCLAVE_ID = 0x00002002,      /* The enclave id is invalid */
        SGX_ERROR_INVALID_SIGNATURE  = 0x00002003,      /* The signature is invalid */
        SGX_ERROR_NDEBUG_ENCLAVE     = 0x00002004,      /* The enclave is signed as product enclave, and can not be created as debuggable enclave. */
        SGX_ERROR_OUT_OF_EPC         = 0x00002005,      /* Not enough EPC is available to load the enclave */
        SGX_ERROR_NO_DEVICE          = 0x00002006,      /* Can't open SGX device */
        SGX_ERROR_MEMORY_MAP_CONFLICT= 0x00002007,      /* Page mapping failed in driver */
        SGX_ERROR_INVALID_METADATA   = 0x00002009,      /* The metadata is incorrect. */
        SGX_ERROR_DEVICE_BUSY        = 0x0000200c,      /* Device is busy, mostly EINIT failed. */
        SGX_ERROR_INVALID_VERSION    = 0x0000200d,      /* Metadata version is inconsistent between uRTS and sgx_sign or uRTS is incompatible with current platform. */
        SGX_ERROR_MODE_INCOMPATIBLE  = 0x0000200e,      /* The target enclave 32/64 bit mode or sim/hw mode is incompatible with the mode of current uRTS. */
        SGX_ERROR_ENCLAVE_FILE_ACCESS = 0x0000200f,     /* Can't open enclave file. */
        SGX_ERROR_INVALID_MISC        = 0x00002010,     /* The MiscSelct/MiscMask settings are not correct.*/

        SGX_ERROR_MAC_MISMATCH       = 0x00003001,      /* Indicates verification error for reports, sealed datas, etc */
        SGX_ERROR_INVALID_ATTRIBUTE  = 0x00003002,      /* The enclave is not authorized */
        SGX_ERROR_INVALID_CPUSVN     = 0x00003003,      /* The cpu svn is beyond platform's cpu svn value */
        SGX_ERROR_INVALID_ISVSVN     = 0x00003004,      /* The isv svn is greater than the enclave's isv svn */
        SGX_ERROR_INVALID_KEYNAME    = 0x00003005,      /* The key name is an unsupported value */

        SGX_ERROR_SERVICE_UNAVAILABLE       = 0x00004001,   /* Indicates aesm didn't response or the requested service is not supported */
        SGX_ERROR_SERVICE_TIMEOUT           = 0x00004002,   /* The request to aesm time out */
        SGX_ERROR_AE_INVALID_EPIDBLOB       = 0x00004003,   /* Indicates epid blob verification error */
        SGX_ERROR_SERVICE_INVALID_PRIVILEGE = 0x00004004,   /* Enclave has no privilege to get launch token */
        SGX_ERROR_EPID_MEMBER_REVOKED       = 0x00004005,   /* The EPID group membership is revoked. */
        SGX_ERROR_UPDATE_NEEDED             = 0x00004006,   /* SGX needs to be updated */
        SGX_ERROR_NETWORK_FAILURE           = 0x00004007,   /* Network connecting or proxy setting issue is encountered */
        SGX_ERROR_AE_SESSION_INVALID        = 0x00004008,   /* Session is invalid or ended by server */
        SGX_ERROR_BUSY                      = 0x0000400a,   /* The requested service is temporarily not availabe */
        SGX_ERROR_MC_NOT_FOUND              = 0x0000400c,   /* The Monotonic Counter doesn't exist or has been invalided */
        SGX_ERROR_MC_NO_ACCESS_RIGHT        = 0x0000400d,   /* Caller doesn't have the access right to specified VMC */
        SGX_ERROR_MC_USED_UP                = 0x0000400e,   /* Monotonic counters are used out */
        SGX_ERROR_MC_OVER_QUOTA             = 0x0000400f,   /* Monotonic counters exceeds quota limitation */
        SGX_ERROR_KDF_MISMATCH              = 0x00004011,   /* Key derivation function doesn't match during key exchange */
        SGX_ERROR_UNRECOGNIZED_PLATFORM     = 0x00004012,   /* EPID Provisioning failed due to platform not recognized by backend server*/

        /* SGX errors are only used in the file API when there is no appropriate EXXX (EINVAL, EIO etc.) error code */
        SGX_ERROR_FILE_BAD_STATUS               = 0x00007001,	/* The file is in bad status, run sgx_clearerr to try and fix it */
        SGX_ERROR_FILE_NO_KEY_ID                = 0x00007002,	/* The Key ID field is all zeros, can't re-generate the encryption key */
        SGX_ERROR_FILE_NAME_MISMATCH            = 0x00007003,	/* The current file name is different then the original file name (not allowed, substitution attack) */
        SGX_ERROR_FILE_NOT_SGX_FILE             = 0x00007004,   /* The file is not an SGX file */
        SGX_ERROR_FILE_CANT_OPEN_RECOVERY_FILE  = 0x00007005,	/* A recovery file can't be opened, so flush operation can't continue (only used when no EXXX is returned)  */
        SGX_ERROR_FILE_CANT_WRITE_RECOVERY_FILE = 0x00007006,   /* A recovery file can't be written, so flush operation can't continue (only used when no EXXX is returned)  */
        SGX_ERROR_FILE_RECOVERY_NEEDED          = 0x00007007,	/* When openeing the file, recovery is needed, but the recovery process failed */
        SGX_ERROR_FILE_FLUSH_FAILED             = 0x00007008,	/* fflush operation (to disk) failed (only used when no EXXX is returned) */
        SGX_ERROR_FILE_CLOSE_FAILED             = 0x00007009,	/* fclose operation (to disk) failed (only used when no EXXX is returned) */
    }
}

pub type sys_error_t = ::int32_t;
pub type SgxResult<T> = result::Result<T, sgx_status_t>;
pub type SgxError = result::Result<(), sgx_status_t>;
pub type SysResult<T> = result::Result<T, sys_error_t>;
pub type SysError = result::Result<(), sys_error_t>;

pub const EPERM: ::int32_t              = 1;
pub const ENOENT: ::int32_t             = 2;
pub const ESRCH: ::int32_t              = 3;
pub const EINTR: ::int32_t              = 4;
pub const EIO: ::int32_t                = 5;
pub const ENXIO: ::int32_t              = 6;
pub const E2BIG: ::int32_t              = 7;
pub const ENOEXEC: ::int32_t            = 8;
pub const EBADF: ::int32_t              = 9;
pub const ECHILD: ::int32_t             = 10;
pub const EAGAIN: ::int32_t             = 11;
pub const ENOMEM: ::int32_t             = 12;
pub const EACCES: ::int32_t             = 13;
pub const EFAULT: ::int32_t             = 14;
pub const ENOTBLK: ::int32_t            = 15;
pub const EBUSY: ::int32_t              = 16;
pub const EEXIST: ::int32_t             = 17;
pub const EXDEV: ::int32_t              = 18;
pub const ENODEV: ::int32_t             = 19;
pub const ENOTDIR: ::int32_t            = 20;
pub const EISDIR: ::int32_t             = 21;
pub const EINVAL: ::int32_t             = 22;
pub const ENFILE: ::int32_t             = 23;
pub const EMFILE: ::int32_t             = 24;
pub const ENOTTY: ::int32_t             = 25;
pub const ETXTBSY: ::int32_t            = 26;
pub const EFBIG: ::int32_t              = 27;
pub const ENOSPC: ::int32_t             = 28;
pub const ESPIPE: ::int32_t             = 29;
pub const EROFS: ::int32_t              = 30;
pub const EMLINK: ::int32_t             = 31;
pub const EPIPE: ::int32_t              = 32;
pub const EDOM: ::int32_t               = 33;
pub const ERANGE: ::int32_t             = 34;
pub const EDEADLK: ::int32_t            = 35;
pub const ENAMETOOLONG: ::int32_t       = 36;
pub const ENOLCK: ::int32_t             = 37;
pub const ENOSYS: ::int32_t             = 38;
pub const ENOTEMPTY: ::int32_t          = 39;
pub const ELOOP: ::int32_t              = 40;
pub const EWOULDBLOCK: ::int32_t        = EAGAIN;
pub const ENOMSG: ::int32_t             = 42;
pub const EIDRM: ::int32_t              = 43;
pub const ECHRNG: ::int32_t             = 44;
pub const EL2NSYNC: ::int32_t           = 45;
pub const EL3HLT: ::int32_t             = 46;
pub const EL3RST: ::int32_t             = 47;
pub const ELNRNG: ::int32_t             = 48;
pub const EUNATCH: ::int32_t            = 49;
pub const ENOCSI: ::int32_t             = 50;
pub const EL2HLT: ::int32_t             = 51;
pub const EBADE: ::int32_t              = 52;
pub const EBADR: ::int32_t              = 53;
pub const EXFULL: ::int32_t             = 54;
pub const ENOANO: ::int32_t             = 55;
pub const EBADRQC: ::int32_t            = 56;
pub const EBADSLT: ::int32_t            = 57;
pub const EDEADLOCK: ::int32_t          = EDEADLK;
pub const EBFONT: ::int32_t             = 59;
pub const ENOSTR: ::int32_t             = 60;
pub const ENODATA: ::int32_t            = 61;
pub const ETIME: ::int32_t              = 62;
pub const ENOSR: ::int32_t              = 63;
pub const ENONET: ::int32_t             = 64;
pub const ENOPKG: ::int32_t             = 65;
pub const EREMOTE: ::int32_t            = 66;
pub const ENOLINK: ::int32_t            = 67;
pub const EADV: ::int32_t               = 68;
pub const ESRMNT: ::int32_t             = 69;
pub const ECOMM: ::int32_t              = 70;
pub const EPROTO: ::int32_t             = 71;
pub const EMULTIHOP: ::int32_t          = 72;
pub const EDOTDOT: ::int32_t            = 73;
pub const EBADMSG: ::int32_t            = 74;
pub const EOVERFLOW: ::int32_t          = 75;
pub const ENOTUNIQ: ::int32_t           = 76;
pub const EBADFD: ::int32_t             = 77;
pub const EREMCHG: ::int32_t            = 78;
pub const ELIBACC: ::int32_t            = 79;
pub const ELIBBAD: ::int32_t            = 80;
pub const ELIBSCN: ::int32_t            = 81;
pub const ELIBMAX: ::int32_t            = 82;
pub const ELIBEXEC: ::int32_t           = 83;
pub const EILSEQ: ::int32_t             = 84;
pub const ERESTART: ::int32_t           = 85;
pub const ESTRPIPE: ::int32_t           = 86;
pub const EUSERS: ::int32_t             = 87;
pub const ENOTSOCK: ::int32_t           = 88;
pub const EDESTADDRREQ: ::int32_t       = 89;
pub const EMSGSIZE: ::int32_t           = 90;
pub const EPROTOTYPE: ::int32_t         = 91;
pub const ENOPROTOOPT: ::int32_t        = 92;
pub const EPROTONOSUPPORT: ::int32_t    = 93;
pub const ESOCKTNOSUPPORT: ::int32_t    = 94;
pub const EOPNOTSUPP: ::int32_t         = 95;
pub const EPFNOSUPPORT: ::int32_t       = 96;
pub const EAFNOSUPPORT: ::int32_t       = 97;
pub const EADDRINUSE: ::int32_t         = 98;
pub const EADDRNOTAVAIL: ::int32_t      = 99;
pub const ENETDOWN: ::int32_t           = 100;
pub const ENETUNREACH: ::int32_t        = 101;
pub const ENETRESET: ::int32_t          = 102;
pub const ECONNABORTED: ::int32_t       = 103;
pub const ECONNRESET: ::int32_t         = 104;
pub const ENOBUFS: ::int32_t            = 105;
pub const EISCONN: ::int32_t            = 106;
pub const ENOTCONN: ::int32_t           = 107;
pub const ESHUTDOWN: ::int32_t          = 108;
pub const ETOOMANYREFS: ::int32_t       = 109;
pub const ETIMEDOUT: ::int32_t          = 110;
pub const ECONNREFUSED: ::int32_t       = 111;
pub const EHOSTDOWN: ::int32_t          = 112;
pub const EHOSTUNREACH: ::int32_t       = 113;
pub const EALREADY: ::int32_t           = 114;
pub const EINPROGRESS: ::int32_t        = 115;
pub const ESTALE: ::int32_t             = 116;
pub const EUCLEAN: ::int32_t            = 117;
pub const ENOTNAM: ::int32_t            = 118;
pub const ENAVAIL: ::int32_t            = 119;
pub const EISNAM: ::int32_t             = 120;
pub const EREMOTEIO: ::int32_t          = 121;
pub const EDQUOT: ::int32_t             = 122;
pub const ENOMEDIUM: ::int32_t          = 123;
pub const EMEDIUMTYPE: ::int32_t        = 124;
pub const ECANCELED: ::int32_t          = 125;
pub const ENOKEY: ::int32_t             = 126;
pub const EKEYEXPIRED: ::int32_t        = 127;
pub const EKEYREVOKED: ::int32_t        = 128;
pub const EKEYREJECTED: ::int32_t       = 129;
pub const EOWNERDEAD: ::int32_t         = 130;
pub const ENOTRECOVERABLE: ::int32_t    = 131;
pub const ERFKILL: ::int32_t            = 132;
pub const EHWPOISON: ::int32_t          = 133;
pub const ENOTSUP: ::int32_t            = EOPNOTSUPP;
