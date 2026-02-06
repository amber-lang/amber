use crate::fragments;
use crate::modules::prelude::*;

use super::fragments::subprocess::SubprocessFragment;

pub enum ArithType {
    BcSed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Or,
}

pub fn translate_bc_sed_computation(
    op: ArithOp,
    left: FragmentKind,
    right: FragmentKind,
) -> FragmentKind {
    let mut math_lib_flag = true;
    // Removes trailing zeros from the expression
    let sed_regex = RawFragment::new("/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//").to_frag();
    let op_str = match op {
        ArithOp::Add => "+",
        ArithOp::Sub => "-",
        ArithOp::Mul => "*",
        ArithOp::Div => "/",
        ArithOp::Modulo => {
            math_lib_flag = false;
            "%"
        }
        ArithOp::Neg => "-",
        ArithOp::Gt => ">",
        ArithOp::Ge => ">=",
        ArithOp::Lt => "<",
        ArithOp::Le => "<=",
        ArithOp::Eq => "==",
        ArithOp::Neq => "!=",
        ArithOp::Not => "!",
        ArithOp::And => "&&",
        ArithOp::Or => "||",
    };
    let math_lib_flag = RawFragment::new(if math_lib_flag { "-l" } else { "" }).to_frag();
    let operator = RawFragment::from(format!(" '{op_str}' ")).to_frag();
    let value = fragments!(
        "echo ",
        left,
        operator,
        right,
        " | bc ",
        math_lib_flag,
        " | sed '",
        sed_regex,
        "'"
    );
    SubprocessFragment::new(value).to_frag()
}

pub fn translate_float_computation(
    meta: &TranslateMetadata,
    operator: ArithOp,
    left: Option<FragmentKind>,
    right: Option<FragmentKind>,
) -> FragmentKind {
    match meta.arith_module {
        ArithType::BcSed => {
            let (left, right) = (
                left.unwrap_or(FragmentKind::Empty),
                right.unwrap_or(FragmentKind::Empty),
            );
            translate_bc_sed_computation(operator, left, right)
        }
    }
}

pub fn translate_computation_eval(
    meta: &mut TranslateMetadata,
    operator: ArithOp,
    left: Option<FragmentKind>,
    right: Option<FragmentKind>,
    is_eval: bool,
) -> FragmentKind {
    let old_eval = meta.eval_ctx;
    meta.eval_ctx = is_eval;
    let result = translate_float_computation(meta, operator, left, right);
    meta.eval_ctx = old_eval;
    result
}
