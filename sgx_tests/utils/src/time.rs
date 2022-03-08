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

use core::arch::x86_64::{__rdtscp, _mm_lfence, _rdtsc};
use core::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct Instant {
    tsc: u64,
    freq: f64,
}

impl Instant {
    #[inline]
    pub fn now(freq: u64) -> Self {
        Instant {
            tsc: Tsc::rdtsc(),
            freq: freq as f64 / 1000_f64,
        }
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        let tsc = Tsc::rdtsc();
        let nanos = (tsc - self.tsc) as f64 / self.freq;
        Duration::from_nanos(nanos as u64)
    }
}

struct Tsc;

impl Tsc {
    #[inline]
    pub fn rdtsc() -> u64 {
        unsafe {
            _mm_lfence();
            let tsc = _rdtsc();
            _mm_lfence();
            tsc
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn rdtscp() -> u64 {
        let mut aux: u32 = 0;
        unsafe { __rdtscp(&mut aux) }
    }
}
