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

#![no_std]
#![cfg_attr(target_vendor = "teaclave", feature(rustc_private))]

extern crate sgx_types;

pub use self::bindings::*;

mod bindings {
    pub use sgx_types::error::errno::*;
    pub use sgx_types::types::time_t;
    pub use sgx_types::types::{
        c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint,
        c_ulong, c_ulonglong, c_ushort, c_void, intmax_t, uintmax_t,
    };
    pub use sgx_types::types::{
        int16_t, int32_t, int64_t, int8_t, uint16_t, uint32_t, uint64_t, uint8_t,
    };
    pub use sgx_types::types::{intptr_t, ptrdiff_t, size_t, ssize_t, uintptr_t};

    extern "C" {
        pub fn sgx_heap_init(
            heap_base: *const c_void,
            heap_size: size_t,
            heap_min_size: size_t,
            is_edmm_supported: c_int,
        ) -> c_uint;
        pub fn sgx_init_string_lib(cpu_features: uint64_t) -> c_int;
    }

    extern "C" {
        pub fn calloc(nobj: size_t, size: size_t) -> *mut c_void;
        pub fn malloc(size: size_t) -> *mut c_void;
        pub fn realloc(p: *mut c_void, size: size_t) -> *mut c_void;
        pub fn free(p: *mut c_void);
        pub fn memalign(align: size_t, size: size_t) -> *mut c_void;
        pub fn malloc_usable_size(ptr: *const c_void) -> size_t;
        pub fn memchr(cx: *const c_void, c: c_int, n: size_t) -> *mut c_void;
        pub fn memcmp(cx: *const c_void, ct: *const c_void, n: size_t) -> c_int;
        pub fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
        pub fn memcpy_verw(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
        pub fn memcpy_s(
            dest: *mut c_void,
            sizeinbytes: size_t,
            src: *const c_void,
            count: size_t,
        ) -> c_int;
        pub fn memcpy_verw_s(
            dest: *mut c_void,
            sizeinbytes: size_t,
            src: *const c_void,
            count: size_t,
        ) -> c_int;
        pub fn memmove(dest: *mut c_void, src: *const c_void, count: size_t) -> *mut c_void;
        pub fn memmove_verw(dest: *mut c_void, src: *const c_void, count: size_t) -> *mut c_void;
        pub fn memmove_s(
            dest: *mut c_void,
            sizeinbytes: size_t,
            src: *const c_void,
            count: size_t,
        ) -> c_int;
        pub fn memmove_verw_s(
            dest: *mut c_void,
            sizeinbytes: size_t,
            src: *const c_void,
            count: size_t,
        ) -> c_int;
        pub fn memset(dest: *mut c_void, c: c_int, n: size_t) -> *mut c_void;
        pub fn memset_verw(dest: *mut c_void, c: c_int, n: size_t) -> *mut c_void;
        pub fn memset_s(s: *mut c_void, smax: size_t, c: c_int, n: size_t) -> c_int;
        pub fn memset_verw_s(s: *mut c_void, smax: size_t, c: c_int, n: size_t) -> c_int;
        pub fn consttime_memequal(b1: *const c_void, b2: *const c_void, len: size_t) -> c_int;

        pub fn strchr(cs: *const c_char, c: c_int) -> *mut c_char;
        pub fn strcmp(cs: *const c_char, ct: *const c_char) -> c_int;
        pub fn strncmp(cs: *const c_char, ct: *const c_char, n: size_t) -> c_int;
        pub fn strcoll(cs: *const c_char, ct: *const c_char) -> c_int;
        pub fn strcspn(cs: *const c_char, ct: *const c_char) -> size_t;
        pub fn strlen(s: *const c_char) -> size_t;
        pub fn strncat(s: *mut c_char, ct: *const c_char, n: size_t) -> *mut c_char;
        pub fn strncpy(dst: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char;
        pub fn strpbrk(cs: *const c_char, ct: *const c_char) -> *mut c_char;
        pub fn strrchr(cs: *const c_char, c: c_int) -> *mut c_char;
        pub fn strspn(cs: *const c_char, ct: *const c_char) -> size_t;
        pub fn strstr(cs: *const c_char, ct: *const c_char) -> *mut c_char;
        pub fn strtok(s: *mut c_char, t: *const c_char) -> *mut c_char;
        pub fn strxfrm(s: *mut c_char, ct: *const c_char, n: size_t) -> size_t;
        pub fn strlcpy(dst: *mut c_char, src: *const c_char, n: size_t) -> size_t;
        pub fn strtol(s: *const c_char, endp: *mut *mut c_char, base: c_int) -> c_long;
        pub fn strtod(s: *const c_char, endp: *mut *mut c_char) -> c_double;
        pub fn strtoul(s: *const c_char, endp: *mut *mut c_char, base: c_int) -> c_ulong;

        pub fn strcat_s(dest: *mut c_char, dest_buffer_size: size_t, src: *const c_char) -> c_int;
        pub fn strncat_s(
            dest: *mut c_char,
            dest_buffer_size: size_t,
            src: *const c_char,
            count: size_t,
        ) -> c_int;
        pub fn strcpy_s(dest: *mut c_char, dest_buffer_size: size_t, src: *const c_char) -> c_int;
        pub fn strncpy_s(
            dest: *mut c_char,
            dest_buffer_size: size_t,
            src: *const c_char,
            count: size_t,
        ) -> c_int;
        pub fn strtok_s(
            string: *mut c_char,
            control: *const c_char,
            context: *mut *mut c_char,
        ) -> *mut c_char;
        pub fn _itoa_s(
            value: c_int,
            buffer: *mut c_char,
            dest_buffer_size: size_t,
            radix: c_int,
        ) -> c_int;
        pub fn _ltoa_s(
            value: c_long,
            buffer: *mut c_char,
            dest_buffer_size: size_t,
            radix: c_int,
        ) -> c_int;
        pub fn _ultoa_s(
            value: c_ulong,
            buffer: *mut c_char,
            dest_buffer_size: size_t,
            radix: c_int,
        ) -> c_int;
        pub fn _i64toa_s(
            value: c_longlong,
            buffer: *mut c_char,
            dest_buffer_size: size_t,
            radix: c_int,
        ) -> c_int;
        pub fn _ui64toa_s(
            value: c_ulonglong,
            buffer: *mut c_char,
            dest_buffer_size: size_t,
            radix: c_int,
        ) -> c_int;

        pub fn atoi(s: *const c_char) -> c_int;
        pub fn atol(s: *const c_char) -> c_long;

        #[cfg_attr(target_os = "linux", link_name = "__errno_location")]
        pub fn errno_location() -> *mut c_int;
        pub fn strerror(errnum: c_int) -> *const c_char;
        pub fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: size_t) -> c_int;
    }
}
