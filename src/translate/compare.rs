use crate::{fragments, raw_fragment};
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::Typed;

pub enum ComparisonOperator {
    Gt,
    Ge,
    Lt,
    Le
}

impl ComparisonOperator {
    pub fn get_bash_operators(&self) -> (FragmentKind, Option<FragmentKind>) {
        match self {
            ComparisonOperator::Gt => (raw_fragment!(" > "), None),
            ComparisonOperator::Ge => (raw_fragment!(" > "), Some(raw_fragment!(" = "))),
            ComparisonOperator::Lt => (raw_fragment!(" < "), None),
            ComparisonOperator::Le => (raw_fragment!(" < "), Some(raw_fragment!(" = ")))
        }
    }
}

pub fn translate_comparison(
    meta: &mut TranslateMetadata,
    operator: ComparisonOperator,
    left: &Expr,
    right: &Expr
) -> FragmentKind {
    let left_stmt = VarStmtFragment::new("__left_comp", left.get_type(), left.translate(meta));
    let left_expr = VarExprFragment::from_stmt(&left_stmt).with_star_expansion(true).to_frag();
    meta.stmt_queue.push_back(left_stmt.to_frag());
    let right_stmt = VarStmtFragment::new("__right_comp", right.get_type(), right.translate(meta));
    let right_expr = VarExprFragment::from_stmt(&right_stmt).with_star_expansion(true).to_frag();
    meta.stmt_queue.push_back(right_stmt.to_frag());
    let (primary_operator, secondary_operator) = operator.get_bash_operators();
    let expr = if let Some(secondary_operator) = secondary_operator {
        fragments!("[[ ", left_expr.clone(), primary_operator, right_expr.clone(), " || ", left_expr, secondary_operator, right_expr, " ]] && echo 1 || echo 0")
    } else {
        fragments!("[[ ", left_expr, primary_operator, right_expr, " ]] && echo 1 || echo 0")
    };
    SubprocessFragment::new(expr).to_frag()
}
