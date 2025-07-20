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

use alloc::collections::LinkedList;
use alloc::sync::Arc;
use core::mem;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicU32, Ordering};
use sgx_crypto::ecc::{EcPrivateKey, EcPublicKey, EcShareKey};
use sgx_sync::{LazyLock, SpinMutex, SpinRwLock};
use sgx_types::types::{AlignKey128bit, EnclaveIdentity, QlQvResult, QuoteNonce, TargetInfo};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Initiator,
    Responder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitiatorState {
    Inited,
    GaGened,
    Msg2Proced,
    Established,
}

impl Default for InitiatorState {
    #[inline]
    fn default() -> Self {
        Self::Inited
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponderState {
    Inited,
    Msg1Proced,
    Msg2Gened,
    Established,
}

impl Default for ResponderState {
    #[inline]
    fn default() -> Self {
        Self::Inited
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Initiator(InitiatorState),
    Responder(ResponderState),
}

impl State {
    #[inline]
    pub fn new(role: Role) -> Self {
        match role {
            Role::Initiator => From::from(InitiatorState::default()),
            Role::Responder => From::from(ResponderState::default()),
        }
    }

    #[inline]
    pub fn check_initiator_state(&self, state: InitiatorState) -> bool {
        self.eq(&state.into())
    }

    #[inline]
    pub fn check_responder_state(&self, state: ResponderState) -> bool {
        self.eq(&state.into())
    }
}

impl From<InitiatorState> for State {
    #[inline]
    fn from(state: InitiatorState) -> Self {
        Self::Initiator(state)
    }
}

impl From<ResponderState> for State {
    #[inline]
    fn from(state: ResponderState) -> Self {
        Self::Responder(state)
    }
}

pub struct Context {
    pub role: Role,
    pub state: State,
    pub pub_key_a: EcPublicKey,
    pub pub_key_b: EcPublicKey,
    pub priv_key: EcPrivateKey,
    pub smk_key: AlignKey128bit,
    pub sk_key: AlignKey128bit,
    pub mk_key: AlignKey128bit,
    pub vk_key: AlignKey128bit,
    pub sp_pub_key: Option<EcPublicKey>,
    pub quote_nonce: QuoteNonce,
    pub qe_target: TargetInfo,
    pub qv_result: Option<QlQvResult>,
    pub enclave_identity: Option<EnclaveIdentity>,
}

impl Context {
    pub fn new(role: Role) -> Context {
        Context {
            role,
            state: State::new(role),
            pub_key_a: EcPublicKey::default(),
            pub_key_b: EcPublicKey::default(),
            priv_key: EcPrivateKey::default(),
            smk_key: AlignKey128bit::default(),
            sk_key: AlignKey128bit::default(),
            mk_key: AlignKey128bit::default(),
            vk_key: AlignKey128bit::default(),
            sp_pub_key: None,
            quote_nonce: QuoteNonce::default(),
            qe_target: TargetInfo::default(),
            qv_result: None,
            enclave_identity: None,
        }
    }

    #[inline]
    fn clear(&mut self) {
        unsafe { ptr::write_bytes(self as *mut _ as *mut u8, 0, mem::size_of::<Context>()) }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.clear()
    }
}

pub struct Session {
    pub context: SpinMutex<Context>,
}

impl Session {
    pub fn new(role: Role) -> Session {
        Session {
            context: SpinMutex::new(Context::new(role)),
        }
    }

    pub fn new_with_context(context: Context) -> Session {
        Session {
            context: SpinMutex::new(context),
        }
    }
}

pub static INITIATOR_SESSION_MAGAGER: LazyLock<SpinRwLock<SessionManager>> =
    LazyLock::new(|| SpinRwLock::new(SessionManager::new()));

pub static RESPONDER_SESSION_MAGAGER: LazyLock<SpinRwLock<SessionManager>> =
    LazyLock::new(|| SpinRwLock::new(SessionManager::new()));

struct Node {
    sid: u32,
    session: Arc<Session>,
}

pub struct SessionManager {
    seed: AtomicU32,
    list: LinkedList<Node>,
}

impl Default for SessionManager {
    fn default() -> SessionManager {
        SessionManager::new()
    }
}

impl SessionManager {
    pub const fn new() -> SessionManager {
        SessionManager {
            seed: AtomicU32::new(1),
            list: LinkedList::new(),
        }
    }

    pub fn find(&self, sid: u32) -> Option<Arc<Session>> {
        self.list
            .iter()
            .find(|&node| node.sid == sid)
            .map(|node| node.session.clone())
    }

    pub fn push(&mut self, session: Session) -> u32 {
        let sid = self.seed.fetch_add(1, Ordering::SeqCst);
        let session = Arc::new(session);
        self.list.push_back(Node { sid, session });
        sid
    }

    pub fn remove(&mut self, sid: u32) -> Option<Arc<Session>> {
        self.list
            .extract_if(|node| node.sid == sid)
            .next()
            .map(|node| node.session)
    }
}

pub(crate) struct DropKey<'a> {
    key: &'a mut AlignKey128bit,
}

impl<'a> DropKey<'a> {
    #[inline]
    pub(crate) fn new(key: &'a mut AlignKey128bit) -> Self {
        DropKey { key }
    }
}

impl Drop for DropKey<'_> {
    #[inline]
    fn drop(&mut self) {
        self.key.key.fill(0);
    }
}

impl Deref for DropKey<'_> {
    type Target = AlignKey128bit;

    fn deref(&self) -> &Self::Target {
        self.key
    }
}

pub(crate) struct DropShareKey<'a> {
    key: &'a mut EcShareKey,
}

impl<'a> DropShareKey<'a> {
    #[inline]
    pub(crate) fn new(key: &'a mut EcShareKey) -> Self {
        DropShareKey { key }
    }
}

impl Drop for DropShareKey<'_> {
    #[inline]
    fn drop(&mut self) {
        self.key.clear();
    }
}

impl Deref for DropShareKey<'_> {
    type Target = EcShareKey;

    fn deref(&self) -> &Self::Target {
        self.key
    }
}

pub(crate) struct DropPrivateKey<'a> {
    key: &'a mut EcPrivateKey,
}

impl<'a> DropPrivateKey<'a> {
    #[inline]
    pub(crate) fn new(key: &'a mut EcPrivateKey) -> Self {
        DropPrivateKey { key }
    }
}

impl Drop for DropPrivateKey<'_> {
    #[inline]
    fn drop(&mut self) {
        self.key.clear();
    }
}

impl Deref for DropPrivateKey<'_> {
    type Target = EcPrivateKey;

    fn deref(&self) -> &Self::Target {
        self.key
    }
}
