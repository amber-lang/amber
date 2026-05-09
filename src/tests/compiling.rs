/// Tests for Amber scripts that check snapshot of generated bash code.
use crate::compiler::{escape_shell_arg, AmberCompiler, CompilerOptions};
use crate::modules::prelude::TranslateModule;
use crate::modules::types::Type;
use crate::translate::fragments::fragment::FragmentRenderable;
use crate::translate::fragments::raw::RawFragment;
use crate::translate::fragments::var_expr::{VarExprFragment, VarIndexValue};
use crate::translate::fragments::var_stmt::VarStmtFragment;
use crate::utils::{ShellType, TranslateMetadata};
use insta::assert_snapshot;
use std::fs;
use std::path::Path;
use test_generator::test_resources;

pub fn translate_amber_code_with_target<T: Into<String>>(
    code: T,
    target: Option<ShellType>,
) -> Option<String> {
    let options = CompilerOptions::default().with_target(target);
    let compiler = AmberCompiler::new(code.into(), None, options);
    let tokens = compiler.tokenize().ok()?;
    let (ast, meta) = compiler.parse(tokens).ok()?;
    let (ast, meta) = compiler.typecheck(ast, meta).ok()?;
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);
    let ast = ast.translate(&mut translate_meta);
    let result = ast.to_string(&mut translate_meta);
    Some(result)
}

pub fn translate_compiler_output_with_target<T: Into<String>>(
    code: T,
    target: Option<ShellType>,
) -> Option<String> {
    let options = CompilerOptions::default().with_target(target);
    let compiler = AmberCompiler::new(code.into(), None, options);
    let tokens = compiler.tokenize().ok()?;
    let (ast, meta) = compiler.parse(tokens).ok()?;
    let (ast, meta) = compiler.typecheck(ast, meta).ok()?;
    compiler.translate(ast, meta).ok()
}

pub fn translate_amber_code<T: Into<String>>(code: T) -> Option<String> {
    translate_amber_code_with_target(code, None)
}

#[test]
fn test_translate_crlf_line_endings() {
    let code = "main {\r\n    echo(\"ok\")\r\n}\r\n";
    let output = translate_amber_code(code).expect("Couldn't translate Amber code with CRLF");
    assert!(output.contains("echo"));
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
    let filename = format!("{filename}__{}", AmberCompiler::resolve_target_shell(None));
    assert_snapshot!(filename, ast);
}

fn assert_target_snapshot(input: &str, target: ShellType) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));
    let ast = translate_amber_code_with_target(code, Some(target))
        .expect("Couldn't translate Amber code");
    let filename = Path::new(input)
        .file_name()
        .expect("Provided directory")
        .to_str()
        .expect("Cannot translate to string");
    let filename = format!("{filename}__{}", target.canonical_name());
    assert_snapshot!(filename, ast);
}

#[test]
fn test_bash_32_snapshot_variable_ref_set_number() {
    assert_target_snapshot(
        "src/tests/validity/variable_ref_set_number.ab",
        ShellType::BashLegacy,
    );
}

#[test]
fn test_bash_32_snapshot_array_assign_by_ref() {
    assert_target_snapshot(
        "src/tests/validity/array_assign_by_ref.ab",
        ShellType::BashLegacy,
    );
}

#[test]
fn test_bash_32_snapshot_array_get_negative_index_by_ref() {
    assert_target_snapshot(
        "src/tests/validity/array_get_negative_index_by_ref.ab",
        ShellType::BashLegacy,
    );
}

#[test]
fn test_bash_32_snapshot_array_get_excl_range_by_ref() {
    assert_target_snapshot(
        "src/tests/validity/array_get_excl_range_by_ref.ab",
        ShellType::BashLegacy,
    );
}

#[test]
fn test_bash_32_snapshot_array_get_incl_range_by_ref() {
    assert_target_snapshot(
        "src/tests/validity/array_get_incl_range_by_ref.ab",
        ShellType::BashLegacy,
    );
}

#[test]
fn test_bash_32_ref_array_len_preserves_prefix() {
    let code = "main { echo(\"ok\") }";
    let options = CompilerOptions::default().with_target(Some(ShellType::BashLegacy));
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let (ast, meta) = compiler.parse(tokens).expect("parse failed");
    let (_, meta) = compiler.typecheck(ast, meta).expect("typecheck failed");
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);

    let rendered = VarExprFragment::new("items", Type::array_of(Type::Text))
        .with_global_id(0)
        .with_ref(true)
        .with_declared(true)
        .with_length_getter(true)
        .to_string(&mut translate_meta);

    assert_eq!(rendered, "\"${#items_0_deref_0_array[@]}\"");
}

#[test]
fn test_bash_32_ref_array_negative_index_uses_pass_through_deref_access() {
    let code = "main { echo(\"ok\") }";
    let options = CompilerOptions::default().with_target(Some(ShellType::BashLegacy));
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let (ast, meta) = compiler.parse(tokens).expect("parse failed");
    let (_, meta) = compiler.typecheck(ast, meta).expect("typecheck failed");
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);

    let rendered = VarExprFragment::new("items", Type::Text)
        .with_global_id(0)
        .with_ref(true)
        .with_declared(true)
        .with_index_by_value(VarIndexValue::Index(crate::raw_fragment!("idx")))
        .to_string(&mut translate_meta);

    assert_eq!(
        rendered,
        "\"${items_0_deref_0_array[idx]?\"Index out of bounds (at unknown)\"}\""
    );
}

#[test]
fn test_zsh_declared_ref_array_assignment_defers_array_expansion_to_inner_eval() {
    let code = "main { echo(\"ok\") }";
    let options = CompilerOptions::default().with_target(Some(ShellType::Zsh));
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let (ast, meta) = compiler.parse(tokens).expect("parse failed");
    let (_, meta) = compiler.typecheck(ast, meta).expect("typecheck failed");
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);

    let rendered = VarStmtFragment::new(
        "target",
        Type::array_of(Type::Text),
        VarExprFragment::new("source", Type::array_of(Type::Text)).to_frag(),
    )
    .with_ref(true)
    .with_declared(true)
    .to_string(&mut translate_meta);

    assert_eq!(rendered, r#"eval "${target}=(\"\${source[@]}\")""#);
}

#[test]
fn test_bash_32_declared_ref_array_assignment_defers_array_expansion_to_inner_eval() {
    let code = "main { echo(\"ok\") }";
    let options = CompilerOptions::default().with_target(Some(ShellType::BashLegacy));
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let tokens = compiler.tokenize().expect("tokenize failed");
    let (ast, meta) = compiler.parse(tokens).expect("parse failed");
    let (_, meta) = compiler.typecheck(ast, meta).expect("typecheck failed");
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);

    let rendered = VarStmtFragment::new(
        "target",
        Type::array_of(Type::Text),
        VarExprFragment::new("source", Type::array_of(Type::Text)).to_frag(),
    )
    .with_ref(true)
    .with_declared(true)
    .to_string(&mut translate_meta);

    assert_eq!(rendered, r#"eval "${target}=(\"\${source[@]}\")""#);
}


#[test]
fn test_lock_default_path_uses_tmpdir_with_tmp_fallback() {
    let code = r#"
main {
    lock() ?
}
"#;
    let result = translate_amber_code(code).expect("Couldn't translate Amber code");

    assert!(
        result.contains("${TMPDIR:-/tmp}/${0##*/}.lock"),
        "Output should contain TMPDIR-based lock path with /tmp fallback"
    );
}

#[test]
fn test_translate_shellversion_preamble() {
    let code = r#"
main {
    echo(shellversion()[0])
}
"#;
    let result = translate_compiler_output_with_target(code, Some(ShellType::BashModern))
        .expect("Couldn't translate Amber code");

    assert!(
        result.contains(r#"IFS='.' read -A EXEC_SHELL_VERSION <<< "$ZSH_VERSION""#),
        "Output should contain the zsh shellversion preamble"
    );
    assert!(
        result.contains(r#"IFS='.' read -a EXEC_SHELL_VERSION <<< "${__exec_shell_version%% *}""#),
        "Output should contain the ksh shellversion preamble"
    );
    assert!(
        result.contains(
            r#"EXEC_SHELL_VERSION=("${BASH_VERSINFO[0]}" "${BASH_VERSINFO[1]}" "${BASH_VERSINFO[2]}")"#,
        ),
        "Output should contain the bash shellversion preamble"
    );
    assert!(
        result.contains("EXEC_SHELL_VERSION[0]"),
        "Output should reference the shellversion builtin variable"
    );
}

#[test]
fn test_translate_shellname_and_shellversion_share_single_preamble() {
    let code = r#"
main {
    echo(shellname())
    echo(shellversion()[0])
}
"#;
    let result = translate_compiler_output_with_target(code, Some(ShellType::BashModern))
        .expect("Couldn't translate Amber code");

    assert_eq!(
        result.matches(r#"if [ -n "$ZSH_VERSION" ]; then"#).count(),
        1
    );
    assert!(result.contains("EXEC_SHELL"));
    assert!(result.contains("EXEC_SHELL_VERSION"));
}

#[test]
fn test_translate_with_sudo() {
    let code = r#"
main {
    sudo $echo "test"$?
}
"#;
    let result = translate_compiler_output_with_target(code, Some(ShellType::BashModern))
        .expect("Couldn't translate Amber code");

    assert!(
        result.contains(r#"[ "$EUID" -ne 0 ] && { { command -v sudo >/dev/null 2>&1 && __sudo=sudo; } || { command -v doas >/dev/null 2>&1 && __sudo=doas; }; }"#),
        "Output should contain the correct sudo preamble"
    );
    assert!(
        result.contains(r#"${__sudo} echo"#),
        "Output should contain sudo echo command with spaces"
    );
}

#[test]
fn test_translate_with_silent() {
    let code = r#"
main {
    silent $echo 1$?
}
"#;
    let result = translate_compiler_output_with_target(code, Some(ShellType::BashModern))
        .expect("Couldn't translate Amber code");

    assert!(
        result.contains(r#"echo 1 >/dev/null"#),
        "Output should contain echo command with separated >/dev/null redirect"
    );
}

#[test]
fn test_translate_with_suppress() {
    let code = r#"
main {
    suppress $echo 2$?
}
"#;
    let result = translate_compiler_output_with_target(code, Some(ShellType::BashModern))
        .expect("Couldn't translate Amber code");

    assert!(
        result.contains(r#"echo 2 2>/dev/null"#),
        "Output should contain echo command with separated 2>/dev/null redirect"
    );
}

#[test]
fn test_find_shell() {
    let bash_cmd = AmberCompiler::find_shell(None);
    assert!(
        bash_cmd.is_some(),
        "find_shell should return Some(Command) on non-Windows"
    );
}

#[test]
fn test_target_from_shell_path() {
    assert_eq!(
        AmberCompiler::target_from_shell_path("/bin/zsh"),
        Some(ShellType::Zsh)
    );
    assert_eq!(
        AmberCompiler::target_from_shell_path("/bin/ksh"),
        Some(ShellType::Ksh)
    );
    assert_eq!(
        AmberCompiler::target_from_shell_path("/bin/bash"),
        Some(ShellType::BashModern)
    );
}

#[test]
fn test_runtime_shell_command_uses_target_family_when_shell_is_unset() {
    assert_eq!(
        AmberCompiler::runtime_shell_command(None, Some(ShellType::BashLegacy)),
        Some("bash".to_string())
    );
    assert_eq!(
        AmberCompiler::runtime_shell_command(None, Some(ShellType::Zsh)),
        Some("zsh".to_string())
    );
}

#[test]
fn test_runtime_shell_command_prefers_amber_shell_over_target() {
    assert_eq!(
        AmberCompiler::runtime_shell_command(Some("/bin/bash".to_string()), Some(ShellType::Zsh)),
        Some("/bin/bash".to_string())
    );
}

#[test]
fn test_runtime_shell_command_uses_shell_env_without_target() {
    assert_eq!(
        AmberCompiler::runtime_shell_command(Some("/bin/bash".to_string()), None),
        Some("/bin/bash".to_string())
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
        result.unwrap().0.contains("test"),
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

#[test]
fn test_escape_shell_arg() {
    use crate::compiler::escape_shell_arg;

    // Test basic escaping
    assert_eq!(escape_shell_arg("hello"), "hello");
    assert_eq!(escape_shell_arg("hello\"world"), "hello\\\"world");
    assert_eq!(escape_shell_arg("$HOME"), "\\$HOME");
    assert_eq!(escape_shell_arg("`pwd`"), "\\`pwd\\`");
    assert_eq!(escape_shell_arg("back\\slash"), "back\\\\slash");
    assert_eq!(escape_shell_arg("history!"), "history\\!");

    // Test shell injection attempts
    assert_eq!(escape_shell_arg("$(whoami)"), "\\$(whoami)");
    assert_eq!(escape_shell_arg("; rm -rf /"), "; rm -rf /"); // semicolon not escaped, but wrapped in quotes
    assert_eq!(escape_shell_arg(r#"test"$`\!"#), r#"test\"\$\`\\\!"#);
}

#[test]
fn test_shell_injection_command_substitution_blocked() {
    let amber_code = r#"main(args) { echo(args[3]) }"#;
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);

    let (messages, bash_code) = compiler.compile().expect("Failed to compile");

    let args = [
        "program-name".to_string(),
        "safe-arg".to_string(),
        "$(echo INJECTED)".to_string(),
    ];

    let target = AmberCompiler::resolve_target_shell(None);
    let mut shell = AmberCompiler::find_shell(Some(target)).expect("Failed to find shell");

    let args_with_escapes = args
        .iter()
        .map(|arg| format!("\"{}\"", escape_shell_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    let bash_code_with_args = format!("set -- {}\n{}", args_with_escapes, bash_code);

    let output = shell
        .arg("-c")
        .arg(&bash_code_with_args)
        .output()
        .expect("Failed to execute");

    let status = output.status;
    assert!(messages.len() <= 1);
    assert!(status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("$(echo INJECTED)"),
        "Expected stdout to contain literal '$(echo INJECTED)', got: {}",
        stdout
    );
}

#[test]
fn test_shell_injection_variable_expansion_blocked() {
    let amber_code = r#"main(args) { echo(args[2]) }"#;
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);

    let (messages, bash_code) = compiler.compile().expect("Failed to compile");

    let args = ["program-name".to_string(), "$HOME".to_string()];

    let target = AmberCompiler::resolve_target_shell(None);
    let mut shell = AmberCompiler::find_shell(Some(target)).expect("Failed to find shell");

    let args_with_escapes = args
        .iter()
        .map(|arg| format!("\"{}\"", escape_shell_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    let bash_code_with_args = format!("set -- {}\n{}", args_with_escapes, bash_code);

    let output = shell
        .arg("-c")
        .arg(&bash_code_with_args)
        .output()
        .expect("Failed to execute");

    let status = output.status;
    assert!(messages.len() <= 1);
    assert!(status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("$HOME"),
        "Expected stdout to contain literal '$HOME', got: {}",
        stdout
    );
}

#[test]
fn test_shell_injection_backslash_handled() {
    let amber_code = r#"main(args) { echo(args[2]) }"#;
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(amber_code.to_string(), None, options);

    let (messages, bash_code) = compiler.compile().expect("Failed to compile");

    let args = ["program-name".to_string(), "\\".to_string()];

    let target = AmberCompiler::resolve_target_shell(None);
    let mut shell = AmberCompiler::find_shell(Some(target)).expect("Failed to find shell");

    let args_with_escapes = args
        .iter()
        .map(|arg| format!("\"{}\"", escape_shell_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    let bash_code_with_args = format!("set -- {}\n{}", args_with_escapes, bash_code);

    let output = shell
        .arg("-c")
        .arg(&bash_code_with_args)
        .output()
        .expect("Failed to execute");

    let status = output.status;
    assert!(messages.len() <= 1);
    assert!(status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\\"),
        "Expected stdout to contain literal '\\\\', got: {}",
        stdout
    );
}
