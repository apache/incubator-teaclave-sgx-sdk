// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
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

//!
//! The mod implements the function of DeSerializable.
//!

use syn::{self, Ident};

use quote::Tokens;

use internals::ast::{Body, Container, Field, Style, Variant};
use internals::{Ctxt};
use param::Parameters;
use fragment::{Fragment, Stmts};

pub fn expand_derive_deserialize(input: &syn::DeriveInput) -> Result<Tokens, String> {
    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, input);
    try!(ctxt.check());

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
    assert!(variants.len() as u64 <= u32::max_value() as u64);

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