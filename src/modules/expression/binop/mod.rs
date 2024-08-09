use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
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
pub mod range;

pub trait BinOp: SyntaxModule<ParserMetadata> {
    fn set_left(&mut self, left: Expr);
    fn set_right(&mut self, right: Expr);
    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult;
}

#[macro_export]
macro_rules! handle_binop {
    (@internal type: Array) => {
        Type::Array(_)
    };

    (@internal type: $type:ident) => {
        Type::$type
    };

    ($meta:expr, $op_name:expr, $left:expr, $right:expr, [$($type_match:ident),+]) => {{
        let left_match = matches!($left.get_type(), $(handle_binop!(@internal type: $type_match))|*);
        let right_match = matches!($right.get_type(), $(handle_binop!(@internal type: $type_match))|*);
        if !left_match || !right_match || $left.get_type() != $right.get_type() {
            let pos = $crate::modules::expression::binop::get_binop_position_info($meta, &$left, &$right);
            let message = Message::new_err_at_position($meta, pos);
            error_type_match!($meta, message, $op_name, $left, $right, [$($type_match),+])
        } else {
            Ok($left.get_type())
        }
    }};

    ($meta:expr, $op_name:expr, $left:expr, $right:expr) => {{
        if $left.get_type() != $right.get_type() {
            let pos = $crate::modules::expression::binop::get_binop_position_info($meta, &$left, &$right);
            let message = Message::new_err_at_position($meta, pos);
            error_type_match!($meta, message, $op_name, $left, $right)
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
