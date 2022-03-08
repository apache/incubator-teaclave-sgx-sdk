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

use crate::enclave::EnclaveRange;
use crate::error;
use core::cmp;
use core::fmt;
use core::mem;
use sgx_types::error::{SgxResult, SgxStatus};

pub fn rand(buf: &mut [u8]) -> SgxResult {
    ensure!(
        buf.is_enclave_range() || buf.is_host_range(),
        SgxStatus::InvalidParameter
    );

    let mut left_len = buf.len();
    let mut offset = 0_usize;

    while left_len > 0 {
        let rand_num = rand32()?.to_ne_bytes();
        let copy_len = cmp::min(left_len, mem::size_of::<u32>());
        buf[offset..offset + copy_len].copy_from_slice(&rand_num[..copy_len]);

        left_len -= copy_len;
        offset += copy_len;
    }
    Ok(())
}

#[cfg(feature = "sim")]
#[inline]
fn rand32() -> SgxResult<u32> {
    if is_x86_feature_detected!("rdrand") {
        rdrand()
    } else {
        Ok(rand_lcg())
    }
}

#[cfg(not(feature = "sim"))]
#[inline]
fn rand32() -> SgxResult<u32> {
    rdrand()
}

#[inline]
fn rdrand() -> SgxResult<u32> {
    const RDRAND_RETRY_TIMES: usize = 10;

    unsafe {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::_rdrand32_step;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::_rdrand32_step;

        let mut ret = 0_u32;
        for _in in 0..RDRAND_RETRY_TIMES {
            if _rdrand32_step(&mut ret) == 1 {
                return Ok(ret);
            }
        }
        Err(SgxStatus::Unexpected)
    }
}

#[cfg(feature = "sim")]
fn rand_lcg() -> u32 {
    use crate::inst::GlobalSim;
    use crate::sync::SpinMutex;

    static LOCK: SpinMutex<()> = SpinMutex::new(());
    let _guard = LOCK.lock();

    let seed = &mut GlobalSim::get_mut().seed;
    let rand = unsafe {
        6364136223846793005_u64
            .unchecked_mul(*seed)
            .unchecked_add(1)
    };
    *seed = rand;
    (rand >> 32) as u32
}

#[inline]
pub fn getrandom(buf: &mut [u8]) {
    if getrandom_fill_bytes(buf).is_err() {
        error::abort();
    }

    fn getrandom_fill_bytes(buf: &mut [u8]) -> SgxResult {
        rand(buf)
    }
}

// A random number generator
pub struct Rng;

impl Rng {
    pub fn new() -> Rng {
        Rng
    }

    pub fn next_u32(&mut self) -> u32 {
        next_u32(&mut getrandom)
    }

    pub fn next_u64(&mut self) -> u64 {
        next_u64(&mut getrandom)
    }

    pub fn next_usize(&mut self) -> usize {
        next_usize(&mut getrandom)
    }

    pub fn fill_bytes(&mut self, buf: &mut [u8]) {
        getrandom(buf)
    }
}

impl fmt::Debug for Rng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rng {{}}")
    }
}

impl Default for Rng {
    fn default() -> Rng {
        Rng::new()
    }
}

fn next_u32(fill_buf: &mut dyn FnMut(&mut [u8])) -> u32 {
    let mut buf = [0_u8; 4];
    fill_buf(&mut buf);
    unsafe { mem::transmute::<[u8; 4], u32>(buf) }
}

fn next_u64(fill_buf: &mut dyn FnMut(&mut [u8])) -> u64 {
    let mut buf = [0_u8; 8];
    fill_buf(&mut buf);
    unsafe { mem::transmute::<[u8; 8], u64>(buf) }
}

fn next_usize(fill_buf: &mut dyn FnMut(&mut [u8])) -> usize {
    let mut buf = [0_u8; mem::size_of::<usize>()];
    fill_buf(&mut buf);
    unsafe { mem::transmute::<[u8; mem::size_of::<usize>()], usize>(buf) }
}
