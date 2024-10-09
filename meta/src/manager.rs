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
            /// Sets the field value (which must implement the `Copy` and
            /// `Clone` traits) and restores the previous value after the
            /// body function has returned.
            pub fn #concat<T, E, B>(&mut self, #name: #segment, mut body: B) -> Result<T, E>
            where
                B: FnMut(&mut Self) -> Result<T, E>,
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
            /// Sets the field value by swapping the references, and
            /// restores the previous value after the body function has
            /// returned.
            pub fn #concat<T, E, B>(&mut self, #name: &mut #segment, mut body: B) -> Result<T, E>
            where
                B: FnMut(&mut Self) -> Result<T, E>,
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
            /// Sets the field value on the encapsulated struct using
            /// its member function, and restores the previous value
            /// after the body function has returned.
            ///
            /// Additionally, to add setter functions designed to work
            /// with `with_foo_fn()`, annotate the encapsulated struct
            /// with `#[derive(ContextHelper)`, and required fields with
            /// `#[context]`.
            pub fn #concat<V, S, T, E, B>(&mut self, mut setter: S, value: V, mut body: B) -> Result<T, E>
            where
                S: FnMut(&mut #segment, V) -> V,
                B: FnMut(&mut Self) -> Result<T, E>,
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
