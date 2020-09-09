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
//! The mod implements the function of Serializable.
//!

use syn::{self, Ident};

use quote::Tokens;

use crate::internals::ast::{Body, Container, Field, Style, Variant};
use crate::internals::{Ctxt};
use crate::param::Parameters;
use crate::fragment::{Fragment, Stmts};

pub fn expand_derive_serialize(input: &syn::DeriveInput) -> Result<Tokens, String> {
    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, input);
    ctxt.check()?;

    let ident = &cont.ident;
    let params = Parameters::new(&cont);
    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();

    let body = Stmts(serialize_body(&cont, &params));

    let impl_block = quote! {
            impl #impl_generics ::sgx_serialize::Serializable for #ident #ty_generics #where_clause {
                fn encode<__S: ::sgx_serialize::Encoder>(&self, __arg_0: &mut __S)
                -> ::std::result::Result<(), __S::Error> {
                    #body
                }
            }
        };

    Ok(impl_block)
}

fn serialize_body(cont: &Container, params: &Parameters) -> Fragment {

    match cont.body {
            Body::Enum(ref variants) => {
                serialize_enum(cont, params, variants)
            }
            Body::Struct(Style::Struct, ref fields) => {
                if fields.iter().any(|field| field.ident.is_none()) {
                    panic!("struct has unnamed fields");
                }
                serialize_struct(cont, fields)
            }
            Body::Struct(Style::Tuple, ref fields) => {
                if fields.iter().any(|field| field.ident.is_some()) {
                    panic!("tuple struct has named fields");
                }
                serialize_tuple_struct(cont, fields)
            }
            Body::Struct(Style::Newtype, ref fields) => {
                if fields.iter().any(|field| field.ident.is_some()) {
                    panic!("newtype struct has named fields");
                }
                serialize_newtype_struct(cont)
            }
            Body::Struct(Style::Unit, _) => {
                serialize_unit_struct(cont)
            }
    }
}

fn fromat_ident(name: &syn::Ident) -> syn::Ident {
    let mut name_str = String::from("\"");
    name_str.push_str(name.clone().as_ref());
    name_str.push_str("\"");
    syn::Ident::from(name_str)
}

fn serialize_enum(
    cont: &Container,
    params: &Parameters,
    variants: &[Variant]
) -> Fragment {
    assert!(variants.len() as u64 <= u32::MAX as u64);

    let self_var = &params.self_var;

    let arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(
            |(variant_index, variant)| {
                serialize_variant(cont, variant, variant_index)
            },
        )
        .collect();

    quote_expr! {
        match *#self_var {
            #(#arms)*
        }
    }
}

fn serialize_variant(
    cont: &Container,
    variant: &Variant,
    variant_index: usize
) -> Tokens {

    let case = serialize_variant_case(cont, variant);

    let body = serialize_variant_body(cont, variant, variant_index);

    quote! {
        #case => #body
    }
}

fn serialize_variant_body(
    cont: &Container,
    variant: &Variant,
    variant_index: usize
) -> Tokens {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);
    let variant_ident = variant.ident.clone();
    let variant_ident_arg = fromat_ident(&variant_ident);

    let body = match variant.style {
        Style::Unit => {
            quote! { {
                    let _e = __arg_0;
                    _e.emit_enum(#name_arg,
                        |_e| -> _ {
                            _e.emit_enum_variant(#variant_ident_arg,
                                                #variant_index,
                                                0usize,
                                                |_e| -> _ {
                                                        return ::std::result::Result::Ok(())
                                                })
                            })
                }
            }
        }
        Style::Newtype => {
            quote! { {
                let _e = __arg_0;
                _e.emit_enum(#name_arg,
                    |_e| -> _ {
                        _e.emit_enum_variant(#variant_ident_arg,
                                            #variant_index,
                                            1usize,
                                            |_e| -> _ {
                                                return _e.emit_enum_variant_arg(
                                                        0usize,
                                                        |_e| -> _ {
                                                            ::sgx_serialize::Serializable::encode(&(*__self_0), _e)
                                                        })
                                            })
                    })
                }
            }
        }
        Style::Tuple => {
            let variant_fields_len = variant.fields.len();
            let variant_args: Vec<Tokens> =
                variant.fields
                .iter()
                .enumerate()
                .map(
                    |(i, _)| {
                        let id = Ident::new(format!("__self_{}", i));
                        if variant_fields_len == i+1 {
                            quote! {
                                return _e.emit_enum_variant_arg(#i,
                                                    |_e| -> _ {
                                                        ::sgx_serialize::Serializable::encode(&(*#id),
                                                                                        _e)})
                            }
                        } else {
                            quote! {
                                match _e.emit_enum_variant_arg(#i,
                                                    |_e| -> _ {
                                                        ::sgx_serialize::Serializable::encode(&(*#id),
                                                                                    _e)})
                                    {
                                    ::std::result::Result::Ok(__try_var) => __try_var,
                                    ::std::result::Result::Err(__try_var) => return ::std::result::Result::Err(__try_var),
                                }
                            }
                        }
                })
                .collect();
            quote! { {
                    let _e = __arg_0;
                    _e.emit_enum(#name_arg,
                        |_e| -> _ {
                            _e.emit_enum_variant(#variant_ident_arg,
                                                #variant_index, #variant_fields_len,
                                                |_e| -> _ {
                                                        #(#variant_args)*
                                                    })
                    })
                }
            }
        }
        Style::Struct => {
            serialize_variant_body_visitor(cont, variant, variant_index)
        }
    };

    quote! { #body }
}

fn serialize_variant_body_visitor(
    cont: &Container,
    variant: &Variant,
    variant_index: usize
) -> Tokens {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    let variant_ident = variant.ident.clone();
    let variant_ident_arg = fromat_ident(&variant_ident);
    let variant_fields_len = variant.fields.len();

    let variant_args: Vec<Tokens>  =
        variant.fields
        .iter()
        .enumerate()
        .map(
            |(i, _)| {
                let id = Ident::new(format!("__self_{}", i));
                if variant_fields_len == i+1 {
                    quote! {
                        return _e.emit_enum_variant_arg(#i,
                                            |_e| -> _ {
                                                ::sgx_serialize::Serializable::encode(&(*#id),
                                                                                _e)})
                    }
                } else {
                    quote! {
                        match _e.emit_enum_variant_arg(#i,
                                            |_e| -> _ {
                                                ::sgx_serialize::Serializable::encode(&(*#id),
                                                                            _e)})
                            {
                            ::std::result::Result::Ok(__try_var) => __try_var,
                            ::std::result::Result::Err(__try_var) => return ::std::result::Result::Err(__try_var),
                        }
                    }
                }
        })
        .collect();
    quote! { {
            let _e = __arg_0;
            _e.emit_enum(#name_arg,
                |_e| -> _ {
                    _e.emit_enum_variant(#variant_ident_arg,
                                        #variant_index, #variant_fields_len,
                                        |_e| -> _ {
                                                #(#variant_args)*
                                            })
            })
        }
    }
}

fn serialize_variant_case(
    cont: &Container,
    variant: &Variant
) -> Tokens {
    let this: syn::Ident = cont.ident.clone().into();
    let variant_ident = variant.ident.clone();

    let case = match variant.style {
        Style::Unit => {
            quote! {
                #this::#variant_ident
            }
        }
        Style::Newtype => {
            quote! {
                #this::#variant_ident(ref __self_0)
            }
        }
        Style::Tuple => {
            let field_names =
                (0..variant.fields.len()).map(|i| Ident::new(format!("__self_{}", i)));
            quote! {
                #this::#variant_ident(#(ref #field_names),*)
            }
        }
        Style::Struct => {
            let fields = variant.fields.iter().map(
                    |f| {
                        f.ident.clone().expect("struct variant has unnamed fields")
                    },
                );
            let selfs = variant.fields.iter().enumerate().map(
                    |(i, _)| {
                        Ident::new(format!("__self_{}", i))
                    },
                );
            quote! {
                #this::#variant_ident { #(#fields: ref #selfs),* }
            }
        }
    };
    quote! { #case }
}

fn serialize_struct(
    cont: &Container,
    fields: &[Field]
)  -> Fragment {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    let self_args:Vec<Tokens> = fields
                        .iter()
                        .enumerate()
                        .map(|(i, filed)| {
                            let arg = Ident::new(format!("ref __self_0_{}", i));
                            let arg1 = match filed.ident {
                                Some(ref ident) => {
                                    let id: Ident = ident.clone().into();
                                    quote!(#id)
                                }
                                None => {
                                    panic!("struct filed must have name!")
                                }
                            };
                            quote!(#arg1: #arg)
                        })
                        .collect();

    let self_args_cnt = fields.iter().count();

    let serialize_stmts = serialize_tuple_struct_visitor(
        fields,
        true
    );

    quote_block! {
        match *self {
                    #name{#(#self_args,)*} =>
                    __arg_0.emit_struct(#name_arg, #self_args_cnt,
                                        |_e| -> _
                                            {
                                                #(#serialize_stmts)*
                                            })
        }
    }
}

fn serialize_tuple_struct(
    cont: &Container,
    fields: &[Field],
) -> Fragment {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    let self_args:Vec<Tokens> = fields
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let arg = Ident::new(format!("ref __self_0_{}", i));
                            quote!(#arg)
                        })
                        .collect();

    let self_args_cnt = fields.iter().count();

    let serialize_stmts = serialize_tuple_struct_visitor(
        fields,
        false
    );

    quote_block! {
        match *self {
                    #name(#(#self_args,)*) =>
                    __arg_0.emit_struct(#name_arg, #self_args_cnt,
                                        |_e| -> _
                                            {
                                                #(#serialize_stmts)*
                                            })
        }
    }
}

fn serialize_tuple_struct_visitor(
    fields: &[Field],
    is_struct: bool,
) -> Vec<Tokens> {
    fields
        .iter()
        .enumerate()
        .map(
            |(i, field)| {
                let field_expr = if is_struct {
                    match field.ident {
                        Some(ref ident) => {
                            let id = Ident::new(format!("\"{}\"", ident.clone().as_ref()));
                            quote!(#id)
                        }
                        None => {
                            panic!("struct filed must have name!")
                        }
                    }
                } else {
                    let id = Ident::new(format!("\"_field{}\"", i));
                    quote!(#id)
                };

                let arg = Ident::new(format!("__self_0_{}", i));
                let field_arg = quote!(#arg);

                if fields.iter().count() == i+1 {
                    quote! {
                        return _e.emit_struct_field(#field_expr,
                                                    #i,
                                                    |_e| -> _ {
                                                        ::sgx_serialize::Serializable::encode(&(*#field_arg), _e)
                                                    });
                    }
                } else {
                    quote! {
                        match _e.emit_struct_field(#field_expr,
                                                    #i,
                                                    |_e| -> _ {
                                                        ::sgx_serialize::Serializable::encode(&(*#field_arg),
                                                                                        _e)
                                                    })
                            {
                            ::std::result::Result::Ok(__try_var)
                            => __try_var,
                            ::std::result::Result::Err(__try_var)
                            =>
                            return ::std::result::Result::Err(__try_var),
                        }
                    }
                }
            },
        )
        .collect()
}

fn serialize_newtype_struct(
    cont: &Container
) -> Fragment {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    quote_expr! {
        match *self {
        #name(ref __self_0_0) =>
        __arg_0.emit_struct(#name_arg, 1usize,
                            |_e| -> _
                                {
                                    return _e.emit_struct_field("_field0",
                                                                0usize,
                                                                |_e| -> _ {
                                                                    ::sgx_serialize::Serializable::encode(&(*__self_0_0), _e)
                                                                });
                                }),
        }
    }
}

fn serialize_unit_struct(cont: &Container) -> Fragment {
    let name: syn::Ident = cont.ident.clone().into();
    let name_arg = fromat_ident(&name);

    quote_expr! {
        match *self {
            #name =>
            __arg_0.emit_struct(#name_arg, 0usize,
                                |_e| -> _ {
                                    return ::std::result::Result::Ok(())
                                }),
        }
    }
}
