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
use align::{AlignArgs, AlignStruct};
use syn::{parse_macro_input, DeriveInput};

mod align;
mod layout;

#[proc_macro_attribute]
pub fn sgx_align(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AlignArgs);
    let layout = args.get_layout().expect("Attribute args error");
    let input = parse_macro_input!(input as DeriveInput);
    let mut alignstruct = AlignStruct::new(layout, input);
    let expanded = alignstruct.build();
    proc_macro::TokenStream::from(expanded)
}
