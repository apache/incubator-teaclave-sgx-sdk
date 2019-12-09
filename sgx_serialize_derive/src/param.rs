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

use syn::{self, Ident};
use crate::internals::ast::Container;
use crate::bound;

#[derive(Debug)]
pub struct Parameters {
    /// Variable holding the value being serialized. Either `self` for local
    /// types or `__self` for remote types.
    pub self_var: Ident,

    /// Path to the type the impl is for. Either a single `Ident` for local
    /// types or `some::remote::Ident` for remote types. Does not include
    /// generic parameters.
    pub this: syn::Path,

    /// Generics including any explicit and inferred bounds for the impl.
    pub generics: syn::Generics,
}

impl Parameters {
    pub fn new(cont: &Container) -> Self {
        let self_var = Ident::new("self");

        let this = cont.ident.clone().into();

        let generics = build_generics(cont);

        Parameters {
            self_var: self_var,
            this: this,
            generics: generics,
        }
    }
}

// All the generics in the input, plus a bound `T: Serialize` for each generic
// field type that will be serialized by us.
fn build_generics(cont: &Container) -> syn::Generics {
    bound::without_defaults(cont.generics)
}
