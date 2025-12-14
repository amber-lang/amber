/// Tests for Amber scripts that check snapshot of generated AST.
use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::modules::prelude::{FragmentKind, TranslateModule};
use crate::utils::TranslateMetadata;
use test_generator::test_resources;
use insta::assert_debug_snapshot;
use std::fs;
use std::path::Path;

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
    let code = fs::read_to_string(input)
        .unwrap_or_else(|_| panic!("Failed to open {input} test file"));
    let ast = translate_amber_code(code)
        .expect("Couldn't translate Amber code");
    let filename = Path::new(input).file_name()
        .expect("Provided directory")
        .to_str()
        .expect("Cannot translate to string");
    assert_debug_snapshot!(filename, ast);
}
