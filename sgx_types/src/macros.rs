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

pub use core::clone::Clone;
pub use core::marker::Copy;
pub use core::default::Default;
pub use core::ptr;
pub use core::mem::transmute;
pub use core::mem::size_of;
pub use crate::marker::ContiguousMemory;

#[macro_export]
macro_rules! cfg_if {
    ($(
        if #[cfg($($meta:meta),*)] { $($it:item)* }
    ) else * else {
        $($it2:item)*
    }) => {
        __cfg_if_items! {
            () ;
            $( ( ($($meta),*) ($($it)*) ), )*
            ( () ($($it2)*) ),
        }
    }
}

#[macro_export]
macro_rules! __cfg_if_items {
    (($($not:meta,)*) ; ) => {};
    (($($not:meta,)*) ; ( ($($m:meta),*) ($($it:item)*) ), $($rest:tt)*) => {
        __cfg_if_apply! { cfg(all(not(any($($not),*)), $($m,)*)), $($it)* }
        __cfg_if_items! { ($($not,)* $($m,)*) ; $($rest)* }
    }
}

#[macro_export]
macro_rules! __cfg_if_apply {
    ($m:meta, $($it:item)*) => {
        $(#[$m] $it)*
    }
}

#[macro_export]
macro_rules! __item {
    ($i:item) => ($i)
}

macro_rules! impl_copy_clone{
    ($($(#[$attr:meta])* pub struct $i:ident { $($field:tt)* })*) => ($(
        __item! {
            #[repr(C)]
            $(#[$attr])*
            pub struct $i { $($field)* }
        }
        impl Copy for $i {}
        impl Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    )*)
}

#[macro_export]
macro_rules! s {
    ($($(#[$attr:meta])* pub struct $i:ident { $($field:tt)* })*) => ($(
        __item! {
            #[repr(C)]
            $(#[$attr])*
            pub struct $i { $($field)* }
        }
        impl Copy for $i {}
        impl Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    )*)
}

#[macro_export]
macro_rules! impl_struct {
    ($($(#[$attr:meta])* pub struct $i:ident { $(pub $name:ident: $field:ty,)* })*) => ($(
        __item! {
            #[repr(C)]
            $(#[$attr])*
            pub struct $i { $(pub $name: $field,)* }
        }
        impl Copy for $i {}
        impl Clone for $i {
            fn clone(&self) -> $i { *self }
        }
        impl Default for $i {
            fn default()->$i {
                $i{$($name: Default::default(),)*}
            }
        }
        unsafe impl $crate::marker::ContiguousMemory for $i {}
    )*)
}

macro_rules! impl_struct_default {
    ($($t:ty;)*) => {$(
        impl Default for $t {
            fn default() -> $t {
                unsafe{macros::transmute([0u8; macros::size_of::<$t>()])}
            }
        }
    )*}
}

macro_rules! impl_struct_clone {
    ($($t:ty;)*) => {$(
        impl Clone for $t {
            fn clone(&self) -> $t {
                unsafe{macros::ptr::read(self)}
            }
        }
    )*}
}

macro_rules! impl_struct_ContiguousMemory {

    ($($t:ty;)*) => {$(
        unsafe impl ContiguousMemory for $t {}
    )*}
}

#[macro_export]
macro_rules! impl_enum {
    (
        #[repr($repr:ident)]
        #[derive($($derive:meta),*)]
        pub enum $name:ident {
            $key:ident = $val:expr,
            $($keys:ident = $vals:expr,)*
        }
    ) => (
        #[repr($repr)]
        #[derive($($derive),*)]
        pub enum $name {
            $key = $val,
            $($keys = $vals,)*
        }

        impl Default for $name {
            fn default() -> $name {
                 $name::$key
            }
        }

        impl $name {
            pub fn from_repr(v: $repr) -> Option<Self> {
                match v {
                    $val => Some($name::$key),
                    $($vals => Some($name::$keys),)*
                    _ => None,
                }
            }

            pub fn from_key(self) -> $repr {
                match self {
                    $name::$key => $val,
                    $($name::$keys => $vals,)*
                }
            }
        }
    )
}

#[macro_export]
macro_rules! meta_data_make_version {
    ($major:ident, $minor:ident) => ( ($major as u64) << 32 | $minor as u64 )
}

#[macro_export]
macro_rules! major_version_of_metadata {
    ($version:ident) => ( ($version as u64) >> 32 )
}

#[macro_export]
macro_rules! minor_version_of_metadata {
    ($version:ident) => ( ($version as u64) & 0x0000_0000_FFFF_FFFF )
}

#[macro_export]
macro_rules! group_id {
    ($gid:expr) => ( (GROUP_FLAG | $gid) )
}

#[macro_export]
macro_rules! is_group_id {
    ($gid:expr) => ( (($gid & GROUP_FLAG) != 0) )
}
