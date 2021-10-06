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

//! # Diffie–Hellman (DH) Session Establishment Functions
//!
//! These functions allow an ISV to establish secure session between two enclaves using the EC DH Key exchange protocol.
//!
use crate::ecp::*;
use alloc::boxed::Box;
use alloc::slice;
use alloc::vec::Vec;
use core::mem;
use core::ptr;
use sgx_tcrypto::*;
use sgx_trts::memeq::ConsttimeMemEq;
use sgx_trts::trts::*;
use sgx_tse::*;
use sgx_types::marker::ContiguousMemory;
use sgx_types::*;

const AES_CMAC_KDF_ID: [u8; 2] = [1, 0];

pub type SgxDhMsg1 = sgx_dh_msg1_t;
pub type SgxDhMsg2 = sgx_dh_msg2_t;

/// Type for message body of the MSG3 structure used in DH secure session establishment.
#[derive(Clone, Default)]
pub struct SgxDhMsg3Body {
    pub report: sgx_report_t,
    pub additional_prop: Box<[u8]>,
}

/// Type for MSG3 used in DH secure session establishment.
#[derive(Clone, Default)]
pub struct SgxDhMsg3 {
    pub cmac: [u8; SGX_DH_MAC_SIZE],
    pub msg3_body: SgxDhMsg3Body,
}

impl SgxDhMsg3 {
    ///
    /// Create a SgxDhMsg3 with default values.
    ///
    pub fn new() -> SgxDhMsg3 {
        SgxDhMsg3::default()
    }

    ///
    /// Calculate the size of sgx_dh_msg3_t converted from SgxDhMsg3, really add the size of struct sgx_dh_msg3_t and msg3_body.additional_prop.
    ///
    /// # Return value
    ///
    /// The size of sgx_dh_msg3_t needed.
    ///
    pub fn calc_raw_sealed_data_size(&self) -> u32 {
        let max = u32::MAX;
        let dh_msg3_size = mem::size_of::<sgx_dh_msg3_t>();
        let additional_prop_len = self.msg3_body.additional_prop.len();

        if additional_prop_len > (max as usize) - dh_msg3_size {
            return max;
        }

        (dh_msg3_size + additional_prop_len) as u32
    }

    ///
    /// Convert SgxDhMsg3 to sgx_dh_msg3_t, this is an unsafe function.
    ///
    /// # Parameters
    ///
    /// **p**
    ///
    /// The pointer of a sgx_dh_msg3_t buffer to save the buffer of SgxDhMsg3.
    ///
    /// **len**
    ///
    /// The size of the sgx_dh_msg3_t buffer.
    ///
    /// # Return value
    ///
    /// **Some(*mut sgx_dh_msg3_t)**
    ///
    /// Indicates the conversion is successfully. The return value is the mutable pointer of sgx_dh_msg3_t.
    ///
    /// **None**
    ///
    /// The parameters p and len are not available for the conversion.
    ///
    pub unsafe fn to_raw_dh_msg3_t(
        &self,
        p: *mut sgx_dh_msg3_t,
        len: u32,
    ) -> Option<*mut sgx_dh_msg3_t> {
        if p.is_null() {
            return None;
        }
        if !rsgx_raw_is_within_enclave(p as *mut u8, len as usize) {
            return None;
        }

        let additional_prop_len = self.msg3_body.additional_prop.len();
        let dh_msg3_size = mem::size_of::<sgx_dh_msg3_t>();
        if additional_prop_len > u32::MAX as usize - dh_msg3_size {
            return None;
        }
        if len < (dh_msg3_size + additional_prop_len) as u32 {
            return None;
        }

        let dh_msg3 = &mut *p;
        dh_msg3.cmac = self.cmac;
        dh_msg3.msg3_body.report = self.msg3_body.report;
        dh_msg3.msg3_body.additional_prop_length = additional_prop_len as u32;

        if additional_prop_len > 0 {
            let raw_msg3 = slice::from_raw_parts_mut(p as *mut u8, len as usize);
            raw_msg3[dh_msg3_size..].copy_from_slice(&self.msg3_body.additional_prop);
        }
        Some(p)
    }

    ///
    /// Convert sgx_dh_msg3_t to SgxDhMsg3, this is an unsafe function.
    ///
    /// # Parameters
    ///
    /// **p**
    ///
    /// The pointer of a sgx_dh_msg3_t buffer.
    ///
    /// **len**
    ///
    /// The size of the sgx_dh_msg3_t buffer.
    ///
    /// # Return value
    ///
    /// **Some(SgxDhMsg3)**
    ///
    /// Indicates the conversion is successfully. The return value is SgxDhMsg3.
    ///
    /// **None**
    ///
    /// The parameters p and len are not available for the conversion.
    ///
    pub unsafe fn from_raw_dh_msg3_t(p: *mut sgx_dh_msg3_t, len: u32) -> Option<SgxDhMsg3> {
        if p.is_null() {
            return None;
        }
        if !rsgx_raw_is_within_enclave(p as *mut u8, len as usize) {
            return None;
        }

        let raw_msg3 = &*p;
        let additional_prop_len = raw_msg3.msg3_body.additional_prop_length;
        let dh_msg3_size = mem::size_of::<sgx_dh_msg3_t>() as u32;
        if additional_prop_len > u32::MAX - dh_msg3_size {
            return None;
        }
        if len < dh_msg3_size + additional_prop_len {
            return None;
        }

        let mut dh_msg3 = SgxDhMsg3 {
            cmac: raw_msg3.cmac,
            msg3_body: SgxDhMsg3Body {
                report: raw_msg3.msg3_body.report,
                additional_prop: Default::default(),
            },
        };

        if additional_prop_len > 0 {
            let mut additional_prop: Vec<u8> = vec![0_u8; additional_prop_len as usize];
            let ptr_additional_prop = p.offset(1) as *const u8;
            ptr::copy_nonoverlapping(
                ptr_additional_prop,
                additional_prop.as_mut_ptr(),
                additional_prop_len as usize,
            );
            dh_msg3.msg3_body.additional_prop = additional_prop.into_boxed_slice();
        }
        Some(dh_msg3)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum SgxDhSessionState {
    SGX_DH_SESSION_STATE_ERROR,
    SGX_DH_SESSION_STATE_RESET,
    SGX_DH_SESSION_RESPONDER_WAIT_M2,
    SGX_DH_SESSION_INITIATOR_WAIT_M1,
    SGX_DH_SESSION_INITIATOR_WAIT_M3,
    SGX_DH_SESSION_ACTIVE,
}

/// DH secure session responder
#[derive(Copy, Clone)]
pub struct SgxDhResponder {
    state: SgxDhSessionState,
    prv_key: sgx_align_ec256_private_t,
    pub_key: sgx_ec256_public_t,
    smk_aek: sgx_align_key_128bit_t,
    shared_key: sgx_align_ec256_dh_shared_t,
}

impl Default for SgxDhResponder {
    fn default() -> SgxDhResponder {
        SgxDhResponder {
            state: SgxDhSessionState::SGX_DH_SESSION_STATE_RESET,
            prv_key: sgx_align_ec256_private_t::default(),
            pub_key: sgx_ec256_public_t::default(),
            smk_aek: sgx_align_key_128bit_t::default(),
            shared_key: sgx_align_ec256_dh_shared_t::default(),
        }
    }
}

unsafe impl ContiguousMemory for SgxDhResponder {}

impl SgxDhResponder {
    ///
    /// Initialize DH secure session responder.
    ///
    /// Indicates role of responder  the caller plays in the secure session establishment.
    ///
    /// The value of role of the responder of the session establishment must be `SGX_DH_SESSION_RESPONDER`.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    pub fn init_session() -> SgxDhResponder {
        Self::default()
    }
    ///
    /// Generates MSG1 for the responder of DH secure session establishment and records ECC key pair in session structure.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **msg1**
    ///
    /// A pointer to an SgxDhMsg1 msg1 buffer. The buffer holding the msg1
    /// message, which is referenced by this parameter, must be within the enclave.
    /// The DH msg1 contains the responder’s public key and report based target
    /// info.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the input parameters is incorrect.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The API is invoked in incorrect order or state.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// An unexpected error occurred.
    ///
    pub fn gen_msg1(&mut self, msg1: &mut SgxDhMsg1) -> SgxError {
        if !rsgx_data_is_within_enclave(self) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if !rsgx_data_is_within_enclave(msg1) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if self.state != SgxDhSessionState::SGX_DH_SESSION_STATE_RESET {
            *self = Self::default();
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let error = self.dh_generate_message1(msg1);
        if let Err(mut ret) = error {
            *self = Self::default();
            if ret != sgx_status_t::SGX_ERROR_OUT_OF_MEMORY {
                ret = sgx_status_t::SGX_ERROR_UNEXPECTED;
            }
            return Err(ret);
        }

        self.state = SgxDhSessionState::SGX_DH_SESSION_RESPONDER_WAIT_M2;
        Ok(())
    }

    ///
    /// The responder handles msg2 sent by initiator and then derives AEK, updates session information and generates msg3.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **msg2**
    ///
    /// Point to dh message 2 buffer generated by session initiator, and the buffer must be in enclave address space.
    ///
    /// **msg3**
    ///
    /// Point to dh message 3 buffer generated by session responder in this function, and the buffer must be in enclave address space.
    ///
    /// **aek**
    ///
    /// A pointer that points to instance of sgx_key_128bit_t. The aek is derived as follows:
    ///
    /// ```
    /// KDK := CMAC(key0, LittleEndian(gab x-coordinate))
    /// AEK = AES-CMAC(KDK, 0x01||"AEK"||0x00||0x80||0x00)
    /// ```
    /// The key0 used in the key extraction operation is 16 bytes of 0x00. The plain
    /// text used in the AES-CMAC calculation of the KDK is the Diffie-Hellman shared
    /// secret elliptic curve field element in Little Endian format.The plain text used
    /// in the AEK calculation includes:
    ///
    /// * a counter (0x01)
    ///
    /// * a label: the ASCII representation of the string 'AEK' in Little Endian format
    ///
    /// * a bit length (0x80)
    ///
    /// **initiator_identity**
    ///
    /// A pointer that points to instance of sgx_dh_session_enclave_identity_t.
    /// Identity information of initiator includes isv svn, isv product id, the
    /// enclave attributes, MRSIGNER, and MRENCLAVE. The buffer must be in
    /// enclave address space. The caller should check the identity of the peer and
    /// decide whether to trust the peer and use the aek.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the input parameters is incorrect.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The API is invoked in incorrect order or state.
    ///
    /// **SGX_ERROR_KDF_MISMATCH**
    ///
    /// Indicates the key derivation function does not match.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// An unexpected error occurred.
    ///
    pub fn proc_msg2(
        &mut self,
        msg2: &SgxDhMsg2,
        msg3: &mut SgxDhMsg3,
        aek: &mut sgx_key_128bit_t,
        initiator_identity: &mut sgx_dh_session_enclave_identity_t,
    ) -> SgxError {
        if !rsgx_data_is_within_enclave(self) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if !rsgx_data_is_within_enclave(msg2)
            || !rsgx_data_is_within_enclave(aek)
            || !rsgx_data_is_within_enclave(initiator_identity)
            || !rsgx_raw_is_within_enclave(
                msg3 as *const _ as *const u8,
                mem::size_of::<SgxDhMsg3>(),
            )
        {
            *self = Self::default();
            self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if msg3.msg3_body.additional_prop.len() > 0
            && (!(rsgx_slice_is_within_enclave(&msg3.msg3_body.additional_prop))
                || (msg3.msg3_body.additional_prop.len()
                    > (u32::MAX as usize) - mem::size_of::<sgx_dh_msg3_t>()))
        {
            *self = Self::default();
            self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if self.state != SgxDhSessionState::SGX_DH_SESSION_RESPONDER_WAIT_M2 {
            *self = Self::default();
            self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ecc_state = SgxEccHandle::new();
        ecc_state.open().map_err(|ret| self.set_error(ret))?;
        self.shared_key = ecc_state
            .compute_align_shared_dhkey(&self.prv_key.key, &msg2.g_b)
            .map_err(|ret| self.set_error(ret))?;

        self.smk_aek =
            derive_key(&self.shared_key.key, &EC_SMK_LABEL).map_err(|ret| self.set_error(ret))?;

        #[cfg(feature = "use_lav2")]
        self.lav2_verify_message2(msg2)
            .map_err(|ret| self.set_error(ret))?;
        #[cfg(not(feature = "use_lav2"))]
        self.dh_verify_message2(msg2)
            .map_err(|ret| self.set_error(ret))?;

        initiator_identity.isv_svn = msg2.report.body.isv_svn;
        initiator_identity.isv_prod_id = msg2.report.body.isv_prod_id;
        initiator_identity.attributes = msg2.report.body.attributes;
        initiator_identity.mr_signer = msg2.report.body.mr_signer;
        initiator_identity.mr_enclave = msg2.report.body.mr_enclave;

        #[cfg(feature = "use_lav2")]
        self.lav2_generate_message3(msg2, msg3)
            .map_err(|ret| self.set_error(ret))?;
        #[cfg(not(feature = "use_lav2"))]
        self.dh_generate_message3(msg2, msg3)
            .map_err(|ret| self.set_error(ret))?;

        let align_aek =
            derive_key(&self.shared_key.key, &EC_AEK_LABEL).map_err(|ret| self.set_error(ret))?;
        *aek = align_aek.key;
        *self = Self::default();
        self.state = SgxDhSessionState::SGX_DH_SESSION_ACTIVE;

        Ok(())
    }

    fn dh_generate_message1(&mut self, msg1: &mut SgxDhMsg1) -> SgxError {
        msg1.target = Default::default();
        msg1.g_a = Default::default();

        let mut target = sgx_target_info_t::default();
        let report_data = sgx_report_data_t::default();

        let report = rsgx_create_report(&target, &report_data)?;
        SGX_LAV2_PROTO_SPEC.make_target_info(&report, &mut target)?;

        let ecc_state = SgxEccHandle::new();
        ecc_state.open()?;
        let (prv_key, pub_key) = ecc_state.create_align_key_pair()?;

        self.prv_key = prv_key;
        self.pub_key = pub_key;
        msg1.g_a = pub_key;
        msg1.target = target;

        Ok(())
    }

    fn dh_verify_message2(&self, msg2: &SgxDhMsg2) -> SgxError {
        let kdf_id =
            &msg2.report.body.report_data.d[SGX_SHA256_HASH_SIZE..SGX_SHA256_HASH_SIZE + 2];
        let data_hash = &msg2.report.body.report_data.d[..SGX_SHA256_HASH_SIZE];

        if !kdf_id.eq(&AES_CMAC_KDF_ID) {
            return Err(sgx_status_t::SGX_ERROR_KDF_MISMATCH);
        }

        let report = msg2.report;
        let data_mac = rsgx_rijndael128_cmac_msg(&self.smk_aek.key, &report)?;
        if !data_mac.consttime_memeq(&msg2.cmac) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        rsgx_verify_report(&report)?;

        let sha_handle = SgxShaHandle::new();
        sha_handle.init()?;
        sha_handle.update_msg(&self.pub_key)?;
        sha_handle.update_msg(&msg2.g_b)?;
        let msg_hash = sha_handle.get_hash()?;

        if !msg_hash.eq(data_hash) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        Ok(())
    }

    fn lav2_verify_message2(&self, msg2: &SgxDhMsg2) -> SgxError {
        let sha_handle = SgxShaHandle::new();
        sha_handle.init()?;
        sha_handle.update_msg(&msg2.report.body.report_data)?;
        sha_handle.update_msg(&msg2.g_b)?;
        let msg_hash = sha_handle.get_hash()?;

        let mut report = msg2.report;
        report.body.report_data = sgx_report_data_t::default();
        report.body.report_data.d[..SGX_SHA256_HASH_SIZE].copy_from_slice(&msg_hash);

        rsgx_verify_report(&report)?;
        let data_mac = rsgx_rijndael128_cmac_msg(&self.smk_aek.key, &msg2.g_b)?;
        if !data_mac.consttime_memeq(&msg2.cmac) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        let proto_spec =
            unsafe { SgxLAv2ProtoSpec::from_report_data(&msg2.report.body.report_data) };
        if (!proto_spec.signature.eq(&SGX_LAV2_PROTO_SPEC.signature))
            || (proto_spec.ver != SGX_LAV2_PROTO_SPEC.ver)
        {
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }

        Ok(())
    }

    fn dh_generate_message3(&self, msg2: &SgxDhMsg2, msg3: &mut SgxDhMsg3) -> SgxError {
        msg3.cmac = Default::default();
        msg3.msg3_body.report = Default::default();

        let sha_handle = SgxShaHandle::new();
        sha_handle.init()?;
        sha_handle.update_msg(&msg2.g_b)?;
        sha_handle.update_msg(&self.pub_key)?;
        let msg_hash = sha_handle.get_hash()?;

        let mut target = sgx_target_info_t::default();
        let mut report_data = sgx_report_data_t::default();
        let report = msg2.report;

        report_data.d[..SGX_SHA256_HASH_SIZE].copy_from_slice(&msg_hash);
        SGX_LAV2_PROTO_SPEC.make_target_info(&report, &mut target)?;
        msg3.msg3_body.report = rsgx_create_report(&target, &report_data)?;

        let add_prop_len = msg3.msg3_body.additional_prop.len() as u32;
        let cmac_handle = SgxCmacHandle::new();
        cmac_handle.init(&self.smk_aek.key)?;
        cmac_handle.update_msg(&msg3.msg3_body.report)?;
        cmac_handle.update_msg(&add_prop_len)?;
        if add_prop_len > 0 {
            cmac_handle.update_slice(&msg3.msg3_body.additional_prop)?;
        }
        msg3.cmac = cmac_handle.get_hash()?;

        Ok(())
    }

    fn lav2_generate_message3(&self, msg2: &SgxDhMsg2, msg3: &mut SgxDhMsg3) -> SgxError {
        msg3.cmac = Default::default();
        msg3.msg3_body.report = Default::default();

        let proto_spec =
            unsafe { SgxLAv2ProtoSpec::from_report_data(&msg2.report.body.report_data) };
        let mut target = sgx_target_info_t::default();
        let mut report_data = sgx_report_data_t::default();
        let report = msg2.report;

        let sha_handle = SgxShaHandle::new();
        sha_handle.init()?;
        sha_handle.update_msg(&self.pub_key)?;
        sha_handle.update_msg(&proto_spec)?;
        let msg_hash = sha_handle.get_hash()?;

        report_data.d[..SGX_SHA256_HASH_SIZE].copy_from_slice(&msg_hash);
        SGX_LAV2_PROTO_SPEC.make_target_info(&report, &mut target)?;
        msg3.msg3_body.report = rsgx_create_report(&target, &report_data)?;

        let cmac_handle = SgxCmacHandle::new();
        cmac_handle.init(&self.smk_aek.key)?;
        if msg3.msg3_body.additional_prop.len() > 0 {
            cmac_handle.update_slice(&msg3.msg3_body.additional_prop)?;
        }
        cmac_handle.update_msg(&self.pub_key)?;
        msg3.cmac = cmac_handle.get_hash()?;

        Ok(())
    }

    fn set_error(&mut self, sgx_ret: sgx_status_t) -> sgx_status_t {
        *self = Self::default();
        self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
        match sgx_ret {
            sgx_status_t::SGX_ERROR_OUT_OF_MEMORY => sgx_status_t::SGX_ERROR_OUT_OF_MEMORY,
            sgx_status_t::SGX_ERROR_KDF_MISMATCH => sgx_status_t::SGX_ERROR_KDF_MISMATCH,
            _ => sgx_status_t::SGX_ERROR_UNEXPECTED,
        }
    }
}

/// DH secure session Initiator
#[derive(Copy, Clone)]
pub struct SgxDhInitiator {
    state: SgxDhSessionState,
    smk_aek: sgx_align_key_128bit_t,
    pub_key: sgx_ec256_public_t,
    peer_pub_key: sgx_ec256_public_t,
    shared_key: sgx_align_ec256_dh_shared_t,
}

impl Default for SgxDhInitiator {
    fn default() -> SgxDhInitiator {
        SgxDhInitiator {
            state: SgxDhSessionState::SGX_DH_SESSION_INITIATOR_WAIT_M1,
            smk_aek: sgx_align_key_128bit_t::default(),
            pub_key: sgx_ec256_public_t::default(),
            peer_pub_key: sgx_ec256_public_t::default(),
            shared_key: sgx_align_ec256_dh_shared_t::default(),
        }
    }
}

unsafe impl ContiguousMemory for SgxDhInitiator {}

impl SgxDhInitiator {
    ///
    /// Initialize DH secure session Initiator.
    ///
    /// Indicates role of initiator the caller plays in the secure session establishment.
    ///
    /// The value of role of the initiator of the session establishment must be `SGX_DH_SESSION_INITIATOR`.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    pub fn init_session() -> SgxDhInitiator {
        Self::default()
    }

    ///
    /// The initiator of DH secure session establishment handles msg1 sent by responder and then generates msg2,
    /// and records initiator’s ECC key pair in DH session structure.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **msg1**
    ///
    /// Point to dh message 1 buffer generated by session responder, and the buffer must be in enclave address space.
    ///
    /// **msg2**
    ///
    /// Point to dh message 2 buffer, and the buffer must be in enclave address space.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the input parameters is incorrect.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The API is invoked in incorrect order or state.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// An unexpected error occurred.
    ///
    pub fn proc_msg1(&mut self, msg1: &SgxDhMsg1, msg2: &mut SgxDhMsg2) -> SgxError {
        if !rsgx_data_is_within_enclave(self) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if !rsgx_data_is_within_enclave(msg1) || !rsgx_data_is_within_enclave(msg2) {
            *self = Self::default();
            self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if self.state != SgxDhSessionState::SGX_DH_SESSION_INITIATOR_WAIT_M1 {
            *self = Self::default();
            self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        let ecc_state = SgxEccHandle::new();
        ecc_state.open().map_err(|ret| self.set_error(ret))?;
        let (mut prv_key, pub_key) = ecc_state
            .create_align_key_pair()
            .map_err(|ret| self.set_error(ret))?;
        self.shared_key = ecc_state
            .compute_align_shared_dhkey(&prv_key.key, &msg1.g_a)
            .map_err(|ret| self.set_error(ret))?;

        prv_key = sgx_align_ec256_private_t::default();
        self.pub_key = pub_key;
        self.smk_aek =
            derive_key(&self.shared_key.key, &EC_SMK_LABEL).map_err(|ret| self.set_error(ret))?;

        #[cfg(feature = "use_lav2")]
        self.lav2_generate_message2(msg1, msg2)
            .map_err(|ret| self.set_error(ret))?;
        #[cfg(not(feature = "use_lav2"))]
        self.dh_generate_message2(msg1, msg2)
            .map_err(|ret| self.set_error(ret))?;

        self.peer_pub_key = msg1.g_a;
        self.state = SgxDhSessionState::SGX_DH_SESSION_INITIATOR_WAIT_M3;

        Ok(())
    }

    ///
    /// The initiator handles msg3 sent by responder and then derives AEK, updates
    /// session information and gets responder’s identity information.
    ///
    /// # Requirements
    ///
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Parameters
    ///
    /// **msg3**
    ///
    /// Point to dh message 3 buffer generated by session responder, and the buffer must be in enclave address space.
    ///
    /// **aek**
    ///
    /// A pointer that points to instance of sgx_key_128bit_t. The aek is derived as follows:
    ///
    /// ```
    /// KDK:= CMAC(key0, LittleEndian(gab x-coordinate))
    /// AEK = AES-CMAC(KDK, 0x01||"AEK"||0x00||0x80||0x00)
    /// ```
    ///
    /// The key0 used in the key extraction operation is 16 bytes of 0x00. The plain
    /// text used in the AES-CMAC calculation of the KDK is the Diffie-Hellman shared
    /// secret elliptic curve field element in Little Endian format.
    /// The plain text used in the AEK calculation includes:
    ///
    /// * a counter (0x01)
    ///
    /// * a label: the ASCII representation of the string 'AEK' in Little Endian format
    ///
    /// * a bit length (0x80)
    ///
    /// **responder_identity**
    ///
    /// Identity information of responder including isv svn, isv product id, the enclave
    /// attributes, MRSIGNER, and MRENCLAVE. The buffer must be in enclave address space.
    /// The caller should check the identity of the peer and decide whether to trust the
    /// peer and use the aek or the msg3_body.additional_prop field of msg3.
    ///
    /// # Errors
    ///
    /// **SGX_ERROR_INVALID_PARAMETER**
    ///
    /// Any of the input parameters is incorrect.
    ///
    /// **SGX_ERROR_INVALID_STATE**
    ///
    /// The API is invoked in incorrect order or state.
    ///
    /// **SGX_ERROR_OUT_OF_MEMORY**
    ///
    /// The enclave is out of memory.
    ///
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// An unexpected error occurred.
    ///
    pub fn proc_msg3(
        &mut self,
        msg3: &SgxDhMsg3,
        aek: &mut sgx_key_128bit_t,
        responder_identity: &mut sgx_dh_session_enclave_identity_t,
    ) -> SgxError {
        if !rsgx_data_is_within_enclave(self) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if !rsgx_raw_is_within_enclave(msg3 as *const _ as *const u8, mem::size_of::<SgxDhMsg3>())
            || !rsgx_data_is_within_enclave(aek)
            || !rsgx_data_is_within_enclave(responder_identity)
        {
            *self = Self::default();
            self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        if msg3.msg3_body.additional_prop.len() > 0
            && (!rsgx_slice_is_within_enclave(&msg3.msg3_body.additional_prop)
                || (msg3.msg3_body.additional_prop.len()
                    > (u32::MAX as usize) - mem::size_of::<sgx_dh_msg3_t>()))
        {
            *self = Self::default();
            self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        if self.state != SgxDhSessionState::SGX_DH_SESSION_INITIATOR_WAIT_M3 {
            *self = Self::default();
            self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
            return Err(sgx_status_t::SGX_ERROR_INVALID_STATE);
        }

        #[cfg(feature = "use_lav2")]
        self.lav2_verify_message3(msg3)
            .map_err(|ret| self.set_error(ret))?;
        #[cfg(not(feature = "use_lav2"))]
        self.dh_verify_message3(msg3)
            .map_err(|ret| self.set_error(ret))?;

        let align_aek =
            derive_key(&self.shared_key.key, &EC_AEK_LABEL).map_err(|ret| self.set_error(ret))?;
        *aek = align_aek.key;

        *self = Self::default();
        self.state = SgxDhSessionState::SGX_DH_SESSION_ACTIVE;

        responder_identity.cpu_svn = msg3.msg3_body.report.body.cpu_svn;
        responder_identity.misc_select = msg3.msg3_body.report.body.misc_select;
        responder_identity.isv_svn = msg3.msg3_body.report.body.isv_svn;
        responder_identity.isv_prod_id = msg3.msg3_body.report.body.isv_prod_id;
        responder_identity.attributes = msg3.msg3_body.report.body.attributes;
        responder_identity.mr_signer = msg3.msg3_body.report.body.mr_signer;
        responder_identity.mr_enclave = msg3.msg3_body.report.body.mr_enclave;

        Ok(())
    }

    fn dh_generate_message2(&self, msg1: &SgxDhMsg1, msg2: &mut SgxDhMsg2) -> SgxError {
        msg2.report = Default::default();
        msg2.cmac = Default::default();
        msg2.g_b = self.pub_key;

        let sha_handle = SgxShaHandle::new();
        sha_handle.init()?;
        sha_handle.update_msg(&msg1.g_a)?;
        sha_handle.update_msg(&msg2.g_b)?;
        let msg_hash = sha_handle.get_hash()?;

        let mut report_data = sgx_report_data_t::default();
        report_data.d[..SGX_SHA256_HASH_SIZE].copy_from_slice(&msg_hash);
        report_data.d[SGX_SHA256_HASH_SIZE..SGX_SHA256_HASH_SIZE + 2]
            .copy_from_slice(&AES_CMAC_KDF_ID);

        let target = msg1.target;
        msg2.report = rsgx_create_report(&target, &report_data)?;
        let report = msg2.report;
        msg2.cmac = rsgx_rijndael128_cmac_msg(&self.smk_aek.key, &report)?;

        Ok(())
    }

    fn lav2_generate_message2(&self, msg1: &SgxDhMsg1, msg2: &mut SgxDhMsg2) -> SgxError {
        msg2.report = Default::default();
        msg2.cmac = Default::default();
        msg2.g_b = self.pub_key;

        let sha_handle = SgxShaHandle::new();
        sha_handle.init()?;
        sha_handle.update_msg(&SGX_LAV2_PROTO_SPEC)?;
        sha_handle.update_msg(&msg2.g_b)?;
        let msg_hash = sha_handle.get_hash()?;

        let target = msg1.target;
        let mut report_data = sgx_report_data_t::default();
        report_data.d[..SGX_SHA256_HASH_SIZE].copy_from_slice(&msg_hash);

        msg2.report = rsgx_create_report(&target, &report_data)?;
        // Replace report_data with proto_spec
        unsafe {
            msg2.report.body.report_data = SGX_LAV2_PROTO_SPEC.to_report_data();
        }
        msg2.cmac = rsgx_rijndael128_cmac_msg(&self.smk_aek.key, &msg2.g_b)?;

        Ok(())
    }

    fn dh_verify_message3(&self, msg3: &SgxDhMsg3) -> SgxError {
        let add_prop_len = msg3.msg3_body.additional_prop.len() as u32;

        let cmac_handle = SgxCmacHandle::new();
        cmac_handle.init(&self.smk_aek.key)?;
        cmac_handle.update_msg(&msg3.msg3_body.report)?;
        cmac_handle.update_msg(&add_prop_len)?;
        if add_prop_len > 0 {
            cmac_handle.update_slice(&msg3.msg3_body.additional_prop)?;
        }
        let data_mac = cmac_handle.get_hash()?;

        if !data_mac.consttime_memeq(&msg3.cmac) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        rsgx_verify_report(&msg3.msg3_body.report)?;

        let sha_handle = SgxShaHandle::new();
        sha_handle.init()?;
        sha_handle.update_msg(&self.pub_key)?;
        sha_handle.update_msg(&self.peer_pub_key)?;
        let msg_hash = sha_handle.get_hash()?;

        let data_hash = &msg3.msg3_body.report.body.report_data.d[..SGX_SHA256_HASH_SIZE];
        if !msg_hash.eq(data_hash) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        Ok(())
    }

    fn lav2_verify_message3(&self, msg3: &SgxDhMsg3) -> SgxError {
        let sha_handle = SgxShaHandle::new();
        sha_handle.init()?;
        sha_handle.update_msg(&self.peer_pub_key)?;
        sha_handle.update_msg(&SGX_LAV2_PROTO_SPEC)?;
        let msg_hash = sha_handle.get_hash()?;

        let mut report = msg3.msg3_body.report;
        report.body.report_data = sgx_report_data_t::default();
        report.body.report_data.d[..SGX_SHA256_HASH_SIZE].copy_from_slice(&msg_hash);

        if !&report.body.report_data.d[..].eq(&msg3.msg3_body.report.body.report_data.d[..]) {
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }

        rsgx_verify_report(&report)?;

        let cmac_handle = SgxCmacHandle::new();
        cmac_handle.init(&self.smk_aek.key)?;
        if msg3.msg3_body.additional_prop.len() > 0 {
            cmac_handle.update_slice(&msg3.msg3_body.additional_prop)?;
        }
        cmac_handle.update_msg(&self.peer_pub_key)?;
        let data_mac = cmac_handle.get_hash()?;

        if !data_mac.consttime_memeq(&msg3.cmac) {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }

        Ok(())
    }

    fn set_error(&mut self, sgx_ret: sgx_status_t) -> sgx_status_t {
        *self = Self::default();
        self.state = SgxDhSessionState::SGX_DH_SESSION_STATE_ERROR;
        match sgx_ret {
            sgx_status_t::SGX_ERROR_OUT_OF_MEMORY => sgx_status_t::SGX_ERROR_OUT_OF_MEMORY,
            _ => sgx_status_t::SGX_ERROR_UNEXPECTED,
        }
    }
}

#[derive(Copy, Clone, Default)]
struct SgxLAv2ProtoSpec {
    signature: [u8; 6],
    ver: u8,
    rev: u8,
    target_spec: [u16; 28],
}

unsafe impl ContiguousMemory for SgxLAv2ProtoSpec {}

impl SgxLAv2ProtoSpec {
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn to_report_data(&self) -> sgx_report_data_t {
        mem::transmute::<SgxLAv2ProtoSpec, sgx_report_data_t>(*self)
    }

    pub unsafe fn from_report_data(data: &sgx_report_data_t) -> SgxLAv2ProtoSpec {
        mem::transmute::<sgx_report_data_t, SgxLAv2ProtoSpec>(*data)
    }

    pub fn ts_count(&self) -> u16 {
        self.target_spec[0] >> 8
    }

    pub fn is_valid(&self) -> bool {
        self.ver == 2 && self.rev == 0 && self.target_spec[0] as u8 == 0 && self.ts_count() < 28
    }

    pub fn make_target_info(&self, rpt: &sgx_report_t, ti: &mut sgx_target_info_t) -> SgxError {
        if !self.is_valid() {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }

        let d = ti as *mut sgx_target_info_t as *mut u8;
        let f = rpt as *const sgx_report_t as *const u8;
        rsgx_lfence();

        let mut to: i32 = 0;
        for i in 1..(self.ts_count() + 1) as usize {
            let size: i32 = 1 << (self.target_spec[i] & 0xF);
            to += size - 1;
            to &= -size;
            if (to + size) as usize > mem::size_of::<sgx_target_info_t>() {
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }

            let from: i32 = (self.target_spec[i] >> 4) as i32;
            if from >= 0 {
                if (from + size) as usize > mem::size_of::<sgx_report_t>() {
                    return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
                }
                unsafe {
                    ptr::copy_nonoverlapping(
                        f.offset(from as isize),
                        d.offset(to as isize),
                        size as usize,
                    );
                }
            } else if from == -1 {
                break;
            } else {
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
            to += size;
        }
        Ok(())
    }
}

const SGX_LAV2_PROTO_SPEC: SgxLAv2ProtoSpec = SgxLAv2ProtoSpec {
    signature: [0x53, 0x47, 0x58, 0x20, 0x4C, 0x41], // "SGX LA"
    ver: 2,
    rev: 0,
    target_spec: [
        0x0600, // target_spec count & revision
        0x0405, // MRENCLAVE
        0x0304, // ATTRIBUTES
        0x0140, // CET_ATTRIBUTES
        0x1041, // CONFIGSVN
        0x0102, // MISCSELECT
        0x0C06, // CONFIGID
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
};

pub fn rsgx_self_target() -> SgxResult<sgx_target_info_t> {
    let mut target_info = sgx_target_info_t::default();
    let report = rsgx_self_report();
    SGX_LAV2_PROTO_SPEC
        .make_target_info(&report, &mut target_info)
        .map(|_| target_info)
}
