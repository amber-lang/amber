use heraclitus_compiler::{
    error,
    prelude::{Failure, Message, Token},
};

use crate::utils::ParserMetadata;

pub mod block;
pub mod builtin;
pub mod command;
pub mod condition;
pub mod expression;
pub mod function;
pub mod imports;
pub mod keywords;
pub mod loops;
pub mod main;
pub mod prelude;
pub mod shorthand;
pub mod statement;
pub mod test;
pub mod typecheck;
pub mod types;
pub mod variable;

pub fn handle_symbol_scope_declaration(
    meta: &mut ParserMetadata,
    name: &str,
    tok: Option<Token>,
) -> Result<(), Failure> {
    if meta.get_function_in_current_scope(name).is_some()
        || meta.get_var_in_current_scope(name).is_some()
    {
        return error!(
            meta,
            tok,
            format!("Cannot redeclare '{}' in the same scope", name)
        );
    }

    Ok(())
}

pub trait ReachableBreak {
    fn has_reachable_break(&self) -> bool;
}

impl<T: ReachableBreak> ReachableBreak for Box<T> {
    fn has_reachable_break(&self) -> bool {
        (**self).has_reachable_break()
    }
}

impl<T: ReachableBreak> ReachableBreak for Option<T> {
    fn has_reachable_break(&self) -> bool {
        self.as_ref().is_some_and(ReachableBreak::has_reachable_break)
    }
}

impl<T: ReachableBreak> ReachableBreak for Vec<T> {
    fn has_reachable_break(&self) -> bool {
        self.iter().any(ReachableBreak::has_reachable_break)
    }
}

#[macro_export]
macro_rules! impl_reachable_break {
    ($ty:ty $(, $field:ident)*) => {
        impl $crate::modules::ReachableBreak for $ty {
            fn has_reachable_break(&self) -> bool {
                false $(|| $crate::modules::ReachableBreak::has_reachable_break(&self.$field))*
            }
        }
    };
}
