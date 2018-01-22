// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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

//! Support for `#[derive(Rand)]`
//!
//! **Note**
//!
//! TcsPolicy must be Bound.
//!
//! # Examples
//!
//! ```
//! extern crate sgx_rand;
//! #[macro_use]
//! extern crate sgx_rand_derive;
//!
//! #[derive(Rand, Debug)]
//! struct MyStruct {
//!     a: i32,
//!     b: u32,
//! }
//!
//! fn main() {
//!     println!("{:?}", sgx_rand::random::<MyStruct>());
//! }
//! ```

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(Rand)]
pub fn rand_derive(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_rand_derive(&ast);
    gen.parse().unwrap()
}

fn impl_rand_derive(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let rand = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref body)) => {
            let fields = body
                .iter()
                .filter_map(|field| field.ident.as_ref())
                .map(|ident| quote! { #ident: __rng.gen() })
                .collect::<Vec<_>>();

            quote! { #name { #(#fields,)* } }
        },
        syn::Body::Struct(syn::VariantData::Tuple(ref body)) => {
            let fields = (0..body.len())
                .map(|_| quote! { __rng.gen() })
                .collect::<Vec<_>>();

            quote! { #name (#(#fields),*) }
        },
        syn::Body::Struct(syn::VariantData::Unit) => {
            quote! { #name }
        },
        syn::Body::Enum(ref body) => {
            if body.is_empty() {
                panic!("`Rand` cannot be derived for enums with no variants");
            }

            let len = body.len();
            let mut arms = body
                .iter()
                .map(|variant| {
                    let ident = &variant.ident;
                    match variant.data {
                        syn::VariantData::Struct(ref body) => {
                            let fields = body
                                .iter()
                                .filter_map(|field| field.ident.as_ref())
                                .map(|ident| quote! { #ident: __rng.gen() })
                                .collect::<Vec<_>>();
                            quote! { #name::#ident { #(#fields,)* } }
                        },
                        syn::VariantData::Tuple(ref body) => {
                            let fields = (0..body.len())
                                .map(|_| quote! { __rng.gen() })
                                .collect::<Vec<_>>();

                            quote! { #name::#ident (#(#fields),*) }
                        },
                        syn::VariantData::Unit => quote! { #name::#ident }
                    }
                });

            match len {
                1 => quote! { #(#arms)* },
                2 => {
                    let (a, b) = (arms.next(), arms.next());
                    quote! { if __rng.gen() { #a } else { #b } }
                },
                _ => {
                    let mut variants = arms
                        .enumerate()
                        .map(|(index, arm)| quote! { #index => #arm })
                        .collect::<Vec<_>>();
                    variants.push(quote! { _ => unreachable!() });
                    quote! { match __rng.gen_range(0, #len) { #(#variants,)* } }
                },
            }
        }
    };

    quote! {
        impl #impl_generics ::sgx_rand::Rand for #name #ty_generics #where_clause {
            #[inline]
            fn rand<__R: ::sgx_rand::Rng>(__rng: &mut __R) -> Self {
                #rand
            }
        }
    }
}
