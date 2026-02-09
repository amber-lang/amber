/// Tests for Amber scripts that check snapshot of generated bash code.
use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::modules::prelude::TranslateModule;
use crate::modules::prelude::*;
use crate::utils::TranslateMetadata;
use insta::assert_snapshot;
use std::fs;
use std::path::Path;
use test_generator::test_resources;

pub fn translate_amber_code<T: Into<String>>(code: T) -> Option<String> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.into(), None, options);
    let tokens = compiler.tokenize().ok()?;
    let (ast, meta) = compiler.parse(tokens).ok()?;
    let (ast, meta) = compiler.typecheck(ast, meta).ok()?;
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);
    let ast = ast.translate(&mut translate_meta);
    let result = ast.to_string(&mut translate_meta);
    Some(result)
}

#[test]
fn test_translate_sudo_preamble() {
    let code = r#"
main {
    echo "test"
}
"#;
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let (ast, meta) = compiler.parse(tokens).expect("parse failed");
    let (ast, meta) = compiler.typecheck(ast, meta).expect("typecheck failed");
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);
    let ast = ast.translate(&mut translate_meta);
    let result = ast.to_string(&mut translate_meta);
    assert!(
        result.contains("echo \"test\""),
        "Output should contain bash code"
    );
}

#[test]
fn test_translate_with_sudo() {
    let code = r#"
main {
    sudo $ echo "sudo test" $?
}
"#;
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let (ast, meta) = compiler.parse(tokens).expect("parse failed");
    let (ast, meta) = compiler.typecheck(ast, meta).expect("typecheck failed");
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);
    let ast = ast.translate(&mut translate_meta);
    let result = ast.to_string(&mut translate_meta);
    assert!(
        result.contains("sudo") || result.contains("sudo "),
        "Output may contain sudo"
    );
}

#[test]
fn test_find_bash() {
    let bash_cmd = AmberCompiler::find_bash();
    assert!(
        bash_cmd.is_some(),
        "find_bash should return Some(Command) on non-Windows"
    );
}

#[test]
fn test_test_eval() {
    let options = CompilerOptions::default();
    let mut compiler = AmberCompiler::new("main { echo \"test\" }".to_string(), None, options);
    let result = compiler.test_eval();
    assert!(
        result.is_ok(),
        "test_eval should succeed or return error for invalid bash"
    );
}

/// Autoload the Amber test files in compiling
#[test_resources("src/tests/compiling/*.ab")]
fn test_translation(input: &str) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));
    let ast = translate_amber_code(code).expect("Couldn't translate Amber code");
    let filename = Path::new(input)
        .file_name()
        .expect("Provided directory")
        .to_str()
        .expect("Cannot translate to string");
    assert_snapshot!(filename, ast);
}
