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

use sgx_libc::int32_t;
use sgx_trts::veh::{
    exception_handle, rsgx_register_exception_handler, rsgx_unregister_exception_handler,
};
use sgx_types::SE_WORDSIZE;
use sgx_types::{sgx_exception_info_t, sgx_exception_vector_t};
use sgx_types::{EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_CONTINUE_SEARCH};
use std::collections::LinkedList;
use std::convert::From;
use std::num::NonZeroU64;
use std::ops::Drop;
use std::sync::{Arc, Once, SgxRwLock, SgxMutex, PoisonError, ONCE_INIT};
use std::u64;

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ContinueType {
    Search,
    Execution,
}

impl From<ContinueType> for i32 {
    fn from(continue_type: ContinueType) -> i32 {
        match continue_type {
            ContinueType::Search => EXCEPTION_CONTINUE_SEARCH,
            ContinueType::Execution => EXCEPTION_CONTINUE_EXECUTION,
        }
    }
}

#[allow(unknown_lints, bare_trait_objects)]
type ExceptionHandler = dyn Fn(&mut sgx_exception_info_t) -> ContinueType + Send + Sync;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HandlerId(NonZeroU64);

impl HandlerId {
    fn new() -> HandlerId {
        #[cold]
        fn exhausted() -> ! {
            panic!("failed to generate unique Handler ID: bitspace exhausted")
        }
        static COUNTER: SgxMutex<u64> = SgxMutex::new(0);

        let mut counter = COUNTER.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(id) = counter.checked_add(1) else {
            drop(counter);
            exhausted();
        };

        *counter = id;
        drop(counter);
        HandlerId(NonZeroU64::new(id).unwrap())
    }
}

struct HandlerNode {
    id: HandlerId,
    handler: Arc<ExceptionHandler>,
}

impl HandlerNode {
    // add code here
    pub fn new(id: HandlerId, handler: Arc<ExceptionHandler>) -> Self {
        HandlerNode { id, handler }
    }
    pub fn get_handler_id(&self) -> HandlerId {
        self.id
    }
}

struct ExceptionManager {
    exception_handler: SgxRwLock<LinkedList<HandlerNode>>,
    native_handle: Option<exception_handle>,
}

static mut GLOBAL_DATA: Option<GlobalData> = None;

#[allow(deprecated)]
static GLOBAL_INIT: Once = ONCE_INIT;
struct GlobalData {
    manager: ExceptionManager,
}

impl GlobalData {
    fn get() -> &'static Self {
        unsafe { GLOBAL_DATA.as_ref().unwrap() }
    }
    fn ensure() -> &'static Self {
        GLOBAL_INIT.call_once(|| unsafe {
            GLOBAL_DATA = Some(GlobalData {
                manager: ExceptionManager::new(),
            });
        });
        Self::get()
    }
}

extern "C" fn native_exception_handler(info: *mut sgx_exception_info_t) -> int32_t {
    if let Ok(handlers) = GlobalData::get().manager.exception_handler.read() {
        let info = unsafe { info.as_mut().unwrap() };
        for h in handlers.iter() {
            match (h.handler)(info) {
                ContinueType::Search => {}
                ContinueType::Execution => return EXCEPTION_CONTINUE_EXECUTION,
            }
        }
    }
    unsafe { panic_handler(info).into() }
}

unsafe extern "C" fn panic_handler(info: *mut sgx_exception_info_t) -> ContinueType {
    let exception_info = info.as_mut().unwrap();
    let mut rsp = exception_info.cpu_context.rsp;
    if rsp & 0xF == 0 {
        rsp -= SE_WORDSIZE as u64;
        exception_info.cpu_context.rsp = rsp;
        let addr = rsp as *mut u64;
        *addr = exception_info.cpu_context.rip;
    } else {
    }

    exception_info.cpu_context.rdi = exception_info.exception_vector as u32 as u64;
    exception_info.cpu_context.rsi = exception_info.cpu_context.rip;
    exception_info.cpu_context.rip = exception_panic as usize as u64;

    ContinueType::Execution
}

#[no_mangle]
#[inline(never)]
unsafe fn exception_panic(vector: sgx_exception_vector_t, rip: usize) {
    let exception = match vector {
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_DE => "#DE",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_DB => "#DB",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_BP => "#BP",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_BR => "#BR",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_UD => "#UD",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_GP => "#GP",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_PF => "#PF",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_MF => "#MF",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_AC => "#AC",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_XM => "#XM",
        sgx_exception_vector_t::SGX_EXCEPTION_VECTOR_CP => "#CP",
    };
    panic!("enclave exception: {}, at rip: 0x{:x}", exception, rip);
}

impl ExceptionManager {
    // add code here
    pub fn new() -> Self {
        ExceptionManager {
            exception_handler: SgxRwLock::new(LinkedList::new()),
            native_handle: rsgx_register_exception_handler(0, native_exception_handler),
        }
    }
}

#[allow(dead_code)]
impl Drop for ExceptionManager {
    fn drop(&mut self) {
        if let Some(handler) = self.native_handle {
            rsgx_unregister_exception_handler(handler);
            self.native_handle = None;
            if let Ok(ref mut handlers) = self.exception_handler.write() {
                handlers.clear();
            }
        }
    }
}

fn register_exception_impl<F>(first: bool, handler: F) -> Option<HandlerId>
where
    F: Fn(&mut sgx_exception_info_t) -> ContinueType + Sync + Send + 'static,
{
    let globals = GlobalData::ensure();

    if let Ok(ref mut handlers) = globals.manager.exception_handler.write() {
        let handler_id = HandlerId::new();
        if first {
            handlers.push_front(HandlerNode::new(handler_id, Arc::from(handler)));
        } else {
            handlers.push_back(HandlerNode::new(handler_id, Arc::from(handler)));
        }
        Some(handler_id)
    } else {
        None
    }
}

///
/// The register_exception function allows developers to register an exception handler,
/// and specify whether to prepend (when is_first is true) or append the handler to the handler chain.
///
/// # Description
///
/// The Rust SGX SDK supports the registration of custom exception handler
/// functions. You can write your own code to handle a limited set of hardware
/// exceptions.
///
/// # Note
///
/// 1. OCALLs are not allowed in the exception handler.
/// 2. Custom exception handing only saves general purpose registers in sgx_exception_info_t. You should be careful when touching other registers inthe exception handlers.
///
pub fn register_exception<F>(is_first: bool, handler: F) -> Option<HandlerId>
where
    F: Fn(&mut sgx_exception_info_t) -> ContinueType + Sync + Send + 'static,
{
    register_exception_impl(is_first, handler)
}

///
/// The register function allows developers to register an exception handler.
///
/// # Description
///
/// The Rust SGX SDK supports the registration of custom exception handler
/// functions. You can write your own code to handle a limited set of hardware
/// exceptions.
///
/// # Note
///
/// 1. OCALLs are not allowed in the exception handler.
/// 2. Custom exception handing only saves general purpose registers in sgx_exception_info_t. You should be careful when touching other registers inthe exception handlers.
///
pub fn register<F>(handler: F) -> Option<HandlerId>
where
    F: Fn(&mut sgx_exception_info_t) -> ContinueType + Sync + Send + 'static,
{
    register_exception_impl(true, handler)
}

pub fn unregister(id: HandlerId) -> bool {
    let globals = GlobalData::ensure();
    if let Ok(ref mut handlers) = globals.manager.exception_handler.write() {
        handlers
            .drain_filter(|n| n.get_handler_id() == id)
            .next()
            .is_some()
    } else {
        false
    }
}
