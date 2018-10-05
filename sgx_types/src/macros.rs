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

pub use core::clone::Clone;
pub use core::marker::Copy;
pub use core::default::Default;
pub use core::ptr;
pub use core::mem::transmute;
pub use marker::ContiguousMemory;

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
    ($($t:ty, $size:expr;)*) => {$(
        impl Default for $t {
            fn default() -> $t {
                unsafe{::macros::transmute([0u8; $size])}
            }
        }
    )*}
}

macro_rules! impl_struct_clone {
    ($($t:ty;)*) => {$(
        impl Clone for $t {
            fn clone(&self) -> $t {
                unsafe{::macros::ptr::read(self)}
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
    ($gid:expr) => ( (!!($gid & GROUP_FLAG)) )
}
