//! Different protocol formats.

use attr;
use syn;

pub type Discriminator = u32;

/// Represents a format.
pub trait Format : Clone {
    /// From a string.
    fn from_str(s: &str) -> Result<Self, ()>;
}

/// The enum protocol format.
#[derive(Clone, Debug, PartialEq)]
pub enum Enum {
    /// The enum is transmitted by using the 1-based index of the enum variant.
    IntegerDiscriminator,
    /// The enum is transmitted by using the name of the variant.
    StringDiscriminator,
}

impl Enum {
    /// Gets the discriminator of the enum.
    pub fn discriminator(&self, e: &syn::DataEnum,
                         variant: &syn::Variant) -> ::proc_macro2::TokenStream {
        match *self {
            Enum::IntegerDiscriminator => {
                let variant_index = e.variants.iter().position(|v| v.ident == variant.ident).expect("variant not a part of enum");

                let discriminator = match variant.discriminant {
                    Some((_, syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(ref n), .. }))) => n.value() as Discriminator,
                    Some(_) => panic!("unknown discriminator"),
                    // Reserve discriminator 0.
                    None => variant_index as Discriminator + 1,
                };

                quote!(#discriminator)
            },
            Enum::StringDiscriminator => {
                let variant_name = attr::name(&variant.attrs).unwrap_or_else(|| variant.ident.to_string());
                quote! { String::from(#variant_name) }
            },
        }
    }

    pub fn discriminator_for_pattern_matching(&self) -> ::proc_macro2::TokenStream {
        match *self {
            Enum::IntegerDiscriminator => quote!(discriminator),
            Enum::StringDiscriminator => quote!(&discriminator[..]),
        }
    }

    pub fn discriminator_variant_for_pattern_matching(&self, e: &syn::DataEnum,
                                                       variant: &syn::Variant) -> ::proc_macro2::TokenStream {
        match *self {
            Enum::IntegerDiscriminator => self.discriminator(e, variant),
            Enum::StringDiscriminator => {
                let variant_name = attr::name(&variant.attrs).unwrap_or_else(|| variant.ident.to_string());
                quote! { #variant_name }
            },
        }
    }

    pub fn discriminator_type(&self) -> ::proc_macro2::TokenStream {
        match *self {
            Enum::IntegerDiscriminator => {
                let s = syn::Ident::new(&format!("u{}", ::std::mem::size_of::<Discriminator>() * 8), ::proc_macro2::Span::call_site());
                quote!(#s)
            },
            Enum::StringDiscriminator => quote!(String),
        }
    }
}

impl Format for Enum {
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "integer" => Ok(Enum::IntegerDiscriminator),
            "string" => Ok(Enum::StringDiscriminator),
            _ => Err(()),
        }
    }
}

