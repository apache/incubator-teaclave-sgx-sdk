use quote::quote;
use syn::{Data, Fields, DataStruct, DeriveInput, parse_macro_input};
use proc_macro::TokenStream;

#[proc_macro_derive(SgxTypeDebug)]
pub fn derive_sgx_type_debug(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    let expanded = match input.data {
        Data::Struct(DataStruct{ref fields,..}) => {
            if let Fields::Named(ref fields_name) = fields {
                let get_selfs: Vec<_> = fields_name.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    match &field.ty {
                        syn::Type::Array(_) => quote! { add_debug_array_field(&mut s, stringify!(#field_name), &self.#field_name[..]); },
                        _ =>
                        quote! { add_debug_reg_field(&mut s, stringify!(#field_name), &self.#field_name); },
                    }
                }).collect();

                let implemented_debug = quote! {
                    impl core::fmt::Debug for #struct_name {
                        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                            let mut s = f.debug_struct(stringify!(#struct_name));
                            unsafe { #(#get_selfs)* }
                            s.finish()
                        }
                    }
                };
                implemented_debug
            
            } else {
                panic!("SgxTypeDebug does not supports types other than Named Fields")
            }
        }
        _ => panic!("SgxTypeDebug only support Struct")
    };
    expanded.into()
}
