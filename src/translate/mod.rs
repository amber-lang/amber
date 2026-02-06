use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;

pub mod compare;
pub mod compute;
pub mod fragments;
pub mod module;

pub fn check_all_blocks(meta: &ParserMetadata) -> SyntaxResult {
    let mut stack = 0;
    for token in meta.context.expr.iter() {
        match token.word.as_str() {
            "{" => stack += 1,
            "}" => stack -= 1,
            _ => (),
        }
        if stack < 0 {
            return error!(meta, Some(token.clone()), "Unexpected closing bracket");
        }
    }
    Ok(())
}
