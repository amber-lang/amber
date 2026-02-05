use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::compute::translate_float_computation;
use crate::{fragments, raw_fragment};

use super::fragments::var_expr::VarIndexValue;

pub enum ComparisonOperator {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
}

impl ComparisonOperator {
    fn get_bash_lexical_operators(&self) -> (ComparisonOperator, Option<ComparisonOperator>) {
        match self {
            ComparisonOperator::Gt => (ComparisonOperator::Gt, None),
            ComparisonOperator::Ge => (ComparisonOperator::Gt, Some(ComparisonOperator::Eq)),
            ComparisonOperator::Lt => (ComparisonOperator::Lt, None),
            ComparisonOperator::Le => (ComparisonOperator::Lt, Some(ComparisonOperator::Eq)),
            ComparisonOperator::Eq => (ComparisonOperator::Eq, None),
        }
    }

    fn get_opposite_operator(&self) -> ComparisonOperator {
        match self {
            ComparisonOperator::Gt => ComparisonOperator::Le,
            ComparisonOperator::Ge => ComparisonOperator::Lt,
            ComparisonOperator::Lt => ComparisonOperator::Ge,
            ComparisonOperator::Le => ComparisonOperator::Gt,
            ComparisonOperator::Eq => ComparisonOperator::Eq,
        }
    }

    pub fn to_arith_op(&self) -> ArithOp {
        match self {
            ComparisonOperator::Gt => ArithOp::Gt,
            ComparisonOperator::Ge => ArithOp::Ge,
            ComparisonOperator::Lt => ArithOp::Lt,
            ComparisonOperator::Le => ArithOp::Le,
            ComparisonOperator::Eq => ArithOp::Eq,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            ComparisonOperator::Gt => ">",
            ComparisonOperator::Ge => ">=",
            ComparisonOperator::Lt => "<",
            ComparisonOperator::Le => "<=",
            ComparisonOperator::Eq => "==",
        }
    }

    pub fn to_frag(&self) -> FragmentKind {
        raw_fragment!(" {} ", self.to_string())
    }
}

pub fn translate_lexical_comparison(
    meta: &mut TranslateMetadata,
    operator: ComparisonOperator,
    left: &Expr,
    right: &Expr,
) -> FragmentKind {
    let left = {
        let left_stmt = VarStmtFragment::new("left_comp", left.get_type(), left.translate(meta));
        let left_expr = VarExprFragment::from_stmt(&left_stmt)
            .with_array_to_string(true)
            .to_frag();
        meta.stmt_queue.push_back(left_stmt.to_frag());
        left_expr
    };
    let right = {
        let right_stmt =
            VarStmtFragment::new("right_comp", right.get_type(), right.translate(meta));
        let right_expr = VarExprFragment::from_stmt(&right_stmt)
            .with_array_to_string(true)
            .to_frag();
        meta.stmt_queue.push_back(right_stmt.to_frag());
        right_expr
    };
    let (primary_operator, secondary_operator) = operator.get_bash_lexical_operators();
    let expr = if let Some(secondary_operator) = secondary_operator {
        fragments!(
            "[[ ",
            left.clone(),
            primary_operator.to_frag(),
            right.clone(),
            " || ",
            left,
            secondary_operator.to_frag(),
            right,
            " ]] && echo 1 || echo 0"
        )
    } else {
        fragments!(
            "[[ ",
            left,
            primary_operator.to_frag(),
            right,
            " ]] && echo 1 || echo 0"
        )
    };
    SubprocessFragment::new(expr).to_frag()
}

fn create_variable_length_getter(
    meta: &mut TranslateMetadata,
    name: &str,
    expr: &Expr,
) -> VarExprFragment {
    let var_stmt = VarStmtFragment::new(name, expr.get_type(), expr.translate(meta));
    let var_expr = VarExprFragment::from_stmt(&var_stmt).with_length_getter(true);
    meta.stmt_queue.push_back(var_stmt.to_frag());
    var_expr
}

fn create_variable_with_smaller_number(
    name: &str,
    left: VarExprFragment,
    right: VarExprFragment,
) -> (VarStmtFragment, VarExprFragment) {
    let value = SubprocessFragment::new(fragments!(
        " (( ",
        left.clone().to_frag(),
        " < ",
        right.clone().to_frag(),
        " )) ",
        "&& echo ",
        left.clone().to_frag(),
        "|| echo ",
        right.clone().to_frag()
    ))
    .to_frag();
    let len_stmt = VarStmtFragment::new(name, Type::Num, value);
    let len_expr = VarExprFragment::from_stmt(&len_stmt).with_render_type(VarRenderType::NameOf);
    (len_stmt, len_expr)
}

fn compare_array_lengths(
    left_len: VarExprFragment,
    right_len: VarExprFragment,
    operator: ComparisonOperator,
) -> FragmentKind {
    let (op, eq) = operator.get_bash_lexical_operators();
    let comparison_fragment = fragments!(
        left_len.clone().to_frag(),
        op.to_frag(),
        right_len.clone().to_frag()
    );
    let full_comparison_fragment = if let Some(eq) = eq {
        fragments!(
            "(( ",
            left_len.clone().to_frag(),
            eq.to_frag(),
            right_len.clone().to_frag(),
            " || ",
            comparison_fragment,
            " ))"
        )
    } else {
        fragments!("(( ", comparison_fragment, " ))")
    };
    fragments!(full_comparison_fragment, " && echo 1 || echo 0").to_frag()
}

fn create_indexed_variable_with_default_fallback(
    name: &str,
    index: &str,
    expr: VarExprFragment,
) -> (VarStmtFragment, VarExprFragment) {
    let expr_value = expr
        .clone()
        .with_length_getter(false)
        .with_index_by_value(VarIndexValue::Index(raw_fragment!("{index}")))
        .with_default_value(raw_fragment!("0"));
    let var_stmt = VarStmtFragment::new(name, Type::Num, expr_value.to_frag());
    let var_expr = VarExprFragment::from_stmt(&var_stmt);
    (var_stmt, var_expr)
}

pub fn translate_array_lexical_comparison(
    meta: &mut TranslateMetadata,
    operator: ComparisonOperator,
    left: &Expr,
    right: &Expr,
    kind: Type,
) -> FragmentKind {
    let left_expr_length = create_variable_length_getter(meta, "left_comp", left);
    let right_expr_length = create_variable_length_getter(meta, "right_comp", right);
    // Compare lengths of arrays and choose the longest one
    let (len_stmt, len_expr) = create_variable_with_smaller_number(
        "len_comp",
        left_expr_length.clone(),
        right_expr_length.clone(),
    );
    // Iterator variables that will be used in the for loop
    let (left_helper_stmt, left_helper_expr) =
        create_indexed_variable_with_default_fallback("left", "i", left_expr_length.clone());
    let (right_helper_stmt, right_helper_expr) =
        create_indexed_variable_with_default_fallback("right", "i", right_expr_length.clone());
    // Get the operator and its opposite for the if statement
    let (op, _) = operator.get_bash_lexical_operators();
    let (inv_op, ..) = operator
        .get_opposite_operator()
        .get_bash_lexical_operators();
    let pretty_op = operator.to_string();
    // Get the return value when intersection of both left and right values are equal
    let compared_array_lengths =
        compare_array_lengths(left_expr_length, right_expr_length, operator);
    let if_cond = match kind {
        Type::Num => fragments!(
            "(( ",
            translate_float_computation(
                meta,
                op.to_arith_op(),
                Some(left_helper_expr.clone().to_frag()),
                Some(right_helper_expr.clone().to_frag())
            ),
            " != 0 ))"
        ),
        Type::Int => fragments!(
            "(( ",
            left_helper_expr.clone().to_frag(),
            op.to_frag(),
            right_helper_expr.clone().to_frag(),
            " ))"
        ),
        Type::Text => fragments!(
            "[[ ",
            left_helper_expr.clone().to_frag(),
            op.to_frag(),
            right_helper_expr.clone().to_frag(),
            " ]]"
        ),
        _ => unreachable!("Unsupported type {kind} in array lexical comparison"),
    };
    let elif_cond = match kind {
        Type::Num => fragments!(
            "(( ",
            translate_float_computation(
                meta,
                inv_op.to_arith_op(),
                Some(left_helper_expr.to_frag()),
                Some(right_helper_expr.to_frag())
            ),
            " != 0 ))"
        ),
        Type::Int => fragments!(
            "(( ",
            left_helper_expr.to_frag(),
            inv_op.to_frag(),
            right_helper_expr.to_frag(),
            " ))"
        ),
        Type::Text => fragments!(
            "[[ ",
            left_helper_expr.to_frag(),
            inv_op.to_frag(),
            right_helper_expr.to_frag(),
            " ]]"
        ),
        _ => unreachable!("Unsupported type {kind} in array lexical comparison"),
    };
    // If statement that compares two values of the arrays
    let if_stmt = BlockFragment::new(
        vec![
            fragments!("if ", if_cond, "; then"),
            BlockFragment::new(vec![fragments!("echo 1"), fragments!("exit")], true).to_frag(),
            fragments!("elif ", elif_cond, "; then"),
            BlockFragment::new(vec![fragments!("echo 0"), fragments!("exit")], true).to_frag(),
            fragments!("fi"),
        ],
        false,
    );
    // Create a for loop to iterate over the elements of the longest array
    let block = BlockFragment::new(
        vec![
            CommentFragment::new(&format!("Compare if left array {pretty_op} right array"))
                .to_frag(),
            len_stmt.to_frag(),
            fragments!("for (( i=0; i<", len_expr.to_frag(), "; i++ )); do"),
            BlockFragment::new(
                vec![
                    left_helper_stmt.to_frag(),
                    right_helper_stmt.to_frag(),
                    if_stmt.to_frag(),
                ],
                true,
            )
            .to_frag(),
            fragments!("done"),
            fragments!(compared_array_lengths, "\n"),
        ],
        true,
    );
    let var_stmt = VarStmtFragment::new(
        "comp",
        Type::Bool,
        SubprocessFragment::new(fragments!("\n", block.to_frag())).to_frag(),
    );
    meta.push_ephemeral_variable(var_stmt).to_frag()
}

pub fn translate_array_equality(
    left: VarExprFragment,
    right: VarExprFragment,
    negative: bool,
) -> FragmentKind {
    let left_len = left.clone().with_length_getter(true).to_frag();
    let right_len = right.clone().with_length_getter(true).to_frag();
    let left_index = left
        .with_index_by_value(VarIndexValue::Index(raw_fragment!("i")))
        .to_frag();
    let right_index = right
        .with_index_by_value(VarIndexValue::Index(raw_fragment!("i")))
        .to_frag();
    let false_val = raw_fragment!("{}", if negative { "1" } else { "0" });
    let true_val = raw_fragment!("{}", if negative { "0" } else { "1" });
    let block = BlockFragment::new(
        vec![
            fragments!(
                "(( ",
                left_len.clone(),
                " != ",
                right_len,
                " )) && echo ",
                false_val.clone(),
                " && exit"
            ),
            fragments!(
                "for (( i=0; i<",
                left_len.clone(),
                "; i++ )); do [[ \"",
                left_index,
                "\" != \"",
                right_index,
                "\" ]] && echo ",
                false_val,
                " && exit; done"
            ),
            fragments!("echo ", true_val, "\n"),
        ],
        true,
    );
    SubprocessFragment::new(fragments!("\n", block.to_frag())).to_frag()
}
