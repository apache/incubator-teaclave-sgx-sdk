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

use crate::elf::slice::{self, AsSlice};
use crate::error;
use core::mem;
use core::str::{from_utf8, from_utf8_unchecked};
use sgx_types::marker::ContiguousMemory;

pub fn read<T: ContiguousMemory>(input: &[u8]) -> &T {
    assert!(mem::size_of::<T>() <= input.as_slice().len());
    let addr = input.as_slice().as_ptr() as usize;
    // Alignment is always a power of 2, so we can use bit ops instead of a mod here.
    assert!((addr & (mem::align_of::<T>() - 1)) == 0);

    unsafe { read_unsafe(input) }
}

pub fn read_array<T: ContiguousMemory>(input: &[u8]) -> &[T] {
    let t_size = mem::size_of::<T>();
    assert!(t_size > 0, "Can't read arrays of zero-sized types");
    assert!(input.as_slice().len() % t_size == 0);
    let addr = input.as_slice().as_ptr() as usize;
    assert!(addr & (mem::align_of::<T>() - 1) == 0);

    unsafe { read_array_unsafe(input) }
}

pub fn read_str(input: &[u8]) -> &str {
    from_utf8(read_str_bytes(input)).expect("Invalid UTF-8 string")
}

unsafe fn read_unsafe<T: Sized>(input: &[u8]) -> &T {
    &*(input.as_slice().as_ptr() as *const T)
}

unsafe fn read_array_unsafe<T: Sized>(input: &[u8]) -> &[T] {
    let ptr = input.as_slice().as_ptr() as *const T;
    slice::from_raw_parts(ptr, input.as_slice().len() / mem::size_of::<T>())
}

unsafe fn read_str_unsafe(input: &[u8]) -> &str {
    from_utf8_unchecked(read_str_bytes(input))
}

fn read_str_bytes(input: &[u8]) -> &[u8] {
    for (i, byte) in input.iter().enumerate() {
        if *byte == 0 {
            return &input[..i];
        }
    }
    error::abort();
}
