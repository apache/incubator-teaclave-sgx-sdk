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

//! Platform-specific types, as defined by C.
//!
//! Code that interacts via FFI will almost certainly be using the
//! base types provided by C, which aren't nearly as nicely defined
//! as Rust's primitive types. This module provides types which will
//! match those defined by C, so that code that interacts with C will
//! refer to the correct types.

use core::num::*;

macro_rules! type_alias_no_nz {
    {
      $Alias:ident = $Real:ty;
      $( $Cfg:tt )*
    } => {
        $( $Cfg )*
        pub type $Alias = $Real;
    }
}

macro_rules! type_alias {
    {
      $Alias:ident = $Real:ty, $NZAlias:ident = $NZReal:ty;
      $( $Cfg:tt )*
    } => {
        type_alias_no_nz! { $Alias = $Real; $( $Cfg )* }

        $( $Cfg )*
        pub type $NZAlias = $NZReal;
    }
}

type_alias! { c_char = i8, NonZero_c_char = NonZeroI8; }
type_alias! { c_schar = i8, NonZero_c_schar = NonZeroI8; }
type_alias! { c_uchar = u8, NonZero_c_uchar = NonZeroU8; }
type_alias! { c_short = i16, NonZero_c_short = NonZeroI16; }
type_alias! { c_ushort = u16, NonZero_c_ushort = NonZeroU16; }
type_alias! { c_int = i32, NonZero_c_int = NonZeroI32; }
type_alias! { c_uint = u32, NonZero_c_uint = NonZeroU32; }
type_alias! { c_long = i32, NonZero_c_long = NonZeroI32;
#[cfg(target_pointer_width = "32")] }
type_alias! { c_ulong = u32, NonZero_c_ulong = NonZeroU32;
#[cfg(target_pointer_width = "32")] }
type_alias! { c_long = i64, NonZero_c_long = NonZeroI64;
#[cfg(target_pointer_width = "64")] }
type_alias! { c_ulong = u64, NonZero_c_ulong = NonZeroU64;
#[cfg(target_pointer_width = "64")] }
type_alias! { c_longlong = i64, NonZero_c_longlong = NonZeroI64; }
type_alias! { c_ulonglong = u64, NonZero_c_ulonglong = NonZeroU64; }
type_alias_no_nz! { c_float = f32; }
type_alias_no_nz! { c_double = f64; }

#[doc(no_inline)]
pub use core::ffi::c_void;

/// Equivalent to C's `size_t` type, from `stddef.h` (or `cstddef` for C++).
///
/// This type is currently always [`usize`], however in the future there may be
/// platforms where this is not the case.
pub type c_size_t = usize;

/// Equivalent to C's `ptrdiff_t` type, from `stddef.h` (or `cstddef` for C++).
///
/// This type is currently always [`isize`], however in the future there may be
/// platforms where this is not the case.
pub type c_ptrdiff_t = isize;

/// Equivalent to C's `ssize_t` (on POSIX) or `SSIZE_T` (on Windows) type.
///
/// This type is currently always [`isize`], however in the future there may be
/// platforms where this is not the case.
pub type c_ssize_t = isize;
