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
            let msg = format!("Cannot {} value of type '{}' with value of type '{}'", $op_name, $left.get_type(), $right.get_type());
            let all = vec![$(format!("'{}'", stringify!($type_match))),+];
            let types = if all.len() > 1 { [
                all.iter().take(all.len() - 1).cloned().collect::<Vec<_>>().join(", "),
                all.last().unwrap().to_string()
            ].join(" or ") } else { all.join("") };
            let comment = format!("You can only {} values of type {types} together.", $op_name);
            Err(Failure::Loud(Message::new_err_at_position($meta, pos)
                .message(msg)
                .comment(comment)))
        } else {
            Ok($left.get_type())
        }
    }};

    ($meta:expr, $op_name:expr, $left:expr, $right:expr) => {{
        if $left.get_type() != $right.get_type() {
            let pos = $crate::modules::expression::binop::get_binop_position_info($meta, &$left, &$right);
            let msg = format!("Operation '{}' can only be used on arguments of the same type", $op_name);
            Err(Failure::Loud(Message::new_err_at_position($meta, pos).message(msg)))
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
