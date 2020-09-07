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

//! Global constructor/destructor support

/// global_ctors_object is the base macro of implementing constructors.
///
/// global_ctors_object registers functions to the `.init_array` section
/// of the generated enclave binary. On the first ecall of enclave's execution,
/// SGX would execute these registered functions and thus initialize data
/// structures.
#[macro_export]
macro_rules! global_ctors_object {
    ($var_name:ident, $func_name:ident = $func:block) => {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                #[link_section = ".init_array"]
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else if #[cfg(target_os = "windows")]  {
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else if #[cfg(target_os = "macos")]  {
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else {

            }
        }
        #[no_mangle]
        pub fn $func_name() {
            {
                $func
            };
        }
    };
}

#[macro_export]
macro_rules! global_dtors_object {
    ($var_name:ident, $func_name:ident = $func:block) => {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                #[link_section = ".fini_array"]
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else if #[cfg(target_os = "windows")]  {
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else if #[cfg(target_os = "macos")]  {
                #[no_mangle]
                pub static $var_name: fn() = $func_name;
            } else {

            }
        }
        #[no_mangle]
        pub fn $func_name() {
            {
                $func
            };
        }
    };
}
