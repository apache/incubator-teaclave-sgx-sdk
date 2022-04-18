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

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn global_ctor(_attribute: TokenStream, function: TokenStream) -> TokenStream {
    let function: syn::ItemFn = syn::parse_macro_input!(function);
    validate_item("global_ctor", &function);

    let syn::ItemFn {
        attrs,
        block,
        vis,
        sig:
            syn::Signature {
                ident,
                unsafety,
                constness,
                abi,
                ..
            },
        ..
    } = function;

    let ctor_ident = syn::parse_str::<syn::Ident>(format!("{}___rust_ctor___ctor", ident).as_ref())
        .expect("Unable to create identifier");

    // let output = quote!(
    //     #[cfg(not(target_os = "linux"))]
    //     compile_error!("#[global_ctor] is not supported on the current target");

    //     #(#attrs)*
    //     #vis #unsafety extern #abi #constness fn #ident() #block

    //     #[used]
    //     #[allow(non_upper_case_globals)]
    //     #[link_section = ".init_array"]
    //     pub static #ctor_ident
    //     :
    //     unsafe extern "C" fn() =
    //     {
    //         unsafe extern "C" fn #ctor_ident() { #ident() };
    //         #ctor_ident
    //     };
    // );
    let ctor_func_ident =
        syn::parse_str::<syn::Ident>(format!("{}___rust_ctor___ctor__func", ident).as_ref())
            .expect("Unable to create identifier");
    let output = quote!(
        #[cfg(not(target_os = "linux"))]
        compile_error!("#[global_ctor] is not supported on the current target");

        #(#attrs)*
        #vis #unsafety extern #abi #constness fn #ident() #block

        #[used]
        #[allow(non_upper_case_globals)]
        #[link_section = ".init_array"]
        #[no_mangle]
        pub static #ctor_ident: unsafe extern "C" fn() = #ctor_func_ident;
        #[no_mangle]
        pub unsafe extern "C" fn #ctor_func_ident() { #ident() }
    );
    output.into()
}

#[proc_macro_attribute]
pub fn global_dtor(_attribute: TokenStream, function: TokenStream) -> TokenStream {
    let function: syn::ItemFn = syn::parse_macro_input!(function);
    validate_item("global_dtor", &function);

    let syn::ItemFn {
        attrs,
        block,
        vis,
        sig:
            syn::Signature {
                ident,
                unsafety,
                constness,
                abi,
                ..
            },
        ..
    } = function;

    let dtor_ident = syn::parse_str::<syn::Ident>(format!("{}___rust_dtor___dtor", ident).as_ref())
        .expect("Unable to create identifier");

    let output = quote!(
        #[cfg(not(target_os = "linux"))]
        compile_error!("#[global_dtor] is not supported on the current target");

        #(#attrs)*
        #vis #unsafety extern #abi #constness fn #ident() #block

        #[used]
        #[allow(non_upper_case_globals)]
        #[link_section = ".fini_array"]
        pub static #dtor_ident
        :
        unsafe extern "C" fn() =
        {
            unsafe extern "C" fn #dtor_ident() { #ident() };
            #dtor_ident
        };
    );
    output.into()
}

fn validate_item(typ: &str, item: &syn::ItemFn) {
    let syn::ItemFn { vis, sig, .. } = item;

    // Ensure that visibility modifier is not present
    match vis {
        syn::Visibility::Inherited => {}
        _ => panic!("#[{}] methods must not have visibility modifiers", typ),
    }

    // No parameters allowed
    assert!(
        sig.inputs.is_empty(),
        "#[{}] methods may not have parameters",
        typ
    );

    // No return type allowed
    match sig.output {
        syn::ReturnType::Default => {}
        _ => panic!("#[{}] methods must not have return types", typ),
    }
}
