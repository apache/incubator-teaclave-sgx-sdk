
// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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
pub trait BytewiseEquality { }


impl_marker_for!(BytewiseEquality,
                 u8 i8 u16 i16 u32 i32 u64 i64 usize isize char bool);

impl<T: BytewiseEquality> BytewiseEquality for [T] { }

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
pub unsafe trait ContiguousMemory { }

impl_unsafe_marker_for!(ContiguousMemory,
                 u8 i8 u16 i16 u32 i32 u64 i64 usize isize char bool);

unsafe impl<T: ContiguousMemory> ContiguousMemory for [T] { }


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
impl<T: ?Sized> !ContiguousMemory for * const T {}
impl<T: ?Sized> !ContiguousMemory for * mut T {}
impl<'a, T: 'a + ?Sized> !ContiguousMemory for &'a T {}
impl<'a, T: 'a + ?Sized> !ContiguousMemory for &'a mut T {}
*/
