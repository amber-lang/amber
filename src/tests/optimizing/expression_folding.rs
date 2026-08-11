//! Tests for constant folding in logical expressions (And, Or, Not)
//! and the Expr::has_side_effects() method.
//!
//! These tests cover the translate() branches in and.rs, or.rs, not.rs,
//! the has_side_effects() match arms in expr.rs, and the with_math_var()
//! fallback in fragment.rs.
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::FragmentKind;
use crate::tests::compile_code;
use heraclitus_compiler::prelude::*;

const SIDE_EFFECT_FN: &str = "fun side_effect(): Bool {\n echo(\"side\")\n return true\n}\n";

const SIMPLE_FN: &str = "fun f(): Bool {\n return true\n}\n";

fn assert_folds_to_one(code: &str) {
    let out = compile_code(code);
    assert!(
        out.contains("\"1\""),
        "Expected fold to \"1\"\n--- output ---\n{out}"
    );
    assert!(
        !out.contains("&&"),
        "Folded expression should not contain &&\n--- output ---\n{out}"
    );
}

fn assert_folds_to_zero(code: &str) {
    let out = compile_code(code);
    assert!(
        out.contains("\"0\""),
        "Expected fold to \"0\"\n--- output ---\n{out}"
    );
    assert!(
        !out.contains("&&") && !out.contains("||"),
        "Folded expression should not contain && or ||\n--- output ---\n{out}"
    );
}

// ===== And constant folding (and.rs translate branches) =====

#[test]
fn and_fold_true_and_true() {
    assert_folds_to_one(r#"main { echo("{true and true}") }"#);
}

#[test]
fn and_fold_false_and_true() {
    assert_folds_to_zero(r#"main { echo("{false and true}") }"#);
}

#[test]
fn and_preserve_side_effects() {
    let code = compile_code(SIDE_EFFECT_FN.to_string() + r#"main { echo("{false and side_effect()}") }"#);
    assert!(
        code.contains("side_effect"),
        "Must preserve side-effect call\n--- output ---\n{code}"
    );
}

#[test]
fn and_none_branch_variable() {
    let code = compile_code(r#"main { let x = true echo("{x and true}") }"#);
    assert!(
        code.contains("&&"),
        "Variable and constant should use && condition\n--- output ---\n{code}"
    );
}

// ===== Or constant folding (or.rs translate branches) =====

#[test]
fn or_fold_true_or_false() {
    let out = compile_code(r#"main { echo("{true or false}") }"#);
    assert!(
        out.contains("\"1\""),
        "Expected fold to \"1\"\n--- output ---\n{out}"
    );
    assert!(
        !out.contains("||"),
        "Folded expression should not contain ||\n--- output ---\n{out}"
    );
}

#[test]
fn or_fold_false_or_false() {
    let out = compile_code(r#"main { echo("{false or false}") }"#);
    assert!(
        out.contains("\"0\""),
        "Expected fold to \"0\"\n--- output ---\n{out}"
    );
    assert!(
        !out.contains("||"),
        "Folded expression should not contain ||\n--- output ---\n{out}"
    );
}

#[test]
fn or_preserve_side_effects() {
    let code = compile_code(SIDE_EFFECT_FN.to_string() + r#"main { echo("{true or side_effect()}") }"#);
    assert!(
        code.contains("side_effect"),
        "Must preserve side-effect call\n--- output ---\n{code}"
    );
}

#[test]
fn or_none_branch_variable() {
    let code = compile_code(r#"main { let x = false echo("{x or false}") }"#);
    assert!(
        code.contains("||"),
        "Variable or constant should use || condition\n--- output ---\n{code}"
    );
}

// ===== Not constant folding (not.rs translate branches) =====

#[test]
fn not_fold_true() {
    let out = compile_code(r#"main { echo("{not true}") }"#);
    assert!(
        out.contains("\"0\""),
        "not true should fold to \"0\"\n--- output ---\n{out}"
    );
}

#[test]
fn not_fold_false() {
    let out = compile_code(r#"main { echo("{not false}") }"#);
    assert!(
        out.contains("\"1\""),
        "not false should fold to \"1\"\n--- output ---\n{out}"
    );
}

#[test]
fn not_preserve_side_effects() {
    let code = compile_code(SIDE_EFFECT_FN.to_string() + r#"main { echo("{not (false and side_effect())}") }"#);
    assert!(
        code.contains("side_effect"),
        "Must preserve side effect\n--- output ---\n{code}"
    );
    assert!(
        code.contains('!'),
        "Must use NOT operator to preserve evaluation\n--- output ---\n{code}"
    );
}

#[test]
fn not_none_branch_bool_variable() {
    let code = compile_code(r#"main { let x = true echo("{not x}") }"#);
    assert!(
        code.contains('!'),
        "not bool variable should use arithmetic NOT\n--- output ---\n{code}"
    );
}

#[test]
fn not_none_branch_text_variable() {
    let code = compile_code(r#"main { let s = "hello" echo("{not s}") }"#);
    assert!(
        !code.contains("$(("),
        "not text should use condition-based NOT, not arithmetic\n--- output ---\n{code}"
    );
}

// ===== Expr::has_side_effects() coverage (expr.rs) =====
//
// Each test below targets a specific match arm in has_side_effects().

#[test]
fn has_side_effects_bool_arm() {
    // true and true: both operands are Bool -> has_side_effects returns false -> fold
    assert_folds_to_one(r#"main { echo("{true and true}") }"#);
}

#[test]
fn has_side_effects_variable_get_arm() {
    // false and x: x is VariableGet -> has_side_effects returns false -> fold
    assert_folds_to_zero(r#"main { let x = true echo("{false and x}") }"#);
}

#[test]
fn has_side_effects_and_arm() {
    // (false and true) and false: left is And -> delegates to And::has_side_effects
    assert_folds_to_zero(r#"main { echo("{false and true and false}") }"#);
}

#[test]
fn has_side_effects_or_arm() {
    // (true or false) or true: left is Or -> delegates to Or::has_side_effects
    let out = compile_code(r#"main { echo("{true or false or true}") }"#);
    assert!(
        out.contains("\"1\""),
        "Should fold to 1\n--- output ---\n{out}"
    );
    assert!(
        !out.contains("||"),
        "Folded should not contain ||\n--- output ---\n{out}"
    );
}

#[test]
fn has_side_effects_not_arm() {
    // false and not true: right is Not -> delegates to Not::has_side_effects
    assert_folds_to_zero(r#"main { echo("{false and not true}") }"#);
}

#[test]
fn has_side_effects_function_invocation_arm() {
    let code = compile_code(SIMPLE_FN.to_string() + r#"main { echo("{false and f()}") }"#);
    assert!(
        code.contains("f__0_v0"),
        "Must preserve function call (side effect)\n--- output ---\n{code}"
    );
}

#[test]
fn has_side_effects_some_wildcard_arm() {
    // not (false and side_effect()): the Parentheses wrapping the And hits
    // the Some(_) => true catch-all in has_side_effects.
    let code = compile_code(SIDE_EFFECT_FN.to_string() + r#"main { echo("{not (false and side_effect())}") }"#);
    assert!(
        code.contains("side_effect"),
        "Must preserve side effect via Parentheses Some(_) arm\n--- output ---\n{code}"
    );
}

#[test]
fn has_side_effects_none_arm() {
    // Expr::new() has value=None -> has_side_effects returns false
    let expr = Expr::new();
    assert!(
        !expr.has_side_effects(),
        "Default Expr (value=None) should have no side effects"
    );
}

// ===== FragmentKind::with_math_var() fallback (fragment.rs) =====

#[test]
fn with_math_var_fallback_non_var_expr() {
    // Non-VarExpr fragments should pass through with_math_var unchanged.
    let result = FragmentKind::Empty.with_math_var(true);
    assert!(
        matches!(result, FragmentKind::Empty),
        "Empty fragment should pass through with_math_var unchanged"
    );
}
