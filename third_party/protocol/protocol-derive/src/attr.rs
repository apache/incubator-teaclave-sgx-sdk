use format;

use syn;

/// Gets the discriminant format of an enum.
pub fn discriminant_format<F: format::Format>(attrs: &[syn::Attribute]) -> Option<F> {
    helper::protocol_meta_name_value_literal("discriminant", attrs).map(helper::expect_lit_str).map(|format_name| {
        match F::from_str(&format_name) {
            Ok(f) => f,
            Err(..) => panic!("invalid enum discriminant format: '{}'", format_name),
        }
    })
}

/// Gets the name as per the attributes.
pub fn name(attrs: &[syn::Attribute]) -> Option<String> {
    helper::protocol_meta_name_value_literal("name", attrs).map(helper::expect_lit_str)
}

mod helper {
    use syn;
    use proc_macro2;

    pub fn protocol_meta_list(attrs: &[syn::Attribute]) -> Option<syn::MetaList> {
        attrs.iter().filter_map(|attr| match attr.interpret_meta() {
            Some(syn::Meta::List(meta_list)) => {
                if meta_list.ident == syn::Ident::new("protocol", proc_macro2::Span::call_site()) {
                    Some(meta_list)
                } else {
                    // Unrelated attribute.
                    None
                }
            },
            _ => None,
        }).next()
    }

    pub fn protocol_meta_nested_name_values(attrs: &[syn::Attribute]) -> Vec<syn::MetaNameValue> {
        protocol_meta_list(attrs).map(|meta_list| {
            let name_values: Vec<_> = meta_list.nested.iter().
                map(|n| match n {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) => nv.clone(),
                    _ => panic!("attribute must look like #[protocol(name = \"value\")]"),
                }).collect();
            name_values
        }).unwrap_or_else(|| Vec::new())
    }

    pub fn protocol_meta_name_value_literal(meta_name: &str, attrs: &[syn::Attribute]) -> Option<syn::Lit> {
        protocol_meta_nested_name_values(attrs).iter().filter_map(|name_value| {
            if name_value.ident == syn::Ident::new(meta_name, proc_macro2::Span::call_site()) {
                Some(name_value.lit.clone())
            } else {
                None // Different meta_name
            }
        }).next()
    }

    pub fn expect_lit_str(lit: syn::Lit) -> String {
        match lit {
            syn::Lit::Str(s) => s.value(),
            _ => panic!("expected a string literal"),
        }
    }
}

