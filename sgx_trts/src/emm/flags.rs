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

// 感觉这里可以优化一下的，因为EMA_RESERVED，EMA_COMMIT_NOW，EMA_COMMIT_ON_DEMAND其实是个enum。
// 可以or ｜ EMA_SYSTEM EMA_GROWSDOWN EMA_GROWSUP 组成的enum。
use bitflags::bitflags;
use sgx_types::error::{SgxResult, SgxStatus};

bitflags! {
    // 用bitflags的话，在ema输入的时候可能存在RESERVED & COMMIT_NOW 需要check一下
    pub struct AllocFlags: u32 {
        const RESERVED = 0b0001;
        const COMMIT_NOW = 0b0010;
        const COMMIT_ON_DEMAND = 0b0100;
        const GROWSDOWN = 0b00010000;
        const GROWSUP = 0b00100000;
        const FIXED = 0b01000000;
    }
}

impl AllocFlags {
    pub fn try_from(value: u32) -> SgxResult<Self> {
        match value {
            0b0000_0001 => Ok(Self::RESERVED),
            0b0000_0010 => Ok(Self::COMMIT_NOW),
            0b0000_0100 => Ok(Self::COMMIT_ON_DEMAND),
            0b0001_0000 => Ok(Self::GROWSDOWN),
            0b0010_0000 => Ok(Self::GROWSUP),
            0b0100_0000 => Ok(Self::FIXED),
            0b0001_0001 => Ok(Self::RESERVED | Self::GROWSDOWN),
            0b0010_0001 => Ok(Self::RESERVED | Self::GROWSUP),
            0b0100_0001 => Ok(Self::RESERVED | Self::FIXED),
            0b0001_0010 => Ok(Self::COMMIT_NOW | Self::GROWSDOWN),
            0b0010_0010 => Ok(Self::COMMIT_NOW | Self::GROWSUP),
            0b0100_0010 => Ok(Self::COMMIT_NOW | Self::FIXED),
            0b0001_0100 => Ok(Self::COMMIT_ON_DEMAND | Self::GROWSDOWN),
            0b0010_0100 => Ok(Self::COMMIT_ON_DEMAND | Self::GROWSUP),
            0b0100_0100 => Ok(Self::COMMIT_ON_DEMAND | Self::FIXED),
            _ => Err(SgxStatus::InvalidParameter),
        }
    }
}

// bitflags! {
//     #[derive(Default)]
//     pub struct SiFlags: u32 {
//         const NONE = 0;
//         const READ = 1 << 0;
//         const WRITE = 1 << 1;
//         const EXEC = 1 << 2;
//         const READ_WRITE = Self::READ.bits | Self::WRITE.bits;
//         const READ_EXEC = Self::READ.bits | Self::EXEC.bits;
//         const READ_WRITE_EXEC = Self::READ.bits | Self::WRITE.bits | Self::EXEC.bits;
//     }
// }

// bitflags! {
//     #[derive(Default)]
//     pub struct PageType: u32 {
//         const NONE = 0;
//         const REG = 1 << 0;
//         const TCS = 1 << 1;
//         const TRIM = 1 << 2;
//         // 相比于sdk,少了一个va
//     }
// }

// #[derive(Clone)]
// // Memory protection info
// #[repr(C)]
// pub struct ProtInfo  {
//     pub si_flags: SiFlags,
//     pub page_type: PageType,
// }
