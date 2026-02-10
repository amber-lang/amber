use crate::built_info;
use crate::execute_output;
use crate::handle_completion_with_output;
use crate::handle_docs;
use crate::handle_eval;
use crate::render_dash;
use crate::write_output;
use crate::DocsCommand;
use crate::EvalCommand;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_create_output_with_output_flag() {
    use crate::create_output;
    use crate::BuildCommand;

    let cmd = BuildCommand {
        input: PathBuf::from("test.ab"),
        output: Some(PathBuf::from("output.sh")),
        no_proc: vec![],
        minify: false,
    };

    let result = create_output(&cmd);
    assert_eq!(result, PathBuf::from("output.sh"));
}

#[test]
fn test_create_output_with_stdin() {
    use crate::create_output;
    use crate::BuildCommand;

    let cmd = BuildCommand {
        input: PathBuf::from("-"),
        output: None,
        no_proc: vec![],
        minify: false,
    };

    let result = create_output(&cmd);
    assert_eq!(result, PathBuf::from("-"));
}

#[test]
fn test_create_output_default_extension() {
    use crate::create_output;
    use crate::BuildCommand;

    let cmd = BuildCommand {
        input: PathBuf::from("test.amber"),
        output: None,
        no_proc: vec![],
        minify: false,
    };

    let result = create_output(&cmd);
    assert_eq!(result, PathBuf::from("test.sh"));
}

#[test]
fn test_compile_input_file() {
    use crate::compile_input;
    use crate::CompilerOptions;

    let input_file = PathBuf::from("src/tests/functional/test.ab");

    let options = CompilerOptions::default();
    let (code, messages) = compile_input(input_file, options);

    assert!(!code.is_empty());
    assert!(!messages);
}

#[test]
fn test_main_version() {
    let version = built_info::PKG_VERSION;
    assert!(!version.is_empty());
}

#[test]
fn test_handle_eval_success() {
    use crate::handle_eval;
    let result = handle_eval(EvalCommand {
        code: "1 + 1".to_string(),
    });
    assert!(result.is_ok());
}

#[test]
fn test_write_output_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_output.sh");
    let code = "echo \"test\"".to_string();

    write_output(output_file.clone(), code);

    assert!(output_file.exists());
    let content = std::fs::read_to_string(output_file).unwrap();
    assert_eq!(content, "echo \"test\"");
}

#[test]
fn test_render_dash_does_not_panic() {
    render_dash();
}

#[test]
fn test_handle_eval_with_empty_code() {
    let result = handle_eval(EvalCommand {
        code: std::fs::read_to_string("src/tests/testing/empty_out.ab").unwrap(),
    });

    assert!(result.is_ok());
}

#[test]
fn test_handle_eval_with_error() {
    // This test covers the error path in handle_eval (lines 262-265)
    // Using invalid code to trigger a compilation error
    let result = handle_eval(EvalCommand {
        code: "invalid amber syntax @@#$$".to_string(),
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_handle_docs_with_usage_flag() {
    let input_path = PathBuf::from("src/tests/stdlib/math_sum.ab");
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("index.html");

    let cmd = DocsCommand {
        input: input_path,
        output: Some(output_path.clone()),
        usage: true,
    };

    let result = handle_docs(cmd);
    assert!(result.is_ok());
}

#[test]
fn test_execute_output_with_messages() {
    let code = "exit 0".to_string();
    let result = execute_output(code, vec![], true);
    assert!(result.is_ok());
}

#[test]
fn test_handle_completion_does_not_panic() {
    let mut output = Vec::new();
    handle_completion_with_output(&mut output);
    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("_amber"));
    assert!(stdout.contains("amber)"));
}

#[test]
fn test_handle_completion_main() {
    let output = Command::new("cargo")
        .args(["run", "--", "completion"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_amber"));
}
