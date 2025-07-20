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

use crate::arch::Align64;
use core::arch::global_asm;

global_asm!(include_str!("cet.S"));
global_asm!(include_str!("macro.S"));

cfg_if! {
    if #[cfg(feature = "sim")] {
        global_asm!(include_str!("../inst/sim/td.S"), options(att_syntax));
        global_asm!(include_str!("../inst/sim/enclu.S"), options(att_syntax));
    } else if #[cfg(feature = "hyper")] {
        global_asm!(include_str!("../inst/hyper/td.S"), options(att_syntax));
        global_asm!(include_str!("../inst/hyper/enclu.S"), options(att_syntax));
    } else {
        global_asm!(include_str!("../inst/hw/td.S"), options(att_syntax));
        global_asm!(include_str!("../inst/hw/enclu.S"), options(att_syntax));
    }
}

#[cfg(feature = "sim")]
global_asm!(include_str!("../inst/sim/lowlib.S"), options(att_syntax));

global_asm!(include_str!("metadata.S"), options(att_syntax));
global_asm!(include_str!("thunk.S"), options(att_syntax));

cfg_if! {
    if #[cfg(feature = "sim")] {
        global_asm!(include_str!("../inst/sim/xsave_mask.S"),options(att_syntax));
    } else if #[cfg(feature = "hyper")] {
        global_asm!(include_str!("../inst/hyper/xsave_mask.S"), options(att_syntax));
    } else {
        global_asm!(include_str!("../inst/hw/xsave_mask.S"), options(att_syntax));
    }
}

#[cfg(all(not(feature = "sim"), not(feature = "hyper")))]
global_asm!(
    include_str!("../inst/hw/everifyreport.S"),
    options(att_syntax)
);

global_asm!(include_str!("xsave.S"), options(att_syntax));
global_asm!(include_str!("pic.S"), options(att_syntax));

const SYNTHETIC_STATE_SIZE: usize = 512 + 64;
#[link_section = ".niprod"]
#[no_mangle]
pub static mut SYNTHETIC_STATE: Align64<[u32; SYNTHETIC_STATE_SIZE / 4]> = Align64([
    0x037F, 0, 0, 0, 0, 0, 0x1FBF, 0xFFFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 2, 0, 2, 0x80000000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn get_synthetic_state_ptr(
) -> &'static mut Align64<[u32; SYNTHETIC_STATE_SIZE / 4]> {
    &mut SYNTHETIC_STATE
}
