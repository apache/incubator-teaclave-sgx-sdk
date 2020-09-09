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
pub trait BytewiseEquality {}

impl_marker_for!(BytewiseEquality,
                 u8 i8 u16 i16 u32 i32 u64 i64 usize isize char bool);

impl<T: BytewiseEquality> BytewiseEquality for [T] {}

impl_marker_for_array! {BytewiseEquality,
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32 33 34 35 36 37 38 39
    40 41 42 43 44 45 46 47 48 49
    50 51 52 53 54 55 56 57 58 59
    60 61 62 63 64
}

/// Trait for demonstrating one structure locates in contiguous memory.
///
/// This is required for SGX related operations, e.g. crypto related
/// computations. Many of these APIs require the input data locates in
/// a contiguous area of memory inside the enclave. Developer needs to
/// implement this trait as a marker for the data structure he/she wants
/// to feed into SGX apis.
pub unsafe trait ContiguousMemory {}

impl_unsafe_marker_for!(ContiguousMemory,
                 u8 i8 u16 i16 u32 i32 u64 i64 usize isize char bool);

unsafe impl<T: ContiguousMemory> ContiguousMemory for [T] {}

impl_unsafe_marker_for_array! {ContiguousMemory,
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32 33 34 35 36 37 38 39
    40 41 42 43 44 45 46 47 48 49
    50 51 52 53 54 55 56 57 58 59
    60 61 62 63 64
}

/*
impl<T: ?Sized> !ContiguousMemory for *const T {}
impl<T: ?Sized> !ContiguousMemory for *mut T {}
impl<'a, T: 'a + ?Sized> !ContiguousMemory for &'a T {}
impl<'a, T: 'a + ?Sized> !ContiguousMemory for &'a mut T {}
*/
