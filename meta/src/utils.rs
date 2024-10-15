use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Field, Ident, Meta, PathSegment, Type};

/// Implements multiple functions for a struct implementation block.
pub fn make_block(name: &Ident, functions: &Vec<TokenStream2>) -> TokenStream2 {
    // See [https://users.rust-lang.org/t/how-to-use-a-vector-of-tokenstreams-created-with-quote-within-quote/81092].
    quote! {
        impl #name {
            #(#functions)*
        }
    }
}

/// Tests whether a given field attribute is `#[context]`, for both
/// `#[derive(ContextManager)]` and `#[derive(ContextHelper)]` enhanced
/// structs.
pub fn is_context(attr: &Attribute) -> bool {
    if let Meta::Path(path) = &attr.meta {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "context" {
                return true;
            }
        }
    }
    false
}

/// Gets the type of a given field.  Note, we use the `PathSegment` not
/// the contained `Ident`, because that supports generic field types
/// like `Option<String>`.
pub fn get_type(field: &Field) -> Option<&PathSegment> {
    if let Type::Path(path) = &field.ty {
        path.path.segments.last()
    } else {
        None
    }
}
