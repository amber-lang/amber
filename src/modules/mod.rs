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
