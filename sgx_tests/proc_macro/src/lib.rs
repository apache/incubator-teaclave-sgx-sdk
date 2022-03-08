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

extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, ItemFn};

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test_case(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);
    validate_item("test_case", &f);

    let f_ident = &f.sig.ident;
    let test_ident =
        syn::parse_str::<syn::Ident>(format!("{}___rust_sgx_test_case__test", f_ident).as_ref())
            .expect("Unable to create identifier");
    let ctor_ident =
        syn::parse_str::<syn::Ident>(format!("{}___rust_ctor___ctor", test_ident).as_ref())
            .expect("Unable to create identifier");

    let q = quote!(
        #[cfg(not(target_os = "linux"))]
        compile_error!("#[test_case] is not supported on the current target");

        #f

        #[used]
        #[allow(non_upper_case_globals)]
        #[link_section = ".init_array"]
        pub static #ctor_ident
        :
        unsafe extern "C" fn() =
        {
            unsafe extern "C" fn #test_ident() {
                sgx_test_utils::submit(
                    sgx_test_utils::TestCase::new(concat!(module_path!(), "::", stringify!(#f_ident)), #f_ident)
                );
            }
            #test_ident
        };
    );
    q.into()
}

#[proc_macro_attribute]
pub fn bench_case(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);
    validate_item("bench_case", &f);

    let f_ident = &f.sig.ident;
    let bench_ident =
        syn::parse_str::<syn::Ident>(format!("{}___rust_sgx_bench_case__bench", f_ident).as_ref())
            .expect("Unable to create identifier");
    let ctor_ident =
        syn::parse_str::<syn::Ident>(format!("{}___rust_ctor___ctor", bench_ident).as_ref())
            .expect("Unable to create identifier");

    let q = quote!(
        #[cfg(not(target_os = "linux"))]
        compile_error!("#[bench_case] is not supported on the current target");

        #f

        #[used]
        #[allow(non_upper_case_globals)]
        #[link_section = ".init_array"]
        pub static #ctor_ident
        :
        unsafe extern "C" fn() =
        {
            unsafe extern "C" fn #bench_ident() {
                sgx_test_utils::submit(
                    sgx_test_utils::BenchCase::new(concat!(module_path!(), "::", stringify!(#f_ident)), #f_ident)
                );
            }
            #bench_ident
        };
    );
    q.into()
}

fn validate_item(typ: &str, item: &syn::ItemFn) {
    let syn::ItemFn { vis: _, sig, .. } = item;

    // No return type allowed
    match sig.output {
        syn::ReturnType::Default => {}
        _ => panic!("#[{}] methods must not have return types", typ),
    }
}
