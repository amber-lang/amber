use super::fragments::subprocess::SubprocessFragment;
use crate::fragments;
use crate::modules::prelude::*;
use crate::utils::ShellType;

pub enum ArithType {
    Awk,
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

pub fn translate_awk_computation(
    op: ArithOp,
    left: FragmentKind,
    right: FragmentKind,
    with_quotes: bool,
) -> FragmentKind {
    let op_str = match op {
        ArithOp::Add => "+",
        ArithOp::Sub => "-",
        ArithOp::Mul => "*",
        ArithOp::Div => "/",
        ArithOp::Modulo => "%",
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
    let operator = RawFragment::from(op_str.to_string()).to_frag();

    let value = match op {
        ArithOp::Gt | ArithOp::Ge | ArithOp::Lt | ArithOp::Le | ArithOp::Eq | ArithOp::Neq => {
            fragments!(
                "awk \'BEGIN {print (ARGV[1]", operator, "ARGV[2]) ? 1 : 0}\'",
                " ",
                left,
                " ",
                right
            )
        },
        ArithOp::Neg => {
            fragments!(
                "awk \'BEGIN { print -ARGV[1]; }\'",
                " ",
                left
            )
        },
        ArithOp::Not => {
            fragments!(
                "awk \'BEGIN {print (!ARGV[1]) ? 1 : 0}\'",
                " ",
                left
            )
        },
        _ => {
            fragments!(
                "awk \'BEGIN { print ARGV[1]", operator, "ARGV[2] }\'",
                " ",
                left,
                " ",
                right
            )
        }
    };
    SubprocessFragment::new(value)
        .with_quotes(with_quotes)
        .to_frag()
}

pub fn translate_float_computation(
    meta: &TranslateMetadata,
    operator: ArithOp,
    left: Option<FragmentKind>,
    right: Option<FragmentKind>,
) -> FragmentKind {
    match meta.arith_module {
        ArithType::Awk => {
            let (left, right) = (
                left.unwrap_or(FragmentKind::Empty),
                right.unwrap_or(FragmentKind::Empty),
            );
            match meta.target.shell {
                ShellType::BashModern | ShellType::BashLegacy | ShellType::Zsh => {
                    translate_awk_computation(operator, left, right, true)
                }
                // ksh doesn't support quoting inside arithmetic blocks
                ShellType::Ksh => translate_awk_computation(operator, left, right, false),
            }
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
