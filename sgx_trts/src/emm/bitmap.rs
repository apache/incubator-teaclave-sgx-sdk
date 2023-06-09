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
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::clone::Clone;
use sgx_types::error::SgxResult;
use sgx_types::error::SgxStatus;

// box 能否 #[repr(C)]
#[derive(Clone)]
pub struct BitArray<A: Allocator + Clone> {
    pub bits: usize,
    pub bytes: usize,
    pub data: Box<[u8], A>, // temporariy use ResAlloc
    alloc: A,
}

impl<A: Allocator + Clone> BitArray<A> {
    /// Init BitArray in Reserve memory with all zeros.
    pub fn new_in(bits: usize, alloc: A) -> SgxResult<Self> {
        let bytes = (bits + 7) / 8;

        // FIXME: return error if out of memory
        let data: Box<[u8], A> = vec::from_elem_in(0_u8, bytes, alloc.clone()).into_boxed_slice();
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
        (self.data.get(byte_index).unwrap() & bit_mask) != 0
    }

    // Set the value of the bit at a given index.
    pub fn set(&mut self, index: usize, value: bool) {
        let byte_index = index / 8;
        let bit_index = index % 8;
        let bit_mask = 1 << bit_index;

        let data = self.data.as_mut();
        if value {
            data[byte_index] |= bit_mask;
        } else {
            data[byte_index] &= !bit_mask;
        }
    }

    // return chunk range with all true, Vec<[start, end)>
    pub fn true_range(&self) -> Vec<(usize, usize), A> {
        let mut true_range: Vec<(usize, usize), A> = Vec::new_in(self.alloc.clone());

        let start: usize = 0;
        let end: usize = self.bits;

        // TODO: optimized with [u8] slice
        while start < end {
            let mut block_start = start;
            while block_start < end {
                if self.get(block_start) {
                    break;
                } else {
                    block_start += 1;
                }
            }

            if block_start == end {
                break;
            }

            let mut block_end = block_start + 1;
            while block_end < end {
                if self.get(block_end) {
                    block_end += 1;
                } else {
                    break;
                }
            }
            true_range.push((start,end));
        }

        return true_range;  
    }

    /// Set the value of the bit at a given index.
    /// The range includes [0, index).
    pub fn set_until(&mut self, index: usize, value: bool) {
        todo!()
    }

    /// Set the value of the bit at a given index.
    /// The range includes [0, index).
    pub fn set_full(&mut self) {
        self.data.fill(0xFF);
    }

    /// Clear all the bits
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    // split current bit array into left and right bit array
    // return right bit array
    pub fn split(&mut self, pos: usize) -> SgxResult<BitArray<A>> {
        ensure!(pos > 0 && pos < self.bits, SgxStatus::InvalidParameter);

        let byte_index = pos / 8;
        let bit_index = pos % 8;

        // let l_bits = (byte_index << 3) + bit_index;
        let l_bits = pos;
        let l_bytes = (l_bits + 7) / 8;

        let r_bits = self.bits - l_bits;
        let r_bytes = (r_bits + 7) / 8;

        let mut r_array = Self::new_in(r_bits, self.alloc.clone())?;

        for (idx, item) in r_array.data[..(r_bytes - 1)].iter_mut().enumerate() {
            // current byte index in previous bit_array
            let curr_idx = idx + byte_index;
            let low_bits = self.data[curr_idx] >> bit_index;
            let high_bits = self.data[curr_idx + 1] << (8 - bit_index);
            *item = high_bits | low_bits;
        }
        r_array.data[r_bytes - 1] = self.data[self.bytes - 1] >> bit_index;

        self.bits = l_bits;
        self.bytes = l_bytes;

        return Ok(r_array);
    }
}



// FIXME: add more unit test