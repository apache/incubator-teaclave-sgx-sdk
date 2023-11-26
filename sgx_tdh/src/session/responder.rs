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

use super::DhResult;
use crate::message::{DhMsg1, DhMsg2, DhMsg3};
use sgx_crypto::ecc::EcKeyPair;
use sgx_trts::trts::EnclaveRange;
use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::DhSessionRole;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum SessionState {
    Error,
    #[default]
    Reset,
    WaitM2,
    Active,
}

#[derive(Debug)]
pub struct Responder {
    role: DhSessionRole,
    state: SessionState,
    key_pair: EcKeyPair,
}

impl Default for Responder {
    fn default() -> Responder {
        Self::new()
    }
}

impl_struct_ContiguousMemory! {
    SessionState;
    Responder;
}

impl Responder {
    pub fn new() -> Responder {
        Responder {
            role: DhSessionRole::Responder,
            state: SessionState::Reset,
            key_pair: EcKeyPair::default(),
        }
    }

    pub fn generate_msg1(&mut self) -> SgxResult<DhMsg1> {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(self.state == SessionState::Reset, SgxStatus::InvalidState);

        let key_pair = EcKeyPair::create()?;
        let msg = DhMsg1::new(&key_pair.public_key())?;

        self.role = DhSessionRole::Responder;
        self.state = SessionState::WaitM2;
        self.key_pair = key_pair;

        Ok(msg)
    }

    pub fn process_msg2(
        &mut self,
        msg2: &DhMsg2,
        add_prop: Option<&[u8]>,
    ) -> SgxResult<(DhMsg3, DhResult)> {
        ensure!(self.is_enclave_range(), SgxStatus::InvalidParameter);
        ensure!(msg2.is_enclave_range(), SgxStatus::InvalidParameter);

        match add_prop {
            Some(add) if !add.is_empty() => {
                ensure!(add.is_enclave_range(), SgxStatus::InvalidParameter);
            }
            _ => (),
        }

        self.check(
            (self.role == DhSessionRole::Responder) && (self.state == SessionState::WaitM2),
            SgxStatus::InvalidState,
        )?;

        let mut shared_key = self.key_pair.shared_key(&msg2.pub_key_b).map_err(|_| {
            self.clear();
            SgxStatus::Unexpected
        })?;

        let mut smk = shared_key.derive_key("SMK".as_bytes()).map_err(|_| {
            self.clear();
            SgxStatus::Unexpected
        })?;

        let mut is_lav2 = false;
        msg2.verify_with_lav1(&self.key_pair.public_key(), &smk)
            .or_else(|_| {
                let ret = msg2.verify_with_lav2(&smk);
                if ret.is_ok() {
                    is_lav2 = true;
                }
                ret
            })
            .map_err(|err| {
                self.clear();
                err
            })?;

        let aek = shared_key.derive_key("AEK".as_bytes()).map_err(|_| {
            self.clear();
            SgxStatus::Unexpected
        })?;

        let msg3 = if is_lav2 {
            DhMsg3::new_with_lav2(msg2, &self.key_pair.public_key(), &smk, add_prop).map_err(
                |err| {
                    self.clear();
                    err
                },
            )?
        } else {
            DhMsg3::new_with_lav1(msg2, &self.key_pair.public_key(), &smk, add_prop).map_err(
                |err| {
                    self.clear();
                    err
                },
            )?
        };

        shared_key.clear();
        smk.as_mut().fill(0);
        self.clear();
        self.state = SessionState::Active;

        let dh_result = DhResult {
            aek: aek.key,
            enclave_identity: From::from(&msg2.report),
        };

        Ok((msg3, dh_result))
    }

    #[inline]
    fn check(&mut self, cond: bool, err: SgxStatus) -> SgxResult {
        if !cond {
            self.clear();
            return Err(err);
        }
        Ok(())
    }

    #[inline]
    fn clear(&mut self) {
        unsafe { (self as *mut Responder).write_bytes(0, 1) }
        self.state = SessionState::Error;
    }
}
