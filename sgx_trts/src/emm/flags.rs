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

use bitflags::bitflags;
use sgx_types::error::{SgxResult, SgxStatus};

bitflags! {
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
