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

use crate::layout::{self, AlignReq};
use proc_macro2::{Ident, Punct, Span};
use quote::quote;
use std::alloc::Layout;
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    parse_quote, punctuated::Punctuated, DeriveInput, Error, Expr, Fields, Lit, LitInt, Meta,
    Token, Type,
};

#[allow(dead_code)]
struct KeyValue {
    pub ident: Ident,
    pub punct: Punct,
    pub value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let punct = input.parse::<Punct>()?;
        let value = input.parse::<Expr>()?;
        Ok(KeyValue {
            ident,
            punct,
            value,
        })
    }
}

pub struct AlignArgs {
    vars: Vec<KeyValue>,
}

impl Parse for AlignArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let keyvalue = Punctuated::<KeyValue, Token![,]>::parse_terminated(input)?;
        Ok(AlignArgs {
            vars: keyvalue.into_iter().collect(),
        })
    }
}

impl AlignArgs {
    pub fn get_layout(&self) -> Result<Layout> {
        let align_iter = self
            .vars
            .iter()
            .find(|v| v.ident.to_string() == "align");
        let align: usize = if let Some(align_value) = align_iter {
            self.parse_align(&align_value.value)?
        } else {
            return Err(Error::new(
                Span::call_site(),
                "Invalid type of align attribute parsing",
            ));
        };

        let size_iter = self
            .vars
            .iter()
            .find(|v| v.ident.to_string() == "size");
        let size: usize = if let Some(size_value) = size_iter {
            self.parse_size(&size_value.value)?
        } else {
            return Err(Error::new(
                Span::call_site(),
                "Invalid type of size attribute parsing",
            ));
        };
        Layout::from_size_align(size, align)
            .map_err(|_e| Error::new(Span::call_site(), "Layout illegal"))
    }

    fn parse_align(&self, align_expr: &Expr) -> Result<usize> {
        if let Expr::Lit(expr) = align_expr {
            if let Lit::Int(ref lit) = expr.lit {
                let align = lit.base10_parse::<u32>()?;
                Ok(align as usize)
            } else {
                Err(Error::new(
                    Span::call_site(),
                    "Invalid align attribute parsing",
                ))
            }
        } else {
            Err(Error::new(
                Span::call_site(),
                "Invalid align attribute parsing",
            ))
        }
    }

    fn parse_size(&self, size_expr: &Expr) -> Result<usize> {
        if let Expr::Lit(expr) = size_expr {
            if let Lit::Int(ref lit) = expr.lit {
                let size = lit.base10_parse::<u32>()?;
                Ok(size as usize)
            } else {
                Err(Error::new(
                    Span::call_site(),
                    "Invalid size attribute parsing",
                ))
            }
        } else {
            Err(Error::new(
                Span::call_site(),
                "Invalid size attribute parsing",
            ))
        }
    }
}

pub struct AlignStruct {
    input: DeriveInput,
    layout: Layout,
    align_layout: Layout,
}

struct FiledExt<'a> {
    ty: &'a Type,
    name: Option<&'a Ident>,
}

impl<'a> FiledExt<'a> {
    pub fn new(ty: &'a Type, name: Option<&'a Ident>) -> Self {
        FiledExt { ty, name }
    }

    pub fn as_origin_named_field(&self) -> proc_macro2::TokenStream {
        let ty = &self.ty;
        let name = self.name.as_ref().unwrap();
        quote! {
            #name: #ty,
        }
    }
}

impl AlignStruct {
    pub fn new(layout: Layout, input: DeriveInput) -> Self {
        AlignStruct {
            input,
            layout,
            align_layout: unsafe { Layout::from_size_align_unchecked(0, 0) },
        }
    }

    pub fn build(&mut self) -> proc_macro2::TokenStream {
        if !(self.is_contains_specified_attr("C")
            && !self.is_contains_specified_attr("align")
            && !self.is_contains_specified_attr("packed"))
        {
            panic!("Structure attribute must require repr C ");
        }

        let pad_item = self.get_align_pad_field();
        let attrs = self.get_attrs();
        let vis = &self.input.vis;
        let name = &self.input.ident;
        let generics = &self.input.generics;
        let fields = &self.get_origin_fields();
        let align_attr = &self.generate_align_attr();
        quote! {
            #attrs
            #align_attr
            #vis struct #name#generics {
            #pad_item,
            #fields
            }
        }
    }

    fn is_contains_specified_attr(&self, atrr: &str) -> bool {
        let mut erxcept_attr = false;
        for attr in &self.input.attrs {
            let ident = attr.path.get_ident();
            let meta = attr.parse_meta();
            if ident.map_or(false, |v| v.to_string() == "repr") && meta.is_ok() {
                if let Ok(Meta::List(ref m)) = meta {
                    if m.nested.len() > 0 {
                        erxcept_attr = m
                            .nested
                            .iter()
                            .filter(|x| {
                                let mut find = false;
                                if let syn::NestedMeta::Meta(ref s) = x {
                                    if let syn::Meta::Path(p) = s {
                                        find =
                                            p.get_ident().map_or(false, |v| v.to_string() == atrr);
                                    }
                                }
                                find
                            })
                            .next()
                            .is_some();
                    }
                }
            }
        }
        erxcept_attr
    }

    fn get_attrs(&self) -> proc_macro2::TokenStream {
        let attrs = &self.input.attrs;
        quote! {
            #(#attrs)*
        }
    }

    fn generate_align_attr(&self) -> proc_macro2::TokenStream {
        let align = self.align_layout.align();
        let litint = LitInt::new(&format!("{}", align).to_string(), Span::call_site());
        quote! {
           #[repr(align(#litint))]
        }
    }

    fn get_origin_fields(&self) -> proc_macro2::TokenStream {
        if let syn::Data::Struct(ref data) = self.input.data {
            if let Fields::Named(ref fields) = data.fields {
                let fields: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| FiledExt::new(&f.ty, f.ident.as_ref()))
                    .collect();
                let item = fields.iter().map(|x| x.as_origin_named_field());
                quote! {
                    #(#item)*
                }
            } else {
                panic!("Structure fields must have names");
            }
        } else {
            panic!("Only supports struct type");
        }
    }

    fn get_align_pad_field(&mut self) -> proc_macro2::TokenStream {
        let align_req: &[AlignReq] = &[AlignReq {
            offset: 0,
            len: self.layout.size(),
        }];
        let align_layout =
            layout::pad_align_to(self.layout, align_req).expect("Align layout illegal");
        self.align_layout = align_layout;
        let pad = align_layout.size() - align_layout.align() - self.layout.size();
        let ty: syn::Type = parse_quote!([u8; #pad]);
        let name = Ident::new("no_secret_allowed_in_here", Span::call_site());
        quote! {
            #name: #ty
        }
    }
}
