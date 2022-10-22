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

//! Compatibility module for C platform-specific types. Use [`core::ffi`] instead.

macro_rules! alias_core_ffi {
    ($($t:ident)*) => {$(
        // Make this type alias appear cfg-dependent so that Clippy does not suggest
        // replacing expressions like `0 as c_char` with `0_i8`/`0_u8`. This #[cfg(all())] can be
        // removed after the false positive in https://github.com/rust-lang/rust-clippy/issues/8093
        // is fixed.
        #[cfg(all())]
        pub type $t = core::ffi::$t;
    )*}
}

alias_core_ffi! {
    c_char c_schar c_uchar
    c_short c_ushort
    c_int c_uint
    c_long c_ulong
    c_longlong c_ulonglong
    c_float
    c_double
    c_void
}
