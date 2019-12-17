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

use syn;
use crate::Ctxt;

#[derive(Debug)]
pub struct Container<'a> {
    pub ident: syn::Ident,
    pub body: Body<'a>,
    pub generics: &'a syn::Generics,
}

#[derive(Debug)]
pub enum Body<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

#[derive(Debug)]
pub struct Variant<'a> {
    pub ident: syn::Ident,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug)]
pub struct Field<'a> {
    pub ident: Option<syn::Ident>,
    pub ty: &'a syn::Ty,
}

#[derive(Copy, Clone, Debug)]
pub enum Style {
    Struct,
    Tuple,
    Newtype,
    Unit,
}

impl<'a> Container<'a> {
    pub fn from_ast(_cx: &Ctxt, item: &'a syn::DeriveInput) -> Container<'a> {
        let body = match item.body {
            syn::Body::Enum(ref variants) => Body::Enum(enum_from_ast(variants)),
            syn::Body::Struct(ref variant_data) => {
                let (style, fields) = struct_from_ast(variant_data);
                Body::Struct(style, fields)
            }
        };

        let item = Container {
            ident: item.ident.clone(),
            body: body,
            generics: &item.generics,
        };
        item
    }
}

fn enum_from_ast<'a>(variants: &'a [syn::Variant]) -> Vec<Variant<'a>> {
    variants
        .iter()
        .map(
            |variant| {
                let (style, fields) = struct_from_ast(&variant.data);
                Variant {
                    ident: variant.ident.clone(),
                    style: style,
                    fields: fields,
                }
            }
        )
        .collect()
}

fn struct_from_ast<'a>( data: &'a syn::VariantData) -> (Style, Vec<Field<'a>>) {
    match *data {
        syn::VariantData::Struct(ref fields) => (Style::Struct, fields_from_ast(fields)),
        syn::VariantData::Tuple(ref fields) if fields.len() == 1 => {
            (Style::Newtype, fields_from_ast(fields))
        }
        syn::VariantData::Tuple(ref fields) => (Style::Tuple, fields_from_ast( fields)),
        syn::VariantData::Unit => (Style::Unit, Vec::new()),
    }
}

fn fields_from_ast<'a>(fields: &'a [syn::Field]) -> Vec<Field<'a>> {
    fields
        .iter()
        .map(
            |field| {
                Field {
                    ident: field.ident.clone(),
                    ty: &field.ty,
                }
            },
        )
        .collect()
}
