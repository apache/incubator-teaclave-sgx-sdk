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
    ($(
        if #[cfg($($meta:meta),*)] { $($it:item)* }
    ) else * else {
        $($it2:item)*
    }) => {
        cfg_if! {
            @__items
            () ;
            $( ( ($($meta),*) ($($it)*) ), )*
            ( () ($($it2)*) ),
        }
    };

    // match if/else chains lacking a final `else`
    (
        if #[cfg($($i_met:meta),*)] { $($i_it:item)* }
        $(
            else if #[cfg($($e_met:meta),*)] { $($e_it:item)* }
        )*
    ) => {
        cfg_if! {
            @__items
            () ;
            ( ($($i_met),*) ($($i_it)*) ),
            $( ( ($($e_met),*) ($($e_it)*) ), )*
            ( () () ),
        }
    };

    // Internal and recursive macro to emit all the items
    //
    // Collects all the negated cfgs in a list at the beginning and after the
    // semicolon is all the remaining items
    (@__items ($($not:meta,)*) ; ) => {};
    (@__items ($($not:meta,)*) ; ( ($($m:meta),*) ($($it:item)*) ),
     $($rest:tt)*) => {
        // Emit all items within one block, applying an approprate #[cfg]. The
        // #[cfg] will require all `$m` matchers specified and must also negate
        // all previous matchers.
        cfg_if! { @__apply cfg(all($($m,)* not(any($($not),*)))), $($it)* }

        // Recurse to emit all other items in `$rest`, and when we do so add all
        // our `$m` matchers to the list of `$not` matchers as future emissions
        // will have to negate everything we just matched as well.
        cfg_if! { @__items ($($not,)* $($m,)*) ; $($rest)* }
    };

    // Internal macro to Apply a cfg attribute to a list of items
    (@__apply $m:meta, $($it:item)*) => {
        $(#[$m] $it)*
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
