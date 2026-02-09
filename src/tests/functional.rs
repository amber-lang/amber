use crate::built_info;
use crate::{execute_output, handle_docs, write_output, DocsCommand, EvalCommand, TestCommand};
use std::path::PathBuf;

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

    let temp_dir = tempfile::tempdir().unwrap();
    let input_file = temp_dir.path().join("test.amber");
    std::fs::write(&input_file, "main { echo(\"hello\") }").unwrap();

    let options = CompilerOptions::default();
    let (code, messages) = compile_input(input_file.to_path_buf(), options);

    assert!(!code.is_empty());
    assert!(!messages);
}

#[test]
fn test_main_version() {
    let version = built_info::PKG_VERSION;
    assert!(!version.is_empty());
}

#[test]
fn test_main_git_version() {
    use crate::set_file_permission;
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("script.sh");
    let file = std::fs::File::create(&output_file).unwrap();

    set_file_permission(&file, output_file.to_string_lossy().to_string());

    let metadata = std::fs::metadata(&output_file).unwrap();
    let mode = metadata.permissions().mode();

    #[cfg(not(windows))]
    assert_eq!(mode & 0o777, 0o755);
}

#[test]
fn test_handle_eval_success() {
    use crate::handle_eval;

    let result = handle_eval(EvalCommand {
        code: "main { 42 }".to_string(),
    });
    assert!(result.is_ok());
}

#[test]
fn test_handle_docs_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("test.ab");
    std::fs::write(&input_path, "").unwrap();
    let output_path = temp_dir.path().join("index.html");

    let cmd = DocsCommand {
        input: input_path,
        output: Some(output_path.clone()),
        usage: false,
    };

    let result = handle_docs(cmd);
    assert!(result.is_ok());
}

#[test]
fn test_write_output_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.sh");

    write_output(output_file.clone(), "echo test".to_string());

    let content = std::fs::read_to_string(&output_file).unwrap();
    assert_eq!(content, "echo test");
}

#[test]
fn test_execute_output_without_messages() {
    let code = "echo test".to_string();
    let result = execute_output(code, vec![], false);
    assert!(result.is_ok());
}

#[test]
fn test_handle_test_success() {
    use std::path::Path;

    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.ab");
    std::fs::write(&test_file, "main { echo(\"hello\") }").unwrap();

    let cmd = TestCommand {
        input: Path::to_path_buf(&test_file),
        args: vec![],
        no_proc: vec![],
    };

    let result = crate::testing::handle_test(cmd);
    assert!(result.is_ok());
}
