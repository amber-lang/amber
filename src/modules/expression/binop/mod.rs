use heraclitus_compiler::prelude::*;
use crate::{utils::{metadata::ParserMetadata}, modules::types::{Type, Typed}};
use super::super::expression::expr::Expr;

pub mod add;
pub mod sub;
pub mod mul;
pub mod div;
pub mod modulo;
pub mod and;
pub mod or;
pub mod gt;
pub mod ge;
pub mod lt;
pub mod le;
pub mod eq;
pub mod neq;

pub fn expression_arms_of_type(meta: &mut ParserMetadata, left: &Type, right: &Type, kinds: &[Type], tok_pos: Option<Token>, message: &str) -> Result<Type, Failure> {
    if kinds.iter().all(|kind | ![left, right].iter().all(|item| **item == *kind)) {
        error!(meta, tok_pos, message)
    } else {
        Ok(left.clone())
    }
}

pub fn expression_arms_of_same_type(meta: &mut ParserMetadata, left: &Expr, right: &Expr, tok_pos: Option<Token>, message: &str) -> SyntaxResult {
    if left.get_type() != right.get_type() {
        error!(meta, tok_pos, message)
    } else {
        Ok(())
    }
}

pub fn parse_left_expr(meta: &mut ParserMetadata, module: &mut Expr, op: &str) -> Result<usize, Failure> {
    // Save left border and run binop left cut border check
    let old_border = meta.binop_border;
    let new_border = binop_left_cut(meta, op)?;
    meta.binop_border = Some(new_border);
    // Parse the left expression
    syntax(meta, module)?;
    // Revert border back to the original
    meta.binop_border = old_border;
    Ok(new_border)
}

// Check if this binop can actually take place and return a new boundary for the left hand expression
fn binop_left_cut(meta: &mut ParserMetadata, op: &str) -> Result<usize, Failure> {
    let old_index = meta.get_index();
    let mut parenthesis = 0;
    while let Some(token) = meta.get_current_token() {
        // If we were supposed to parse just a fraction
        if let Some(border) = meta.binop_border {
            if border <= meta.get_index() {
                break
            }
        }
        match token.word.as_str() {
            "(" | "{" => parenthesis += 1,
            ")" | "}" => parenthesis -= 1,
            "\n" => break,
            _ => {}
        };
        if parenthesis == 0 && op == token.word {
            // Case when the operator is in the beginning of the line
            if meta.get_index() > old_index {
                let new_index = meta.get_index();
                meta.set_index(old_index);
                return Ok(new_index)
            }
            else {
                let err = PositionInfo::from_metadata(meta);
                meta.set_index(old_index);
                return Err(Failure::Quiet(err))
            }
        }
        meta.increment_index();
    }
    let err = PositionInfo::from_metadata(meta);
    meta.set_index(old_index);
    Err(Failure::Quiet(err))
}

pub fn strip_text_quotes(text: &mut String) {
    if text.starts_with('"') && text.ends_with('"') {
        *text = text[1..text.len() - 1].to_string();
    }
}
