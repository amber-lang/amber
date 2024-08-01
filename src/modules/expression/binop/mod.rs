use super::super::expression::expr::Expr;
use crate::utils::metadata::ParserMetadata;
use heraclitus_compiler::prelude::*;

pub mod add;
pub mod and;
pub mod div;
pub mod eq;
pub mod ge;
pub mod gt;
pub mod le;
pub mod lt;
pub mod modulo;
pub mod mul;
pub mod neq;
pub mod or;
pub mod range;
pub mod sub;

pub trait BinOp: SyntaxModule<ParserMetadata> {
    fn set_left(&mut self, left: Expr);
    fn set_right(&mut self, right: Expr);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}

#[macro_export]
macro_rules! handle_binop {
    ($meta:expr, $left:expr, $right:expr, $msg:expr, $($comment:ident,)? [$($type_match:pat),*]) => {{
        let left_match = matches!($left.get_type(), $($type_match)|*);
        let right_match = matches!($right.get_type(), $($type_match)|*);
        if !left_match || !right_match || $left.get_type() != $right.get_type() {
            let pos = $crate::modules::expression::binop::get_binop_position_info($meta, &$left, &$right);
            Err(Failure::Loud(Message::new_err_at_position($meta, pos)
                .message($msg)
                $(.comment($comment))?))
        } else {
            Ok($left.get_type())
        }
    }};

    ($meta:expr, $left:expr, $right:expr, $msg:expr $(,$comment:ident)?) => {{
        if $left.get_type() != $right.get_type() {
            let pos = $crate::modules::expression::binop::get_binop_position_info($meta, &$left, &$right);
            Err(Failure::Loud(Message::new_err_at_position($meta, pos)
                .message($msg)
                $(.comment($comment))?))
        } else {
            Ok($left.get_type())
        }
    }};
}

pub fn get_binop_position_info(meta: &ParserMetadata, left: &Expr, right: &Expr) -> PositionInfo {
    let begin = meta.get_token_at(left.pos.0);
    let end = meta.get_token_at(right.pos.1);
    PositionInfo::from_between_tokens(meta, begin, end)
}

pub fn strip_text_quotes(text: &mut String) {
    if text.starts_with('"') && text.ends_with('"') {
        *text = text[1..text.len() - 1].to_string();
    }
}
