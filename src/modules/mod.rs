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

/// Variable or function name that is used or called
// TODO (performance): Symbol names are stored as owned `String`s, so the same names are
// heap-allocated and copied over and over: every clone of a `SymbolName`, a function
// signature, a variable declaration or a scope copies them again (e.g. `nameof` copies
// the function's name, and every monomorphization copies the whole declaration context).
// The same applies to `Token::word` (heraclitus), which is copied with every cloned token,
// and some declarations store a name next to the token that already holds it
// (e.g. `FunctionDeclaration::name` and `name_token`). Interning names and token text
// (e.g. with a string interner) would make them cheap `Copy` symbols and remove those
// duplicates. Interning token text requires changing the lexer in heraclitus.
#[derive(Debug, Clone, Default)]
pub struct SymbolName {
    /// Name by which this symbol is called in code (can be an alias from import)
    pub used: String,
    /// Name this symbol was declared with, used in translation
    pub declared: String,
}

impl SymbolName {
    pub fn new(used_name: String) -> Self {
        SymbolName { used: used_name, ..Default::default() }
    }

    pub fn from_declaration(name: String) -> Self {
        SymbolName { used: name.clone(), declared: name }
    }
}

/// Consumes a comment or line break at the cursor, if there is one.
/// Matching `'\n'` is enough: `AmberCompiler::new` normalizes CRLF and CR line
/// endings to LF before the code is tokenized.
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
    tok: &Option<Token>,
) -> Result<(), Failure> {
    if meta.get_function_in_current_scope(name).is_some()
        || meta.get_var_in_current_scope(name).is_some()
    {
        return error!(
            meta,
            tok.clone(),
            format!("Cannot redeclare '{}' in the same scope", name)
        );
    }

    Ok(())
}
