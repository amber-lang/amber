use crate::utils;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::visit::Visit;
use syn::{Field, Ident, PathSegment};

pub struct ManagerVisitor {
    name: Ident,
    functions: Vec<TokenStream2>,
}

impl ManagerVisitor {
    pub fn new(name: &Ident) -> Self {
        Self {
            name: name.clone(),
            functions: Vec::new(),
        }
    }

    fn make_with(name: &Ident, segment: &PathSegment) -> TokenStream2 {
        let concat = format!("with_{}", name);
        let concat = Ident::new(&concat, name.span());
        quote! {
            pub fn #concat<B>(&mut self, #name: #segment, mut body: B) -> SyntaxResult
            where
                B: FnMut(&mut Self) -> SyntaxResult,
            {
                // Native types are implicitly copied on clone.
                let prev = self.#name.clone();
                self.#name = #name;
                let result = body(self);
                self.#name = prev;
                result
            }
        }
    }

    fn make_with_ref(name: &Ident, segment: &PathSegment) -> TokenStream2 {
        let concat = format!("with_{}_ref", name);
        let concat = Ident::new(&concat, name.span());
        quote! {
            pub fn #concat<B>(&mut self, #name: &mut #segment, mut body: B) -> SyntaxResult
            where
                B: FnMut(&mut Self) -> SyntaxResult,
            {
                use std::mem::swap;
                swap(&mut self.#name, #name);
                let result = body(self);
                swap(&mut self.#name, #name);
                result
            }
        }
    }

    fn make_with_fn(name: &Ident, segment: &PathSegment) -> TokenStream2 {
        let concat = format!("with_{}_fn", name);
        let concat = Ident::new(&concat, name.span());
        quote! {
            pub fn #concat<V, S, B>(&mut self, mut setter: S, value: V, mut body: B) -> SyntaxResult
            where
                S: FnMut(&mut #segment, V) -> V,
                B: FnMut(&mut Self) -> SyntaxResult,
            {
                let prev = setter(&mut self.#name, value);
                let result = body(self);
                setter(&mut self.#name, prev);
                result
            }
        }
    }

    pub fn make_block(&self) -> TokenStream2 {
        utils::make_block(&self.name, &self.functions)
    }
}

impl<'a> Visit<'a> for ManagerVisitor {
    fn visit_field(&mut self, field: &'a Field) {
        if field.attrs.iter().any(utils::is_context) {
            if let Some(name) = &field.ident {
                if let Some(segment) = utils::get_type(field) {
                    self.functions.push(Self::make_with(name, segment));
                    self.functions.push(Self::make_with_ref(name, segment));
                    self.functions.push(Self::make_with_fn(name, segment));
                }
            }
        }
    }
}