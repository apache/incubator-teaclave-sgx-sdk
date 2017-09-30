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

use syn;
use Ctxt;

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