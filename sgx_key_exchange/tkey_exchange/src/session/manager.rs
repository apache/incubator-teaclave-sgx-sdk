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

use alloc::boxed::Box;
use alloc::collections::LinkedList;
use alloc::sync::Arc;
use core::mem;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicU32, Ordering};
use sgx_crypto::ecc::{EcPrivateKey, EcPublicKey, EcShareKey};
use sgx_sync::{LazyLock, SpinMutex, SpinRwLock};
use sgx_types::error::SgxResult;
use sgx_types::types::{AlignKey128bit, Ec256SharedKey, PsSecPropDesc, QuoteNonce, TargetInfo};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum State {
    Inited,
    GaGened,
    Msg2Proced,
}

impl Default for State {
    #[inline]
    fn default() -> State {
        State::Inited
    }
}

type DeriveKeyFn = dyn Fn(
        &Ec256SharedKey,
        u16,
    ) -> SgxResult<(
        AlignKey128bit,
        AlignKey128bit,
        AlignKey128bit,
        AlignKey128bit,
    )> + Sync
    + Send
    + 'static;

#[derive(Default)]
pub struct Context {
    pub sp_pub_key: EcPublicKey,
    pub pub_key_a: EcPublicKey,
    pub pub_key_b: EcPublicKey,
    pub priv_key: EcPrivateKey,
    pub ps_sec_prop: PsSecPropDesc,
    pub smk_key: AlignKey128bit,
    pub sk_key: AlignKey128bit,
    pub mk_key: AlignKey128bit,
    pub vk_key: AlignKey128bit,
    pub quote_nonce: QuoteNonce,
    pub qe_target: TargetInfo,
    pub state: State,
}

impl Context {
    pub fn new() -> Context {
        Self::default()
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
    pub derive_key: Option<Box<DeriveKeyFn>>,
}

impl Default for Session {
    fn default() -> Session {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Session {
        Session {
            context: SpinMutex::new(Context::new()),
            derive_key: None,
        }
    }

    pub fn new_with_context(context: Context) -> Session {
        Session {
            context: SpinMutex::new(context),
            derive_key: None,
        }
    }

    pub fn set_derive_key(mut self, derive_key: Option<Box<DeriveKeyFn>>) -> Session {
        self.derive_key = derive_key;
        self
    }
}

pub static SESSION_MAGAGER: LazyLock<SpinRwLock<SessionManager>> =
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
