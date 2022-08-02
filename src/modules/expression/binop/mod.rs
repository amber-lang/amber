use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;
use super::super::expression::expr::Expr;

pub mod add;
pub mod sub;
pub mod mul;
pub mod div;
pub mod and;
pub mod or;
pub mod gt;
pub mod ge;
pub mod lt;
pub mod le;

struct Binop;

impl Binop {
    fn parse_left_expr(meta: &mut ParserMetadata, module: &mut Expr, op: impl AsRef<str>) -> Result<usize, ErrorDetails> {
        // Save left border and run binop left cut border check
        let old_border = meta.binop_border;
        let new_border = Self::binop_left_cut(meta, op)?;
        meta.binop_border = Some(new_border);
        // Parse the left expression
        syntax(meta, module)?;
        // Revert border back to the original
        meta.binop_border = old_border;
        Ok(new_border)
    }

    // Check if this binop can actually take place and return a new boundary for the left hand expression
    fn binop_left_cut(meta: &mut ParserMetadata, op: impl AsRef<str>) -> Result<usize, ErrorDetails> {
        let old_index = meta.get_index();
        let mut parenthesis = 0;
        while let Some(token) = meta.get_token_at(meta.get_index()) {
            // If we were supposed to parse just a fraction
            if let Some(border) = meta.binop_border {
                if border <= meta.get_index() {
                    break
                }
            }
            match token.word.as_str() {
                "(" => parenthesis += 1,
                ")" => parenthesis -= 1,
                "\n" => break,
                _ => {}
            };
            if parenthesis == 0 && op.as_ref() == token.word {
                // Case when the operator is in the beginning of the line
                if meta.get_index() > old_index {
                    let new_index = meta.get_index();
                    meta.set_index(old_index);
                    return Ok(new_index)
                }
                else {
                    let err = ErrorDetails::from_metadata(meta);
                    meta.set_index(old_index);
                    return Err(err)
                }
            }
            meta.increment_index();
        }
        let err = ErrorDetails::from_metadata(meta);
        meta.set_index(old_index);
        Err(err)
    }

}
