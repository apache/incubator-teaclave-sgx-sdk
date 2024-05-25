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
use sgx_tlibc_sys::EACCES;
use sgx_types::error::OsResult;

use crate::emm::alloc::EmmAllocator;
use crate::emm::alloc::RsrvAlloc;
use crate::emm::alloc::StaticAlloc;

use super::alloc::AllocType;

const BYTE_SIZE: usize = 8;
macro_rules! bytes_num {
    ($num:expr) => {
        ($num + BYTE_SIZE - 1) / BYTE_SIZE
    };
}

#[derive(Debug)]
pub struct BitArray {
    bits: usize,
    bytes: usize,
    data: Box<[u8], &'static dyn EmmAllocator>,
}

impl BitArray {
    /// Init BitArray with all zero bits
    pub fn new(bits: usize, alloc: AllocType) -> OsResult<Self> {
        let bytes = bytes_num!(bits);

        // FIXME: return error if OOM
        let data = vec::from_elem_in(0_u8, bytes, alloc.alloctor()).into_boxed_slice();

        Ok(Self { bits, bytes, data })
    }

    /// Get the value of the bit at a given index
    pub fn get(&self, index: usize) -> OsResult<bool> {
        if index >= self.bits {
            return Err(EACCES);
        }

        let byte_index = index / BYTE_SIZE;
        let bit_index = index % BYTE_SIZE;
        let bit_mask = 1 << bit_index;
        Ok((self.data.get(byte_index).unwrap() & bit_mask) != 0)
    }

    /// Check whether all bits are set true
    pub fn all_true(&self) -> bool {
        for pos in 0..self.bits {
            if !self.get(pos).unwrap() {
                return false;
            }
        }
        true
    }

    /// Set the value of the bit at the specified index
    pub fn set(&mut self, index: usize, value: bool) -> OsResult {
        if index >= self.bits {
            return Err(EACCES);
        }
        let byte_index = index / BYTE_SIZE;
        let bit_index = index % BYTE_SIZE;
        let bit_mask = 1 << bit_index;

        if value {
            self.data[byte_index] |= bit_mask;
        } else {
            self.data[byte_index] &= !bit_mask;
        }
        Ok(())
    }

    /// Set all the bits to true
    pub fn set_full(&mut self) {
        self.data.fill(0xFF);
    }

    /// Clear all the bits
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    fn alloc_type(&self) -> AllocType {
        let allocator = *Box::allocator(&self.data);
        if allocator.as_any().downcast_ref::<RsrvAlloc>().is_some() {
            AllocType::Reserve
        } else if allocator.as_any().downcast_ref::<StaticAlloc>().is_some() {
            AllocType::Static
        } else {
            panic!()
        }
    }

    /// Split current bit array at specified position, return a new allocated bit array
    /// corresponding to the bits at the range of [pos, end).
    /// And the current bit array manages the bits at the range of [0, pos).
    pub fn split(&mut self, pos: usize) -> OsResult<BitArray> {
        assert!(pos > 0 && pos < self.bits);

        let byte_index = pos / BYTE_SIZE;
        let bit_index = pos % BYTE_SIZE;

        let lbits = pos;
        let lbytes = bytes_num!(lbits);

        let rbits = self.bits - lbits;
        let rbytes = bytes_num!(rbits);

        let mut rarray = Self::new(rbits, self.alloc_type())?;

        let rdata = &mut rarray.data;
        let ldata = &mut self.data;
        for (idx, item) in rdata[..(rbytes - 1)].iter_mut().enumerate() {
            // current byte index in previous bit_array
            let curr_idx = idx + byte_index;
            let low_bits = ldata[curr_idx] >> bit_index;
            let high_bits = ldata[curr_idx + 1] << (8 - bit_index);
            *item = high_bits | low_bits;
        }
        rdata[rbytes - 1] = ldata[self.bytes - 1] >> bit_index;

        self.bits = lbits;
        self.bytes = lbytes;

        Ok(rarray)
    }
}
