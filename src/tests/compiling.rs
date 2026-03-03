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

#[test]
fn test_translate_sudo_preamble() {
    let code = r#"
main {
    echo("test")
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
    sudo $ echo "test" $?
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
    assert!(result.contains("sudo"), "Output should contain sudo");
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
fn test_parse_with_debug_flags() {
    let code = r#"main { echo("test") }"#;
    let options = CompilerOptions {
        debug_time: true,
        debug_parser: true,
        ..Default::default()
    };
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let result = compiler.parse(tokens);

    assert!(result.is_ok(), "Translate should succeed with debug flags");
}

#[test]
fn test_translate_with_debug_parser() {
    let code = r#"main { echo("test") }"#;
    let options = CompilerOptions {
        debug_parser: true,
        ..Default::default()
    };
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let result = compiler.parse(tokens);

    assert!(result.is_ok(), "Parse should succeed with debug parser");
}

#[test]
fn test_typecheck_with_debug_time() {
    let code = r#"main { echo("test") }"#;
    let options = CompilerOptions {
        debug_time: true,
        ..Default::default()
    };
    let compiler = AmberCompiler::new(code.to_string(), None, options);

    let tokens = compiler.tokenize().expect("tokenize failed");
    let (block, meta) = compiler.parse(tokens).expect("parse failed");

    let result = compiler.typecheck(block, meta);

    assert!(result.is_ok(), "Typecheck should succeed with debug time");
}

#[test]
fn test_parse_with_debug_time() {
    let code = r#"main { echo("test") }"#;
    let options = CompilerOptions {
        debug_time: true,
        ..Default::default()
    };
    let compiler = AmberCompiler::new(code.to_string(), None, options);

    let tokens = compiler.tokenize().expect("tokenize failed");
    let result = compiler.parse(tokens);

    assert!(result.is_ok(), "Parse should succeed with debug time");
}

#[test]
fn test_tokenize_error_singleline_missing_close() {
    let code = r#"main { echo "hello }"#;
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize();
    assert!(tokens.is_err(), "Should error on unclosed string");
}

#[test]
fn test_tokenize_error_unclosed_comment() {
    let code = r#"main { echo "hello" // comment without closing
"#;
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize();
    assert!(tokens.is_err(), "Should error on unclosed comment");
}

#[test]
fn test_document_with_output() {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(
        r#"
 main {
     echo("test")
 }
 "#
        .to_string(),
        Some("src/tests/validity/variable_simple.ab".to_string()),
        options,
    );

    let temp_dir = std::env::temp_dir();
    let tokens = compiler.tokenize().expect("tokenize failed");
    let (block, meta) = compiler.parse(tokens).expect("parse failed");
    let (block, meta) = compiler.typecheck(block, meta).expect("typecheck failed");

    compiler.document(block, meta, Some(temp_dir.to_string_lossy().to_string()));
}

#[test]
#[cfg(test)]
fn test_test_eval() {
    let code = r#"
main {
    echo("test")
}
"#;
    let options = CompilerOptions::default();
    let mut compiler = AmberCompiler::new(code.to_string(), None, options);
    let result = compiler.test_eval();
    assert!(result.is_ok(), "test_eval should succeed");
    assert!(
        result.unwrap().contains("test"),
        "Output should contain 'test'"
    );
}

#[test]
#[cfg(test)]
fn test_test_eval_with_error() {
    let code = r#"
main {
    this_is_not_valid_amber
}
"#;
    let options = CompilerOptions::default();
    let mut compiler = AmberCompiler::new(code.to_string(), None, options);
    let result = compiler.test_eval();
    assert!(result.is_err(), "test_eval should error on invalid code");
}

#[test]
fn test_gen_header_with_env() {
    let code = r#"main { echo("test") }"#;
    let temp_dir = std::env::temp_dir();
    let header_path = temp_dir.join("amber_test_header.sh");
    let header_content = "#!/usr/bin/env bash\n# Custom header\n";

    std::fs::write(&header_path, header_content).expect("Failed to write header file");

    let options = CompilerOptions {
        header_path: Some(header_path.to_string_lossy().to_string()),
        ..Default::default()
    };
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let (block, meta) = compiler.parse(tokens).expect("parse failed");
    let (block, meta) = compiler.typecheck(block, meta).expect("typecheck failed");
    let result = compiler.translate(block, meta);

    std::fs::remove_file(&header_path).ok();

    assert!(result.is_ok(), "Should succeed with custom header");
    let translated = result.unwrap();
    assert!(
        translated.starts_with("#!/usr/bin/env bash\n# Custom header"),
        "Should contain custom header"
    );
}

#[test]
fn test_gen_footer_with_env() {
    let code = r#"main { echo("test") }"#;
    let temp_dir = std::env::temp_dir();
    let footer_path = temp_dir.join("amber_test_footer.sh");
    let footer_content = "# Custom footer";

    std::fs::write(&footer_path, footer_content).expect("Failed to write footer file");

    let options = CompilerOptions {
        footer_path: Some(footer_path.to_string_lossy().to_string()),
        ..Default::default()
    };
    let compiler = AmberCompiler::new(code.to_string(), None, options);

    let tokens = compiler.tokenize().expect("tokenize failed");
    let (block, meta) = compiler.parse(tokens).expect("parse failed");
    let (block, meta) = compiler.typecheck(block, meta).expect("typecheck failed");

    let result = compiler.translate(block, meta);

    std::fs::remove_file(&footer_path).ok();

    assert!(result.is_ok(), "Should succeed with custom footer");
    let translated = result.unwrap();
    assert!(
        translated.contains("# Custom footer"),
        "Should contain custom footer"
    );
}
