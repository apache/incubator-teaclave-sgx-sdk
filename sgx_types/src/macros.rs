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

pub use crate::marker::ContiguousMemory;
pub use core::clone::Clone;
pub use core::default::Default;
pub use core::marker::Copy;
pub use core::mem::size_of;
pub use core::mem::transmute;
pub use core::ptr;

#[macro_export]
macro_rules! cfg_if {
    // match if/else chains with a final `else`
    (
        $(
            if #[cfg( $i_meta:meta )] { $( $i_tokens:tt )* }
        ) else+
        else { $( $e_tokens:tt )* }
    ) => {
        $crate::cfg_if! {
            @__items () ;
            $(
                (( $i_meta ) ( $( $i_tokens )* )) ,
            )+
            (() ( $( $e_tokens )* )) ,
        }
    };

    // match if/else chains lacking a final `else`
    (
        if #[cfg( $i_meta:meta )] { $( $i_tokens:tt )* }
        $(
            else if #[cfg( $e_meta:meta )] { $( $e_tokens:tt )* }
        )*
    ) => {
        $crate::cfg_if! {
            @__items () ;
            (( $i_meta ) ( $( $i_tokens )* )) ,
            $(
                (( $e_meta ) ( $( $e_tokens )* )) ,
            )*
        }
    };

    // Internal and recursive macro to emit all the items
    //
    // Collects all the previous cfgs in a list at the beginning, so they can be
    // negated. After the semicolon is all the remaining items.
    (@__items ( $( $_:meta , )* ) ; ) => {};
    (
        @__items ( $( $no:meta , )* ) ;
        (( $( $yes:meta )? ) ( $( $tokens:tt )* )) ,
        $( $rest:tt , )*
    ) => {
        // Emit all items within one block, applying an appropriate #[cfg]. The
        // #[cfg] will require all `$yes` matchers specified and must also negate
        // all previous matchers.
        #[cfg(all(
            $( $yes , )?
            not(any( $( $no ),* ))
        ))]
        $crate::cfg_if! { @__identity $( $tokens )* }

        // Recurse to emit all other items in `$rest`, and when we do so add all
        // our `$yes` matchers to the list of `$no` matchers as future emissions
        // will have to negate everything we just matched as well.
        $crate::cfg_if! {
            @__items ( $( $no , )* $( $yes , )? ) ;
            $( $rest , )*
        }
    };

    // Internal macro to make __apply work out right for different match types,
    // because of how macros match/expand stuff.
    (@__identity $( $tokens:tt )* ) => {
        $( $tokens )*
    };
}

#[macro_export]
macro_rules! __item {
    ($i:item) => {
        $i
    };
}

macro_rules! impl_copy_clone{
    ($($(#[$attr:meta])* pub struct $i:ident { $($field:tt)* })*) => ($(
        $crate::__item! {
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, PartialEq))]
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
        $crate::__item! {
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
        $crate::__item! {
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, PartialEq))]
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

macro_rules! impl_packed_struct {
    ($($(#[$attr:meta])* pub struct $i:ident { $(pub $name:ident: $field:ty,)* })*) => ($(
        $crate::__item! {
            #[cfg_attr(feature = "extra_traits", derive(Copy, Debug, Eq, PartialEq))]
            #[repr(C, packed)]
            $(#[$attr])*
            pub struct $i { $(pub $name: $field,)* }
        }
        #[cfg(not(feature = "extra_traits"))]
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

macro_rules! impl_packed_copy_clone {
    ($($(#[$attr:meta])* pub struct $i:ident { $($field:tt)* })*) => ($(
        $crate::__item! {
            #[cfg_attr(feature = "extra_traits", derive(Copy, Debug, Eq, PartialEq))]
            #[repr(C, packed)]
            $(#[$attr])*
            pub struct $i { $($field)* }
        }
        #[cfg(not(feature = "extra_traits"))]
        impl Copy for $i {}
        impl Clone for $i {
            fn clone(&self) -> $i { *self }
        }
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
    ($major:ident, $minor:ident) => {
        ($major as u64) << 32 | $minor as u64
    };
}

#[macro_export]
macro_rules! major_version_of_metadata {
    ($version:ident) => {
        ($version as u64) >> 32
    };
}

#[macro_export]
macro_rules! minor_version_of_metadata {
    ($version:ident) => {
        ($version as u64) & 0x0000_0000_FFFF_FFFF
    };
}

#[macro_export]
macro_rules! group_id {
    ($gid:expr) => {
        (GROUP_FLAG | $gid)
    };
}

#[macro_export]
macro_rules! is_group_id {
    ($gid:expr) => {
        (($gid & GROUP_FLAG) != 0)
    };
}
