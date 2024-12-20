use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, visit::Visit, visit_mut::VisitMut, GenericArgument, ItemFn, Lifetime,
    PathArguments, Type, TypePath, Visibility,
};

struct ReplaceLifetimes;

impl VisitMut for ReplaceLifetimes {
    fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
        *i = syn::parse_quote!('a);
    }
}

struct GenericExtractor {
    tys: Vec<Type>,
    //lifetimes: Vec<Lifetime>,
}

impl Visit<'_> for GenericExtractor {
    fn visit_type_path(&mut self, i: &TypePath) {
        for segment in &i.path.segments {
            if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                for arg in &args.args {
                    match arg {
                        //GenericArgument::Lifetime(lifetime) => {
                        //    self.lifetimes.push(lifetime.clone())
                        //}
                        GenericArgument::Type(ty) => self.tys.push(ty.clone()),
                        _ => {}
                    }
                }
            }
        }
    }

    fn visit_type_reference(&mut self, i: &syn::TypeReference) {
        syn::visit::visit_type_reference(self, i);
    }

    fn visit_type(&mut self, i: &Type) {
        match i {
            Type::Path(type_path) => self.visit_type_path(type_path),
            _ => syn::visit::visit_type(self, i),
        }
    }
}

#[proc_macro_attribute]
pub fn ecall(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    let mut raw_fn = input_fn.clone();

    input_fn.sig.inputs.iter_mut().for_each(|arg| {
        ReplaceLifetimes.visit_fn_arg_mut(arg);
    });

    let mut ex = GenericExtractor {
        tys: Vec::new(),
        //lifetimes: Vec::new(),
    };

    let fn_name = &input_fn.sig.ident;
    let fn_args = input_fn.sig.inputs.iter();
    let (arg_names, arg_tys): (Vec<_>, Vec<_>) = input_fn
        .sig
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

            impl<'a> sgx_new_edl::ecall::Ecall<(#(#tys), *)> for _PhantomMarker<'a> {
                const IDX: usize = idx::#fn_name;
                type Args = (#(#arg_tys), *);

                fn call(&self, args: Self::Args) {
                    let (#(#arg_names), *) = args;

                    #fn_name(#(#arg_names), *);
                }
            }

            pub fn ecall<'a>(eid: usize, o_tab: &[sgx_new_edl::ocall::OTabEntry], #(#fn_args),*) {
                sgx_new_edl::ecall::EcallWrapper::wrapper_u(&_PhantomMarker::default(), eid, o_tab, (#(#arg_names), *));
            }

            pub fn entry(args: *const u8) {
                sgx_new_edl::ecall::EcallWrapper::wrapper_t(&_PhantomMarker::default(), args);
            }

            #raw_fn
        }
    }
    .into()
}
