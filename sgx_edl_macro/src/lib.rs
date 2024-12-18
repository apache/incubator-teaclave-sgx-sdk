use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Visibility};

#[proc_macro_attribute]
pub fn ecall(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    let fn_lfs: Vec<_> = input_fn.sig.generics.lifetimes().map(|lf| lf).collect();
    let fn_name = &input_fn.sig.ident;
    let fn_args = input_fn.sig.inputs.iter();
    let (arg_names, arg_tys): (Vec<_>, Vec<_>) = input_fn
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => unimplemented!(),
            syn::FnArg::Typed(pat_type) => (pat_type.pat.as_ref(), pat_type.ty.as_ref()),
        })
        .unzip();

    input_fn.vis = Visibility::Inherited;

    quote! {
        pub mod #fn_name {
            use super::*;

            #[derive(Default)]
            struct _PhantomMarker<#(#fn_lfs),*> {
                _phantom: core::marker::PhantomData<(#(&#fn_lfs ()),*)>,
            }

            impl<#(#fn_lfs),*> sgx_edl::ecall::Ecall for _PhantomMarker<#(#fn_lfs),*> {
                const IDX: usize = idx::#fn_name;
                type Args = (#(#arg_tys), *);

                fn call(&self, args: Self::Args) -> Self::Args {
                    let (#(#arg_names), *) = args;

                    #fn_name(#(#arg_names), *);
                    todo!()
                }
            }

            pub fn ecall<#(#fn_lfs),*>(eid: usize, #(#fn_args),*) {
                sgx_edl::ecall::EcallWrapper::wrapper_u(&_PhantomMarker::default(), eid, todo!(), (#(#arg_names), *));
            }

            pub fn entry(args: *const u8) {
                sgx_edl::ecall::EcallWrapper::wrapper_t(&_PhantomMarker::default(), args);
            }

            #input_fn
        }
    }
    .into()
}
