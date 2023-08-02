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
use core::mem::{self, ManuallyDrop};
use core::ptr;
use sgx_types::*;

pub type aex_handle = *const sgx_aex_mitigation_node_t;

///
/// rsgx_set_ssa_aexnotify allows developers to enable the AEX-Notify feature
/// upon a piece of enclave code.
///
/// # Description
///
/// You can enable or disable AEX-Notify in the enclave code using this function.
/// To enable AEX-Notify for critical code that you want to mitigate for single-step
/// attacks, call this function. The following execution will be executed with
/// AEX-Notify enabled until you call this function to disable it.
///
pub fn rsgx_set_ssa_aexnotify(is_enable: bool) -> SgxError {
    let mut flags = 0_i32;
    if is_enable {
        flags = 1;
    }
    let ret = unsafe { sgx_set_ssa_aexnotify(flags) };

    match ret {
        sgx_status_t::SGX_SUCCESS => Ok(()),
        _ => Err(ret),
    }
}

///
/// rsgx_register_aex_handler allows developers to register an AEX- Notify handler.
///
/// # Description
///
/// The Rust SDK allows you to register custom AEX-Notify functions. You can write your
/// own code to provide an AEX-Notify handler that detects single-step attacks. For instance,
/// you can provide a handler that counts the Async Exit. If the count is abnormal which
/// means single-step attacks occur, you can take proper actions.
///
pub fn rsgx_register_aex_handler(
    handler: sgx_aex_mitigation_fn_t,
    args: usize,
) -> SgxResult<aex_handle> {
    let mut node: Box<sgx_aex_mitigation_node_t> = Box::new(sgx_aex_mitigation_node_t {
        handler,
        args: args as *const c_void,
        next: ptr::null_mut(),
    });
    let node_ptr = &mut *node as *mut sgx_aex_mitigation_node_t;
    let ret = unsafe { sgx_register_aex_handler(node_ptr, handler, args as *const _) };

    match ret {
        sgx_status_t::SGX_SUCCESS => {
            mem::forget(node);
            Ok(node_ptr)
        }
        _ => Err(ret),
    }
}

///
/// rsgx_unregister_aex_handler is used to unregister an AEX-Notify handler.
///
/// # Description
///
/// The Rust SDK allows you to register custom AEX-Notify functions. You can write your own code to
/// provide an AEX-Notify handler that detects single-step attacks.
///
pub fn rsgx_unregister_aex_handler(handle: aex_handle) -> SgxError {
    unsafe {
        let node: ManuallyDrop<Box<sgx_aex_mitigation_node_t>> =
            ManuallyDrop::new(Box::from_raw(handle as *mut sgx_aex_mitigation_node_t));
        let ret = sgx_unregister_aex_handler(node.handler);
        match ret {
            sgx_status_t::SGX_SUCCESS => {
                let _ = ManuallyDrop::into_inner(node);
                Ok(())
            }
            _ => Err(ret),
        }
    }
}
