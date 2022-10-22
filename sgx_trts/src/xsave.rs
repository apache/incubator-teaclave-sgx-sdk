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

use sgx_types::types;

pub fn get_xfrm() -> u64 {
    extern "C" {
        pub fn set_xsave_enabled(state: i32);
    }

    cfg_if! {
        if #[cfg(not(feature = "sim"))] {
            use crate::se::AlignReport;
            let report = AlignReport::get_self();
            let xfrm = report.0.body.attributes.xfrm;
        } else {
            use crate::inst::GlobalSim;
            let secs = GlobalSim::get().secs();
            let xfrm = secs.attributes.xfrm;
        }
    }

    let enbaled = i32::from(xfrm != types::XFRM_LEGACY);
    unsafe {
        set_xsave_enabled(enbaled);
    }

    #[cfg(feature = "sim")]
    {
        extern "C" {
            pub fn set_xsave_mask_low(low: u32);
            pub fn set_xsave_mask_high(high: u32);
        }
        unsafe {
            set_xsave_mask_low((xfrm >> 32) as u32);
            set_xsave_mask_high((xfrm & 0xFFFFFFFF) as u32);
        }
    }
    xfrm
}

pub fn is_enabled() -> bool {
    extern "C" {
        fn get_xsave_enabled() -> i32;
    }
    unsafe { get_xsave_enabled() != 0 }
}
