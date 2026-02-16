/// Tests for Amber scripts that check snapshot of generated AST.
use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::modules::prelude::{FragmentKind, FragmentRenderable, TranslateModule};
use crate::optimizer::ephemeral_vars::remove_ephemeral_variables;
use crate::optimizer::optimize_fragments;
use crate::utils::TranslateMetadata;
use insta::assert_snapshot;
use std::fs;
use std::path::Path;
use test_generator::test_resources;

pub fn translate_and_optimize_amber_code<T: Into<String>>(code: T) -> Option<String> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.into(), None, options);
    let tokens = compiler.tokenize().ok()?;
    let (ast, meta) = compiler.parse(tokens).ok()?;
    let (ast, meta) = compiler.typecheck(ast, meta).ok()?;
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);
    let mut translation = ast.translate(&mut translate_meta);
    optimize_fragments(&mut translation);
    Some(translation.to_string(&mut translate_meta))
}

/// Autoload the Amber test files in translation
#[test_resources("src/tests/optimizing/*.ab")]
fn test_translation(input: &str) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));
    let output = translate_and_optimize_amber_code(code).expect("Couldn't translate Amber code");
    let filename = Path::new(input)
        .file_name()
        .expect("Provided directory")
        .to_str()
        .expect("Cannot translate to string");
    assert_snapshot!(filename, output);
}

/// Unit tests for ephemeral_vars optimizer
mod ephemeral_vars_tests {
    use crate::modules::prelude::*;

    fn create_var_stmt(name: &str, value: FragmentKind, is_ephemeral: bool) -> FragmentKind {
        FragmentKind::VarStmt(Box::new(VarStmt {
            name: name.to_string(),
            value: Box::new(value),
            is_ephemeral,
            is_mutable: false,
            position: None,
        }))
    }

    #[test]
    fn test_remove_ephemeral_variables_basic() {
        let ephemeral_value = Box::new(create_var_stmt("x", RawFragment::new("5").to_frag(), true));
        let usage = Box::new(create_var_stmt("y", VarExpr::new("x").to_frag(), false));

        let mut ast = FragmentKind::Block(Box::new(Block {
            statements: vec![*ephemeral_value, *usage],
            position: None,
        }));

        remove_ephemeral_variables(&mut ast);

        if let FragmentKind::Block(block) = &ast {
            assert_eq!(block.statements.len(), 1, "should remove ephemeral var");
        }
    }

    #[test]
    fn test_remove_ephemeral_variables_non_ephemeral() {
        let non_ephemeral = Box::new(create_var_stmt("x", RawFragment::new("5").to_frag(), false));
        let usage = Box::new(create_var_stmt("y", VarExpr::new("x").to_frag(), false));

        let mut ast = FragmentKind::Block(Box::new(Block {
            statements: vec![*non_ephemeral, *usage],
            position: None,
        }));

        remove_ephemeral_variables(&mut ast);

        if let FragmentKind::Block(block) = &ast {
            assert_eq!(
                block.statements.len(),
                2,
                "should not remove non-ephemeral var"
            );
        }
    }

    #[test]
    fn test_remove_ephemeral_variables_different_names() {
        let ephemeral = Box::new(create_var_stmt("x", RawFragment::new("5").to_frag(), true));
        let usage = Box::new(create_var_stmt("y", VarExpr::new("z").to_frag(), false));

        let mut ast = FragmentKind::Block(Box::new(Block {
            statements: vec![*ephemeral, *usage],
            position: None,
        }));

        remove_ephemeral_variables(&mut ast);

        if let FragmentKind::Block(block) = &ast {
            assert_eq!(block.statements.len(), 2, "should not remove different var");
        }
    }

    #[test]
    fn test_remove_ephemeral_variables_length_expression() {
        let ephemeral = Box::new(create_var_stmt(
            "arr",
            RawFragment::new("[1, 2, 3]").to_frag(),
            true,
        ));
        let usage = Box::new(create_var_stmt(
            "len",
            VarExpr {
                name: "arr".to_string(),
                is_length: true,
                ..Default::default()
            }
            .to_frag(),
            false,
        ));

        let mut ast = FragmentKind::Block(Box::new(Block {
            statements: vec![*ephemeral, *usage],
            position: None,
        }));

        remove_ephemeral_variables(&mut ast);

        if let FragmentKind::Block(block) = &ast {
            assert_eq!(block.statements.len(), 2, "should not remove for length");
        }
    }
}
