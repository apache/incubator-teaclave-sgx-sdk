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

//!
//! The mod implements the function of DeSerializable.
//!

use syn::{self, Ident};

use quote::Tokens;

use crate::internals::ast::{Body, Container, Field, Style, Variant};
use crate::internals::{Ctxt};
use crate::param::Parameters;
use crate::fragment::{Fragment, Stmts};

pub fn expand_derive_deserialize(input: &syn::DeriveInput) -> Result<Tokens, String> {
    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, input);
    ctxt.check()?;

    let ident = &cont.ident;
    let params = Parameters::new(&cont);
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();

    let body = Stmts(deserialize_body(&cont));

    let impl_block = quote! {
            impl #impl_generics ::sgx_serialize::DeSerializable for #ident #ty_generics #where_clause {
                fn decode<__D: ::sgx_serialize::Decoder>(__arg_0: &mut __D)
                -> ::std::result::Result<#ident #ty_generics , __D::Error> {
                    #body
                }
            }
        };

    Ok(impl_block)
}

fn deserialize_body(cont: &Container) -> Fragment {

    match cont.body {
            Body::Enum(ref variants) => {
                deserialize_enum(cont, variants)
            }
            Body::Struct(Style::Struct, ref fields) => {
                if fields.iter().any(|field| field.ident.is_none()) {
                    panic!("struct has unnamed fields");
                }
                deserialize_struct(cont, fields)
            }
            Body::Struct(Style::Tuple, ref fields) => {
                if fields.iter().any(|field| field.ident.is_some()) {
                    panic!("tuple struct has named fields");
                }
                deserialize_tuple_struct(cont, fields)
            }
            Body::Struct(Style::Newtype, ref fields) => {
                if fields.iter().any(|field| field.ident.is_some()) {
                    panic!("newtype struct has named fields");
                }
                deserialize_newtype_struct(cont)
            }
            Body::Struct(Style::Unit, _) => {
                deserialize_unit_struct(cont)
            }
    }
}

fn fromat_ident(name: &syn::Ident) -> syn::Ident {
    let mut name_str = String::from("\"");
    name_str.push_str(name.clone().as_ref());
    name_str.push_str("\"");
    syn::Ident::from(name_str)
}

fn deserialize_enum(
    cont: &Container,
    variants: &[Variant]
) -> Fragment {
    assert!(variants.len() as u64 <= u32::MAX as u64);

    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    let arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(
            |(variant_index, variant)| {
                deserialize_variant(cont, variant, variant_index)
            },
        )
        .collect();

    let variants_slice: Vec<_> = variants
        .iter()
        .map(
            |variant| {
                let variant_ident = variant.ident.clone();
                fromat_ident(&variant_ident)
            },
        )
        .collect();

    quote_expr! {
        __arg_0.read_enum(#name_arg, |_d | ->_ {
            _d.read_enum_variant( &[#(#variants_slice, )*] , |_d, i | ->_ {
                ::std::result::Result::Ok(
                    match i {
                        #(#arms,)*
                        _ => panic!("internal error: entered unreachable code"),
                })})
        })
    }
}

fn deserialize_variant(
    cont: &Container,
    variant: &Variant,
    variant_index: usize
) -> Tokens {
    let this: syn::Ident = cont.ident.clone().into();
    let variant_ident = variant.ident.clone();

    let case = match variant.style {
        Style::Unit => {
            quote! {
                #variant_index => {
                    #this::#variant_ident
                }
            }
        }
        Style::Newtype => {
            quote! {
                #variant_index => {
                    #this::#variant_ident(match _d.read_enum_variant_arg(0usize, ::sgx_serialize::DeSerializable::decode) {
                            ::std::result::Result::Ok(__try_var) => __try_var,
                            ::std::result::Result::Err(__try_var) => {
                                return ::std::result::Result::Err(__try_var)
                            },
                    })
                }
            }
        }
        Style::Tuple => {
            let fileds_case: Vec<_> = variant.fields
                .iter()
                .enumerate()
                .map(
                    |(i, _)| -> _ {
                        quote! {
                            match _d.read_enum_variant_arg(#i, ::sgx_serialize::DeSerializable::decode) {
                                ::std::result::Result::Ok(__try_var) => __try_var,
                                ::std::result::Result::Err(__try_var) =>
                                return ::std::result::Result::Err(__try_var),
                            }
                        }
                    }
                )
                .collect();

            quote! {
                #variant_index => {
                    #this::#variant_ident(
                        #(#fileds_case,)*
                    )
                }
            }
        }
        Style::Struct => {
            let fields = variant
                .fields
                .iter()
                .map(
                    |f| {
                        f.ident
                            .clone()
                            .expect("struct variant has unnamed fields")
                    },
                );
            let fileds_case: Vec<_> = variant.fields
                .iter()
                .enumerate()
                .map(
                    |(i, _)| -> _ {
                        quote! {
                            match _d.read_enum_variant_arg(#i, ::sgx_serialize::DeSerializable::decode) {
                                ::std::result::Result::Ok(__try_var) => __try_var,
                                ::std::result::Result::Err(__try_var) =>
                                return ::std::result::Result::Err(__try_var),
                            }
                        }
                    }
                )
                .collect();
            quote! {
                #variant_index => {
                    #this::#variant_ident{
                        #(#fields: #fileds_case,)*
                    }
                }
            }
        }
    };

    quote! {
        #case
    }
}


fn deserialize_struct(
    cont: &Container,
    fields: &[Field]
)  -> Fragment {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    let self_args_cnt = fields.iter().count();

    let serialize_stmts = deserialize_tuple_struct_visitor(
        fields,
        true
    );

    quote_block! {
        __arg_0.read_struct(
            #name_arg,
            #self_args_cnt,
            |_d| -> _ {
                    ::std::result::Result::Ok(#name{
                                #(#serialize_stmts, )*
                    })
                })
    }
}

fn deserialize_tuple_struct(
    cont: &Container,
    fields: &[Field],
) -> Fragment {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    let self_args_cnt = fields.iter().count();

    let deserialize_stmts = deserialize_tuple_struct_visitor(
        fields,
        false
    );

    quote_block! {
        __arg_0.read_struct(
            #name_arg,
            #self_args_cnt,
            |_d| -> _ {
                ::std::result::Result::Ok(#name(
                #(#deserialize_stmts,)*
                ))
            }
        )
    }
}

fn deserialize_tuple_struct_visitor(
    fields: &[Field],
    is_struct: bool,
) -> Vec<Tokens> {
    fields
        .iter()
        .enumerate()
        .map(
            |(i, field)| {
                let (name, field_expr) = if is_struct {
                    match field.ident {
                        Some(ref ident) => {
                            let id = Ident::new(format!("\"{}\"", ident.clone().as_ref()));
                            (ident.clone().into(), quote!(#id))
                        }
                        None => {
                            panic!("struct filed must have name!")
                        }
                    }
                } else {
                    let id = Ident::new(format!("\"_field{}\"", i));
                    (None, quote!(#id))
                };

                if is_struct {
                    quote! {
                        #name:
                            match _d.read_struct_field(#field_expr,
                                #i,
                                ::sgx_serialize::DeSerializable::decode) {
                            ::std::result::Result::Ok(__try_var) => __try_var,
                            ::std::result::Result::Err(__try_var) => return ::std::result::Result::Err(__try_var),
                        }
                    }
                }
                else {
                    quote! {
                        match _d.read_struct_field(#field_expr,
                                #i,
                                ::sgx_serialize::DeSerializable::decode) {
                            ::std::result::Result::Ok(__try_var) => __try_var,
                            ::std::result::Result::Err(__try_var) => return ::std::result::Result::Err(__try_var),
                        }
                    }
                }
            },
        )
        .collect()
}

fn deserialize_newtype_struct(
    cont: &Container
) -> Fragment {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    quote_expr! {
        __arg_0.read_struct(#name_arg, 1usize,
                            |_d| -> _ {
                                ::std::result::Result::Ok( #name(
                                        match _d.read_struct_field(
                                                "_field0",
                                                0usize,
                                                ::sgx_serialize::DeSerializable::decode){
                                            ::std::result::Result::Ok(__try_var) => __try_var,
                                            ::std::result::Result::Err(__try_var) => {
                                                return ::std::result::Result::Err(__try_var)
                                            }
                                        }))
                                })
    }
}

fn deserialize_unit_struct(cont: &Container) -> Fragment {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    quote_expr! {
        __arg_0.read_struct(#name_arg, 0usize,
                                |_d| -> _ {
                                    ::std::result::Result::Ok(#name)
                                })
    }
}