/// Tests for Amber scripts that check snapshot of generated AST.
use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::modules::prelude::{FragmentKind, TranslateModule};
use crate::utils::TranslateMetadata;
use insta::assert_debug_snapshot;
use std::fs;
use std::path::Path;
use test_generator::test_resources;

pub fn translate_amber_code<T: Into<String>>(code: T) -> Option<FragmentKind> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.into(), None, options);
    let tokens = compiler.tokenize().ok()?;
    let (ast, meta) = compiler.parse(tokens).ok()?;
    let (ast, meta) = compiler.typecheck(ast, meta).ok()?;
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);
    Some(ast.translate(&mut translate_meta))
}

/// Autoload the Amber test files in translation
#[test_resources("src/tests/translating/*.ab")]
fn test_translation(input: &str) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));
    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");
    let filename = Path::new(input)
        .file_name()
        .expect("Provided directory")
        .to_str()
        .expect("Cannot translate to string");
    assert_debug_snapshot!(filename, ast);
}

mod compare_tests {
    use crate::translate::compare::ComparisonOperator;

    #[test]
    fn test_comparison_operator_to_string() {
        assert_eq!(ComparisonOperator::Gt.to_string(), ">");
        assert_eq!(ComparisonOperator::Ge.to_string(), ">=");
        assert_eq!(ComparisonOperator::Lt.to_string(), "<");
        assert_eq!(ComparisonOperator::Le.to_string(), "<=");
        assert_eq!(ComparisonOperator::Eq.to_string(), "==");
    }

    #[test]
    fn test_comparison_operator_to_arith_op() {
        use crate::translate::compute::ArithOp;

        assert_eq!(ComparisonOperator::Gt.to_arith_op(), ArithOp::Gt);
        assert_eq!(ComparisonOperator::Ge.to_arith_op(), ArithOp::Ge);
        assert_eq!(ComparisonOperator::Lt.to_arith_op(), ArithOp::Lt);
        assert_eq!(ComparisonOperator::Le.to_arith_op(), ArithOp::Le);
        assert_eq!(ComparisonOperator::Eq.to_arith_op(), ArithOp::Eq);
    }
}

mod compute_tests {
    use crate::modules::prelude::*;
    use crate::translate::compute::{
        translate_bc_sed_computation, translate_float_computation, ArithOp,
    };

    fn create_test_metadata() -> TranslateMetadata {
        let options = crate::compiler::CompilerOptions::default();
        let compiler = crate::compiler::AmberCompiler::new(
            "main { echo(\"test\") }".to_string(),
            None,
            options,
        );
        let tokens = compiler.tokenize().unwrap();
        let (ast, meta) = compiler.parse(tokens).unwrap();
        let (_, meta) = compiler.typecheck(ast, meta).unwrap();
        TranslateMetadata::new(meta, &compiler.options)
    }

    fn raw_frag(s: &str) -> FragmentKind {
        RawFragment::new(s).to_frag()
    }

    #[test]
    fn test_translate_bc_sed_computation_add() {
        let result = translate_bc_sed_computation(ArithOp::Add, raw_frag("5"), raw_frag("3"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("echo"), "should contain echo");
        assert!(code.contains("+"), "should contain + operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_sub() {
        let result = translate_bc_sed_computation(ArithOp::Sub, raw_frag("5"), raw_frag("3"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("-"), "should contain - operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_mul() {
        let result = translate_bc_sed_computation(ArithOp::Mul, raw_frag("5"), raw_frag("3"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("*"), "should contain * operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_div() {
        let result = translate_bc_sed_computation(ArithOp::Div, raw_frag("6"), raw_frag("3"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("/"), "should contain / operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_modulo() {
        let result = translate_bc_sed_computation(ArithOp::Modulo, raw_frag("5"), raw_frag("3"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("%"), "should contain % operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_gt() {
        let result = translate_bc_sed_computation(ArithOp::Gt, raw_frag("5"), raw_frag("3"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains(">"), "should contain > operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_ge() {
        let result = translate_bc_sed_computation(ArithOp::Ge, raw_frag("5"), raw_frag("3"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains(">="), "should contain >= operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_lt() {
        let result = translate_bc_sed_computation(ArithOp::Lt, raw_frag("3"), raw_frag("5"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("<"), "should contain < operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_le() {
        let result = translate_bc_sed_computation(ArithOp::Le, raw_frag("3"), raw_frag("5"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("<="), "should contain <= operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_eq() {
        let result = translate_bc_sed_computation(ArithOp::Eq, raw_frag("5"), raw_frag("5"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("=="), "should contain == operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_neq() {
        let result = translate_bc_sed_computation(ArithOp::Neq, raw_frag("5"), raw_frag("3"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("!="), "should contain != operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_neg() {
        let result = translate_bc_sed_computation(ArithOp::Neg, raw_frag("5"), raw_frag(""));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("-"), "should contain - operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_not() {
        let result = translate_bc_sed_computation(ArithOp::Not, raw_frag("true"), raw_frag(""));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("!"), "should contain ! operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_and() {
        let result = translate_bc_sed_computation(ArithOp::And, raw_frag("true"), raw_frag("true"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("&&"), "should contain && operator");
    }

    #[test]
    fn test_translate_bc_sed_computation_or() {
        let result = translate_bc_sed_computation(ArithOp::Or, raw_frag("true"), raw_frag("false"));
        let mut meta = create_test_metadata();
        let code = result.to_string(&mut meta);

        assert!(code.contains("||"), "should contain || operator");
    }

    #[test]
    fn test_translate_float_computation_bc_sed() {
        let meta = create_test_metadata();

        let result = translate_float_computation(
            &meta,
            ArithOp::Add,
            Some(raw_frag("5")),
            Some(raw_frag("3")),
        );
        let mut meta2 = create_test_metadata();
        let code = result.to_string(&mut meta2);

        assert!(code.contains("+"), "should contain + operator");
    }

    #[test]
    fn test_translate_float_computation_with_none() {
        let meta = create_test_metadata();

        let result = translate_float_computation(&meta, ArithOp::Add, None, None);
        let mut meta2 = create_test_metadata();
        let code = result.to_string(&mut meta2);

        assert!(code.contains("echo"), "should contain echo");
    }
}
