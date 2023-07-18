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
use alloc::boxed::Box;
use alloc::vec;
use core::alloc::Allocator;
use core::alloc::Layout;
use core::clone::Clone;
use core::ptr::NonNull;
use sgx_types::error::SgxResult;
use sgx_types::error::SgxStatus;

use super::alloc::ResAlloc;
use super::alloc::StaticAlloc;
use super::interior::Alloc;

#[repr(C)]
#[derive(Clone)]
pub struct BitArray {
    bits: usize,
    bytes: usize,
    data: *mut u8,
    alloc: Alloc,
}

impl BitArray {
    /// Init BitArray in Reserve memory with all zeros.
    pub fn new(bits: usize, alloc: Alloc) -> SgxResult<Self> {
        let bytes = (bits + 7) / 8;

        // FIXME: return error if out of memory
        let data = match alloc {
            Alloc::Reserve => {
                let data = vec::from_elem_in(0_u8, bytes, ResAlloc).into_boxed_slice();
                Box::into_raw(data) as *mut u8
            }
            Alloc::Static => {
                let data = vec::from_elem_in(0_u8, bytes, StaticAlloc).into_boxed_slice();
                Box::into_raw(data) as *mut u8
            }
        };

        Ok(Self {
            bits,
            bytes,
            data,
            alloc,
        })
    }

    // Get the value of the bit at a given index.
    // todo: return SgxResult
    pub fn get(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = index % 8;
        let bit_mask = 1 << bit_index;
        let data = unsafe { core::slice::from_raw_parts_mut(self.data, self.bytes) };
        (data.get(byte_index).unwrap() & bit_mask) != 0
    }

    // check whether each bit set true
    pub fn all_true(&self) -> bool {
        for pos in 0..self.bits {
            if !self.get(pos) {
                return false;
            }
        }
        true
    }

    // Set the value of the bit at a given index.
    pub fn set(&mut self, index: usize, value: bool) {
        let byte_index = index / 8;
        let bit_index = index % 8;
        let bit_mask = 1 << bit_index;

        let data = unsafe { core::slice::from_raw_parts_mut(self.data, self.bytes) };

        if value {
            data[byte_index] |= bit_mask;
        } else {
            data[byte_index] &= !bit_mask;
        }
    }

    /// Set the value of the bit at a given index.
    /// The range includes [0, index).
    pub fn set_until(&mut self, index: usize, value: bool) {
        todo!()
    }

    /// Set the value of the bit at a given index.
    /// The range includes [0, index).
    pub fn set_full(&mut self) {
        let data = unsafe { core::slice::from_raw_parts_mut(self.data, self.bytes) };
        data.fill(0xFF);
    }

    /// Clear all the bits
    pub fn clear(&mut self) {
        let data = unsafe { core::slice::from_raw_parts_mut(self.data, self.bytes) };
        data.fill(0);
    }

    // split current bit array into left and right bit array
    // return right bit array
    // TODO: more check
    pub fn split(&mut self, pos: usize) -> SgxResult<BitArray> {
        ensure!(pos > 0 && pos < self.bits, SgxStatus::InvalidParameter);

        let byte_index = pos / 8;
        let bit_index = pos % 8;

        // let l_bits = (byte_index << 3) + bit_index;
        let l_bits = pos;
        let l_bytes = (l_bits + 7) / 8;

        let r_bits = self.bits - l_bits;
        let r_bytes = (r_bits + 7) / 8;

        let r_array = Self::new(r_bits, self.alloc.clone())?;

        let r_data = unsafe { core::slice::from_raw_parts_mut(r_array.data, r_array.bytes) };

        let l_data = unsafe { core::slice::from_raw_parts_mut(self.data, self.bytes) };

        for (idx, item) in r_data[..(r_bytes - 1)].iter_mut().enumerate() {
            // current byte index in previous bit_array
            let curr_idx = idx + byte_index;
            let low_bits = l_data[curr_idx] >> bit_index;
            let high_bits = l_data[curr_idx + 1] << (8 - bit_index);
            *item = high_bits | low_bits;
        }
        r_data[r_bytes - 1] = l_data[self.bytes - 1] >> bit_index;

        self.bits = l_bits;
        self.bytes = l_bytes;

        return Ok(r_array);
    }
}

impl Drop for BitArray {
    fn drop(&mut self) {
        match self.alloc {
            Alloc::Reserve => {
                // for interior allocator, layout is redundant
                let fake_layout: Layout = Layout::new::<u8>();
                unsafe {
                    let data_ptr = NonNull::new_unchecked(self.data);
                    ResAlloc.deallocate(data_ptr, fake_layout);
                }
            }
            Alloc::Static => {
                // for interior allocator, layout is redundant
                // If the bitmap is splitted, the size of
                // allocated layout is not recorded in bitmap struct
                let fake_layout: Layout = Layout::new::<u8>();
                unsafe {
                    let data_ptr = NonNull::new_unchecked(self.data);
                    StaticAlloc.deallocate(data_ptr, fake_layout);
                }
            }
        }
    }
}

// FIXME: add more unit test
