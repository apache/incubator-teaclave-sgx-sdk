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

use crate::feature::SysFeatures;
use sgx_types::types::EnclaveMode;

pub use crate::call::OcBuffer;
pub use crate::enclave::at_exit;
pub use crate::enclave::MmLayout;
pub use crate::enclave::{is_within_enclave, is_within_host, EnclaveRange};
pub use crate::error::abort;
pub use crate::feature::Version;

#[inline]
pub fn enclave_mode() -> EnclaveMode {
    cfg_if! {
        if #[cfg(feature = "sim")] {
            EnclaveMode::Sim
        } else if #[cfg(feature = "hyper")] {
            EnclaveMode::Hyper
        } else {
            EnclaveMode::Hw
        }
    }
}

#[inline]
pub fn is_supported_edmm() -> bool {
    SysFeatures::get().is_edmm()
}

#[inline]
pub fn version() -> Version {
    SysFeatures::get().version()
}

#[inline]
pub fn cpu_core_num() -> u32 {
    SysFeatures::get().core_mum()
}
