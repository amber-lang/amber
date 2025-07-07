use crate::utils;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::visit::Visit;
use syn::{Field, Ident, PathSegment};

pub struct HelperVisitor {
    name: Ident,
    functions: Vec<TokenStream2>,
}

impl HelperVisitor {
    pub fn new(name: &Ident) -> Self {
        Self {
            name: name.clone(),
            functions: Vec::new(),
        }
    }

    fn make_function(name: &Ident, segment: &PathSegment) -> TokenStream2 {
        let concat = format!("set_{name}");
        let concat = Ident::new(&concat, name.span());
        quote! {
            /// Sets the field value and returns the previous value.
            pub fn #concat(&mut self, mut #name: #segment) -> #segment {
                use std::mem::swap;
                swap(&mut self.#name, &mut #name);
                #name
            }
        }
    }

    pub fn make_block(&self) -> TokenStream2 {
        utils::make_block(&self.name, &self.functions)
    }
}

impl<'a> Visit<'a> for HelperVisitor {
    fn visit_field(&mut self, field: &'a Field) {
        if field.attrs.iter().any(utils::is_context) {
            if let Some(name) = &field.ident {
                if let Some(segment) = utils::get_type(field) {
                    self.functions.push(Self::make_function(name, segment));
                }
            }
        }
    }
}
