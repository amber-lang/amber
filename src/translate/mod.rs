use heraclitus_compiler::prelude::*;
use crate::utils::ParserMetadata;

pub mod module;
pub mod fragments;
pub mod compute;
pub mod compare;

pub fn check_all_blocks(meta: &ParserMetadata) -> SyntaxResult {
    let mut stack = 0;
    for token in meta.context.expr.iter() {
        match token.word.as_str() {
            "{" => stack += 1,
            "}" => stack -= 1,
            _ => ()
        }
        if stack < 0 {
            return error!(meta, Some(token.clone()), "Unexpected closing bracket")
        }
    }
    Ok(())
}
