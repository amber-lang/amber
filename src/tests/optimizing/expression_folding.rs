//! Tests for constant folding in logical expressions (`And`, `Or`, and `Not`)
//! driven by side-effect-aware control-flow analysis.
use crate::modules::prelude::FragmentKind;
use crate::tests::{compile_code, eval_bash};

const SIDE_EFFECT_FN: &str = "fun side_effect(): Bool {\n echo(\"side\")\n return true\n}\n";

/// Compile and execute Amber source, then verify that constant folding preserved
/// the observable output produced by a side effect.
fn assert_side_effect_runs(code: &str, needle: &str) {
    let shell = compile_code(code);
    let (stdout, stderr) = eval_bash(shell);
    assert!(
        stdout.contains(needle),
        "Side effect should produce \"{needle}\" in stdout\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}

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
    assert_side_effect_runs(
        &(SIDE_EFFECT_FN.to_string() + r#"main { echo("{side_effect() and false}") }"#),
        "side",
    );
}

#[test]
fn and_preserves_right_side_effect() {
    assert_side_effect_runs(
        &(SIDE_EFFECT_FN.to_string() + r#"main { echo("{false and side_effect()}") }"#),
        "side",
    );
}

#[test]
fn and_preserves_side_effect_in_interpolated_text() {
    let code = r#"
fun effectful_text_fn(): Text {
    echo("side")
    return "text"
}

main {
    let result = false and ("{effectful_text_fn()}" == "text")
    echo(result)
}
"#;
    assert_side_effect_runs(code, "side");
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
    assert_side_effect_runs(
        &(SIDE_EFFECT_FN.to_string() + r#"main { echo("{side_effect() or true}") }"#),
        "side",
    );
}

#[test]
fn or_preserves_right_side_effect() {
    assert_side_effect_runs(
        &(SIDE_EFFECT_FN.to_string() + r#"main { echo("{true or side_effect()}") }"#),
        "side",
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
    let code = SIDE_EFFECT_FN.to_string() + r#"main { echo("{not (false and side_effect())}") }"#;
    assert_side_effect_runs(&code, "side");
    let generated = compile_code(code);
    assert!(
        generated.contains("$(( !"),
        "Must use arithmetic NOT operator to preserve evaluation\n--- output ---\n{generated}"
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

// ===== Bool comparison rendering (compare.rs::create_bool_comparison) =====

#[test]
fn bool_variable_in_condition_uses_comparison_form() {
    // A Bool variable used as a condition should render as [[ var != 0 ]],
    // not as a literal "true" or "false".
    let code = "main { let b = true\n if b: echo(\"yes\") }";
    let out = compile_code(code);
    assert!(
        out.contains("[[ ") && out.contains("!= 0"),
        "Bool variable in condition should use [[ var != 0 ]] form\n--- output ---\n{out}"
    );
    assert!(
        !out.contains("[[ true ]]"),
        "Should not contain literal 'true' in condition\n--- output ---\n{out}"
    );
}

#[test]
fn bool_variable_in_condition_short_circuit_defensive() {
    // The "0"→"false" and "1"→"true" short-circuit in create_bool_comparison
    // is a defensive measure for when a raw fragment "0" or "1" reaches a
    // condition context. In practice, the dead-code eliminator handles constant
    // bools before this path is reached, so this test verifies the folded
    // value exists correctly in the output pipeline.
    let code = &format!(
        "{}main {{ echo(\"{{{{true and true}}}}\") }}",
        SIDE_EFFECT_FN
    );
    let out = compile_code(code);
    assert!(
        out.contains("\"1\""),
        "Folded 'true and true' should produce literal \"1\"\n--- output ---\n{out}"
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
