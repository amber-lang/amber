use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Field, Ident, Meta, PathSegment, Type};

// https://users.rust-lang.org/t/how-to-use-a-vector-of-tokenstreams-created-with-quote-within-quote/81092
pub fn make_block(name: &Ident, functions: &Vec<TokenStream2>) -> TokenStream2 {
    quote! {
        impl #name {
            #(#functions)*
        }
    }
}

pub fn is_context(attr: &Attribute) -> bool {
    if let Meta::Path(path) = &attr.meta {
        if let Some(segment) = path.segments.last() {
            let ident = segment.ident.to_string();
            if ident == "context" {
                return true;
            }
        }
    }
    false
}

pub fn get_type(field: &Field) -> Option<&PathSegment> {
    if let Type::Path(path) = &field.ty {
        path.path.segments.last()
    } else {
        None
    }
}
