use std::str::FromStr;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, visit::Visit, visit_mut::VisitMut,
    ForeignItemFn, GenericArgument, Ident, ItemFn, Lifetime, PathArguments, Token, Type, TypePath,
    Visibility,
};

#[proc_macro_attribute]
pub fn ecall(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);
    let mut raw_fn = f.clone();
    let sig = &mut f.sig;

    sig.inputs.iter_mut().for_each(|arg| {
        ReplaceLifetimes.visit_fn_arg_mut(arg);
    });

    let mut ex = GenericExtractor {
        tys: Vec::new(),
        //lifetimes: Vec::new(),
    };

    let fn_name = &sig.ident;
    let extern_name = externed_name(fn_name);
    let (arg_names, arg_tys): (Vec<_>, Vec<_>) = sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => unimplemented!(),
            syn::FnArg::Typed(pat_type) => {
                ex.visit_type(&pat_type.ty);
                (pat_type.pat.as_ref(), pat_type.ty.as_ref())
            }
        })
        .unzip();

    raw_fn.vis = Visibility::Inherited;

    let tys = ex.tys;

    quote! {
        pub mod #fn_name {
            use super::*;

            struct _PhantomMarker<'a> {
                _phantom: &'a ()
            }

            impl<'a> Default for _PhantomMarker<'a> {
                fn default() -> Self {
                    Self {
                        _phantom: &()
                    }
                }
            }

            impl<'a> sgx_new_edl::Ecall<(#(#tys), *)> for _PhantomMarker<'a> {
                type Args = (#(#arg_tys), *);

                fn call(&self, args: Self::Args) -> sgx_types::error::SgxStatus {
                    let (#(#arg_names), *) = args;
                    #fn_name(#(#arg_names), *)
                }
            }

            #[no_mangle]
            pub extern fn #extern_name(args: *const u8) -> sgx_types::error::SgxStatus {
                sgx_new_edl::EcallWrapper::wrapper_t(&_PhantomMarker::default(), args)
            }

            #raw_fn
        }
    }
    .into()
}
