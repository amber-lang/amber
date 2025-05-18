use crate::{fragments, raw_fragment};
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};

use super::fragments::var_expr::VarIndexValue;

pub enum ComparisonOperator {
    Gt,
    Ge,
    Lt,
    Le
}

impl ComparisonOperator {
    fn get_bash_lexical_operators(&self) -> (FragmentKind, Option<FragmentKind>) {
        match self {
            ComparisonOperator::Gt => (raw_fragment!(" > "), None),
            ComparisonOperator::Ge => (raw_fragment!(" > "), Some(raw_fragment!(" = "))),
            ComparisonOperator::Lt => (raw_fragment!(" < "), None),
            ComparisonOperator::Le => (raw_fragment!(" < "), Some(raw_fragment!(" = ")))
        }
    }

    fn get_opposite_operator(&self) -> ComparisonOperator {
        match self {
            ComparisonOperator::Gt => ComparisonOperator::Lt,
            ComparisonOperator::Ge => ComparisonOperator::Le,
            ComparisonOperator::Lt => ComparisonOperator::Gt,
            ComparisonOperator::Le => ComparisonOperator::Ge
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            ComparisonOperator::Gt => ">",
            ComparisonOperator::Ge => ">=",
            ComparisonOperator::Lt => "<",
            ComparisonOperator::Le => "<="
        }
    }
}

pub fn translate_lexical_comparison(
    meta: &mut TranslateMetadata,
    operator: ComparisonOperator,
    left: &Expr,
    right: &Expr
) -> FragmentKind {
    let left = {
        let left_stmt = VarStmtFragment::new("__left_comp", left.get_type(), left.translate(meta));
        let left_expr = VarExprFragment::from_stmt(&left_stmt).with_star_expansion(true).to_frag();
        meta.stmt_queue.push_back(left_stmt.to_frag());
        left_expr
    };
    let right = {
        let right_stmt = VarStmtFragment::new("__right_comp", right.get_type(), right.translate(meta));
        let right_expr = VarExprFragment::from_stmt(&right_stmt).with_star_expansion(true).to_frag();
        meta.stmt_queue.push_back(right_stmt.to_frag());
        right_expr
    };
    let (primary_operator, secondary_operator) = operator.get_bash_lexical_operators();
    let expr = if let Some(secondary_operator) = secondary_operator {
        fragments!("[[ ", left.clone(), primary_operator, right.clone(), " || ", left, secondary_operator, right, " ]] && echo 1 || echo 0")
    } else {
        fragments!("[[ ", left, primary_operator, right, " ]] && echo 1 || echo 0")
    };
    SubprocessFragment::new(expr).to_frag()
}

pub fn translate_version_comparison(
    meta: &mut TranslateMetadata,
    operator: ComparisonOperator,
    left: &Expr,
    right: &Expr
) -> FragmentKind {
    let left = {
        let left_stmt = VarStmtFragment::new("__left_comp", left.get_type(), left.translate(meta));
        let left_expr = VarExprFragment::from_stmt(&left_stmt).with_length_getter(true);
        meta.stmt_queue.push_back(left_stmt.to_frag());
        left_expr
    };
    let right = {
        let right_stmt = VarStmtFragment::new("__right_comp", right.get_type(), right.translate(meta));
        let right_expr = VarExprFragment::from_stmt(&right_stmt).with_length_getter(true);
        meta.stmt_queue.push_back(right_stmt.to_frag());
        right_expr
    };
    // Compare lengths of arrays and choose the longest one
    let (len_stmt, len_expr) = {
        let value = SubprocessFragment::new(
            fragments!("(( ", left.clone().to_frag(), " > ", right.clone().to_frag(), "))",
                " && echo ", left.clone().to_frag(),
                " || echo ", right.clone().to_frag())).to_frag();
        let len_stmt = VarStmtFragment::new("__len_comp", Type::Num, value);
        let len_expr = VarExprFragment::from_stmt(&len_stmt).with_render_type(VarRenderType::NameOf);
        (len_stmt, len_expr)
    };
    // Iterator variables that will be used in the for loop
    let ((left_helper_stmt, left_helper_expr), (right_helper_stmt, right_helper_expr)) = {
        let left_value = left.clone()
            .with_length_getter(false)
            .with_index_by_value(VarIndexValue::Index(raw_fragment!("i")))
            .with_default_value(raw_fragment!("0"));
        let left_stmt = VarStmtFragment::new("__left", Type::Num, left_value.to_frag());
        let left_expr = VarExprFragment::from_stmt(&left_stmt);
        let right_value = right.clone()
            .with_length_getter(false)
            .with_index_by_value(VarIndexValue::Index(raw_fragment!("i")))
            .with_default_value(raw_fragment!("0"));
        let right_stmt = VarStmtFragment::new("__right", Type::Num, right_value.to_frag());
        let right_expr = VarExprFragment::from_stmt(&right_stmt);
        ((left_stmt, left_expr), (right_stmt, right_expr))
    };
    let (op, eq) = operator.get_bash_lexical_operators();
    let (inv_op, ..) = operator.get_opposite_operator().get_bash_lexical_operators();
    let pretty_op = operator.to_string();
    // Create a for loop to iterate over the elements of the longest array
    let block = BlockFragment::new(vec![
        CommentFragment::new(&format!("Compare if left array {pretty_op} right array")).to_frag(),
        len_stmt.to_frag(),
        fragments!("for (( i=0; i<", len_expr.to_frag(), "; i++ )); do"),
        BlockFragment::new(vec![
            left_helper_stmt.to_frag(),
            right_helper_stmt.to_frag(),
            fragments!("if (( ", left_helper_expr.clone().to_frag(), op, right_helper_expr.clone().to_frag(), " )); then"),
            BlockFragment::new(vec![
                fragments!("echo 1"),
                fragments!("exit"),
            ], true).to_frag(),
            fragments!("elif (( ", left_helper_expr.to_frag(), inv_op, right_helper_expr.to_frag(), " )); then"),
            BlockFragment::new(vec![
                fragments!("echo 0"),
                fragments!("exit"),
            ], true).to_frag(),
            fragments!("fi"),
        ], true).to_frag(),
        fragments!("done"),
        fragments!("echo ", if eq.is_some() { raw_fragment!("1") } else { raw_fragment!("0") }, "\n"),
    ], true);
    let var_stmt = VarStmtFragment::new("__comp", Type::Bool, SubprocessFragment::new(fragments!("\n", block.to_frag())).to_frag());
    meta.push_intermediate_variable(var_stmt).to_frag()
}

// $(
//     (( "${#a[@]}" > "${#b[@]}" )) && len="${#a[@]}" || len="${#b[@]}"
//     for (( i=0; i<len; i++ )); do
//         __left=${a[i]:-0}
//         __right=${b[i]:-0}
//         if (( __left < __right )); then
//             echo 1
//             exit
//         elif (( __left > __right )); then
//             echo 0
//             exit
//         fi
//     done
//     echo 0
// )
