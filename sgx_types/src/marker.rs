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

use core::cell::{Cell, RefCell, UnsafeCell};
use core::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
    AtomicU64, AtomicU8, AtomicUsize,
};

macro_rules! impl_marker_for {
    ($traitname:ident, $($ty:ty)*) => {
        $(
            impl $traitname for $ty { }
        )*
    }
}

macro_rules! impl_marker_for_array {
    ($traitname:ident, $($N:expr)+) => {
        $(
            impl<T: $traitname> $traitname for [T; $N] { }
        )+
    }
}

macro_rules! impl_unsafe_marker_for {
    ($traitname:ident, $($ty:ty)*) => {
        $(
            unsafe impl $traitname for $ty { }
        )*
    }
}

macro_rules! impl_unsafe_marker_for_array {
    ($traitname:ident, $($N:expr)+) => {
        $(
            unsafe impl<T: $traitname> $traitname for [T; $N] { }
        )+
    }
}

/// Trait implemented for types that can be compared for equality using their bytewise representation
/// A type can implement BytewiseEquality if all of its components implement BytewiseEquality.
/// # Safety
pub unsafe trait BytewiseEquality {}

impl_unsafe_marker_for!(BytewiseEquality,
    u8 i8 u16 i16 u32 i32 u64 i64 usize isize char bool);

unsafe impl<T: BytewiseEquality> BytewiseEquality for [T] {}

unsafe impl<T: BytewiseEquality, const N: usize> BytewiseEquality for [T; N] {}

/// Trait for demonstrating one structure locates in contiguous memory.
///
/// This is required for SGX related operations, e.g. crypto related
/// computations. Many of these APIs require the input data locates in
/// a contiguous area of memory inside the enclave. Developer needs to
/// implement this trait as a marker for the data structure he/she wants
/// to feed into SGX apis.
/// # Safety
pub unsafe trait ContiguousMemory {}

impl_unsafe_marker_for!(ContiguousMemory,
    u8 i8 u16 i16 u32 i32 u64 i64 usize isize char bool);

impl_unsafe_marker_for!(ContiguousMemory, f32 f64);

impl_unsafe_marker_for!(ContiguousMemory,
    AtomicI16 AtomicI32 AtomicI64 AtomicI8 AtomicIsize AtomicU16 AtomicU32 AtomicU64 AtomicU8 AtomicUsize AtomicBool);

unsafe impl<T: ContiguousMemory> ContiguousMemory for [T] {}
unsafe impl<T: ContiguousMemory, const N: usize> ContiguousMemory for [T; N] {}

unsafe impl<T: ContiguousMemory> ContiguousMemory for Option<T> {}
unsafe impl<T: ContiguousMemory, E: ContiguousMemory> ContiguousMemory for Result<T, E> {}

unsafe impl<T: ContiguousMemory> ContiguousMemory for Cell<T> {}
unsafe impl<T: ContiguousMemory> ContiguousMemory for RefCell<T> {}
unsafe impl<T: ContiguousMemory> ContiguousMemory for UnsafeCell<T> {}

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => (tuple! { $($other,)* })
}

macro_rules! tuple {
    () => ();
    ( $($name:ident,)+ ) => (
        unsafe impl<$($name: ContiguousMemory),+> ContiguousMemory for ($($name,)+) {}
        peel! { $($name,)+ }
    )
}

tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }
