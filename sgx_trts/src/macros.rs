// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! Global constructor support

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
            {$func};
        }
    }
}
