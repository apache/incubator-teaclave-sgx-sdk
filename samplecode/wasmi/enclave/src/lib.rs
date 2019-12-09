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

#![crate_name = "wasmienclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate lazy_static;

use std::prelude::v1::*;
use std::sync::SgxMutex;
use std::ptr;

extern crate wasmi;
extern crate sgxwasm;

use sgxwasm::{SpecDriver, boundary_value_to_runtime_value, result_covert};

use sgx_types::*;
use std::slice;

use wasmi::{ModuleInstance, ImportsBuilder, RuntimeValue, Error as InterpreterError, Module};

extern crate serde;
extern crate serde_json;

lazy_static!{
    static ref SPECDRIVER: SgxMutex<SpecDriver> = SgxMutex::new(SpecDriver::new());
}

#[no_mangle]
pub extern "C"
fn sgxwasm_init() -> sgx_status_t {
    let mut sd = SPECDRIVER.lock().unwrap();
    *sd = SpecDriver::new();
    sgx_status_t::SGX_SUCCESS
}

fn wasm_invoke(module : Option<String>, field : String, args : Vec<RuntimeValue>)
              -> Result<Option<RuntimeValue>, InterpreterError> {
    let mut program = SPECDRIVER.lock().unwrap();
    let module = program.module_or_last(module.as_ref().map(|x| x.as_ref()))
                        .expect(&format!("Expected program to have loaded module {:?}", module));
    module.invoke_export(&field, &args, program.spec_module())
}

fn wasm_get(module : Option<String>, field : String)
            -> Result<Option<RuntimeValue>, InterpreterError> {
    let program = SPECDRIVER.lock().unwrap();
    let module = match module {
        None => {
                 program
                 .module_or_last(None)
                 .expect(&format!("Expected program to have loaded module {:?}",
                        "None"
                 ))
        },
        Some(str) => {
                 program
                 .module_or_last(Some(&str))
                 .expect(&format!("Expected program to have loaded module {:?}",
                         str
                 ))
        }
    };

    let global = module.export_by_name(&field)
                       .ok_or_else(|| {
                           InterpreterError::Global(format!("Expected to have export with name {}", field))
                       })?
                       .as_global()
                       .cloned()
                       .ok_or_else(|| {
                           InterpreterError::Global(format!("Expected export {} to be a global", field))
                       })?;
     Ok(Some(global.get()))
}

fn try_load_module(wasm: &[u8]) -> Result<Module, InterpreterError> {
    wasmi::Module::from_buffer(wasm).map_err(|e| InterpreterError::Instantiation(format!("Module::from_buffer error {:?}", e)))
}

fn wasm_try_load(wasm: Vec<u8>) -> Result<(), InterpreterError> {
    let ref mut spec_driver = SPECDRIVER.lock().unwrap();
    let module = try_load_module(&wasm[..])?;
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())?;
    instance
        .run_start(spec_driver.spec_module())
        .map_err(|trap| InterpreterError::Instantiation(format!("ModuleInstance::run_start error on {:?}", trap)))?;
    Ok(())
}

fn wasm_load_module(name: Option<String>, module: Vec<u8>)
                    -> Result<(), InterpreterError> {
    let ref mut spec_driver = SPECDRIVER.lock().unwrap();
    let module = try_load_module(&module[..])?;
    let instance = ModuleInstance::new(&module, &**spec_driver)
        .map_err(|e| InterpreterError::Instantiation(format!("ModuleInstance::new error on {:?}", e)))?
        .run_start(spec_driver.spec_module())
        .map_err(|trap| InterpreterError::Instantiation(format!("ModuleInstance::run_start error on {:?}", trap)))?;

    spec_driver.add_module(name, instance.clone());

    Ok(())
}

fn wasm_register(name: &Option<String>, as_name: String)
                    -> Result<(), InterpreterError> {
    let ref mut spec_driver = SPECDRIVER.lock().unwrap();
    spec_driver.register(name, as_name)
}

#[no_mangle]
pub extern "C"
fn sgxwasm_run_action(req_bin : *const u8, req_length: usize,
                      result_bin : *mut u8, result_max_len: usize) -> sgx_status_t {

    let req_slice = unsafe { slice::from_raw_parts(req_bin, req_length) };
    let action_req: sgxwasm::SgxWasmAction = serde_json::from_slice(req_slice).unwrap();

    let response;
    let return_status;

    match action_req {
        sgxwasm::SgxWasmAction::Invoke{module,field,args}=> {
            let args = args.into_iter()
                           .map(|x| boundary_value_to_runtime_value(x))
                           .collect::<Vec<RuntimeValue>>();
            let r = wasm_invoke(module, field, args);
            let r = result_covert(r);
            response = serde_json::to_string(&r).unwrap();
            match r {
                Ok(_) => {
                    return_status = sgx_status_t::SGX_SUCCESS;
                },
                Err(_) => {
                    return_status = sgx_status_t::SGX_ERROR_WASM_INTERPRETER_ERROR;
               }
            }
        },
        sgxwasm::SgxWasmAction::Get{module,field} => {
            let r = wasm_get(module, field);
            let r = result_covert(r);
            response = serde_json::to_string(&r).unwrap();
            match r {
                Ok(_v) => {
                    return_status = sgx_status_t::SGX_SUCCESS;
                },
                Err(_x) => {
                    return_status = sgx_status_t::SGX_ERROR_WASM_INTERPRETER_ERROR;
                }
            }
        },
        sgxwasm::SgxWasmAction::LoadModule{name,module} => {
            let r = wasm_load_module(name.clone(), module);
            response = serde_json::to_string(&r).unwrap();
            match r {
                Ok(_) => {
                    return_status = sgx_status_t::SGX_SUCCESS;
                },
                Err(_x) => {
                    return_status = sgx_status_t::SGX_ERROR_WASM_LOAD_MODULE_ERROR;
                }
            }
        },
        sgxwasm::SgxWasmAction::TryLoad{module} => {
            let r = wasm_try_load(module);
            response = serde_json::to_string(&r).unwrap();
            match r {
                Ok(()) => {
                    return_status = sgx_status_t::SGX_SUCCESS;
                },
                Err(_x) => {
                    return_status = sgx_status_t::SGX_ERROR_WASM_TRY_LOAD_ERROR;
                }
            }
        },
        sgxwasm::SgxWasmAction::Register{name, as_name} => {
            let r = wasm_register(&name, as_name.clone());
            response = serde_json::to_string(&r).unwrap();
            match r {
                Ok(()) => {
                    return_status = sgx_status_t::SGX_SUCCESS;
                },
                Err(_x) => {
                    return_status = sgx_status_t::SGX_ERROR_WASM_REGISTER_ERROR;
                }
            }
        }
    }

    //println!("len = {}, Response = {:?}", response.len(), response);

    if response.len() < result_max_len {
        unsafe {
            ptr::copy_nonoverlapping(response.as_ptr(),
                                     result_bin,
                                     response.len());
        }
        return return_status;
    }
    else{
        //println!("Result len = {} > buf size = {}", response.len(), result_max_len);
        return sgx_status_t::SGX_ERROR_WASM_BUFFER_TOO_SHORT;
    }
}

