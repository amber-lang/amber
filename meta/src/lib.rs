mod helper;
mod manager;
mod utils;

use crate::helper::HelperVisitor;
use crate::manager::ManagerVisitor;
use proc_macro::TokenStream;
use syn::visit::Visit;
use syn::*;

#[proc_macro_derive(ContextManager, attributes(context))]
pub fn context_manager(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let mut visitor = ManagerVisitor::new(&input.ident);
    visitor.visit_item_struct(&input);
    let output = visitor.make_block();
    TokenStream::from(output)
}

#[proc_macro_derive(ContextHelper, attributes(context))]
pub fn context_helper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let mut visitor = HelperVisitor::new(&input.ident);
    visitor.visit_item_struct(&input);
    let output = visitor.make_block();
    TokenStream::from(output)
}
