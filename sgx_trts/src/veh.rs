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

use sgx_types::*;

pub type exception_handle = *const c_void;

///
/// rsgx_register_exception_handler registers an exception handler.
///
/// rsgx_register_exception_handler allows developers to register an exception handler, and specify whether to prepend
/// (when is_first_handler is equal to 1) or append the handler to the handler chain.
///
/// # Description
///
/// The Intel(R) SGX SDK supports the registration of custom exception handler functions. You can write your own code to
/// handle a limited set of hardware exceptions. For example, a CPUID instruction inside an enclave will effectively result
/// in a #UD fault (Invalid Opcode Exception). ISV enclave code can have an exception handler to prevent the enclave from
/// being trapped into an exception condition.
///
/// Calling rsgx_register_exception_handler allows you to register an exception handler, and specify whether to prepend
/// (when is_first_handler is nonzero) or append the handler to the handler chain.
///
/// After calling rsgx_register_exception_handler to prepend an exception handler, a subsequent call to this function may
/// add another exception handler at the beginning of the handler chain. Therefore the order in which exception handlers
/// are called does not only depend on the value of the is_first_handler parameter. It depends on the order
/// in which exception handlers are registered.
///
/// # Parameters
///
/// **is_first_handler**
///
/// Specify the order in which the handler should be called. If the parameter is nonzero, the handler is the first handler
/// to be called. If the parameter is zero, the handler is the last handler to be called.
///
/// **exception_handler**
///
/// The exception handler to be called
///
/// # Requirements
///
/// Library: libsgx_trts.a
///
/// # Return value
///
/// **Some(exception_handle)**
///
/// Indicates the exception handler is registered successfully. The return value is an open handle to the custom exception handler.
///
/// **None**
///
/// The exception handler was not registered.
///
pub fn rsgx_register_exception_handler(
    is_first_handler: u32,
    exception_handler: sgx_exception_handler_t,
) -> Option<exception_handle> {
    let handle = unsafe { sgx_register_exception_handler(is_first_handler, exception_handler) };
    if handle.is_null() {
        None
    } else {
        Some(handle)
    }
}

///
/// rsgx_unregister_exception_handler is used to unregister a custom exception handler.
///
/// # Description
///
/// The Intel(R) SGX SDK supports the registration of custom exception handler functions. An enclave developer
/// can write their own code to handle a limited set of hardware exceptions.
///
/// Calling rsgx_unregister_exception_handler allows developers to unregister an exception handler that was registered earlier.
///
/// # Parameters
///
/// **handle**
///
/// A handle to the custom exception handler previously registered using the rsgx_register_exception_handler function.
///
/// # Requirements
///
/// Library: libsgx_trts.a
///
/// # Return value
///
/// **true**
///
/// The custom exception handler is unregistered successfully.
///
/// **false**
///
/// The exception handler was not unregistered (not a valid pointer, handler not found).
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn rsgx_unregister_exception_handler(handle: exception_handle) -> bool {
    let ret = unsafe { sgx_unregister_exception_handler(handle) };
    ret != 0
}
