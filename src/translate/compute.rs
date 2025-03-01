use crate::{modules::prelude::{TranslationFragment, TranslationFragmentable}, utils::TranslateMetadata};

use crate::modules::prelude::*;
use crate::fragments;

use super::fragments::subprocess::SubprocessFragment;

pub enum ArithType {
    BcSed
}

pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Modulo,
    Neg,
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Neq,
    Not,
    And,
    Or
}

pub fn translate_computation(
    meta: &TranslateMetadata,
    operation: ArithOp,
    left: Option<TranslationFragment>,
    right: Option<TranslationFragment>
) -> TranslationFragment {
    match meta.arith_module {
        ArithType::BcSed => {
            let (left, right) = (
                left.unwrap_or_else(|| TranslationFragment::Empty),
                right.unwrap_or_else(|| TranslationFragment::Empty)
            );
            let mut math_lib_flag = true;
            // Removes trailing zeros from the expression
            let sed_regex = RawFragment::new("/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//").to_frag();
            let op = match operation {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "*",
                ArithOp::Div => "/",
                ArithOp::Modulo => {
                    math_lib_flag = false;
                    "%"
                },
                ArithOp::Neg => "-",
                ArithOp::Gt => ">",
                ArithOp::Ge => ">=",
                ArithOp::Lt => "<",
                ArithOp::Le => "<=",
                ArithOp::Eq => "==",
                ArithOp::Neq => "!=",
                ArithOp::Not => "!",
                ArithOp::And => "&&",
                ArithOp::Or => "||"
            };
            let math_lib_flag = RawFragment::new(if math_lib_flag { "-l" } else { "" }).to_frag();
            let operator = RawFragment::new(&format!(" '{op}' ")).to_frag();
            let value = fragments!("echo ", left, operator, right, " | bc ", math_lib_flag, " | sed '", sed_regex, "'");
            SubprocessFragment::new(value).to_frag()
        }
    }
}

pub fn translate_computation_eval(
    meta: &mut TranslateMetadata,
    operation: ArithOp,
    left: Option<TranslationFragment>,
    right: Option<TranslationFragment>,
    is_eval: bool,
) -> TranslationFragment {
    let old_eval = meta.eval_ctx;
    meta.eval_ctx = is_eval;
    let result = translate_computation(meta, operation, left, right);
    meta.eval_ctx = old_eval;
    result
}
