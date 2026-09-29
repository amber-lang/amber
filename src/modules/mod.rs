use heraclitus_compiler::{
    error,
    prelude::{token, token_by, Failure, Message, Token},
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

/// Consumes a comment or line break at the cursor, if there is one.
pub fn skip_comments_and_newlines(meta: &mut ParserMetadata) -> bool {
    token_by(meta, |token| {
        token.starts_with("//") || token.starts_with('\n')
    })
    .is_ok()
}

/// Parses `item (, item)* close`, tolerating comments, line breaks and a
/// trailing comma before `close`.
pub fn parse_comma_separated<T, F>(
    meta: &mut ParserMetadata,
    close: &str,
    mut item: F,
) -> Result<Vec<T>, Failure>
where
    F: FnMut(&mut ParserMetadata) -> Result<T, Failure>,
{
    let mut items = vec![];
    loop {
        if skip_comments_and_newlines(meta) {
            continue;
        }
        if token(meta, close).is_ok() {
            break;
        }
        items.push(item(meta)?);
        match token(meta, close) {
            Ok(_) => break,
            Err(_) => token(meta, ",")?,
        };
    }
    Ok(items)
}

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
