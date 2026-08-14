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
        target: None,
        shebang: None,
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
        target: None,
        shebang: None,
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
        target: None,
        shebang: None,
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
        target: None,
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
        target: None,
    });

    assert!(result.is_ok());
}

#[test]
fn test_handle_eval_with_error() {
    // This test covers the error path in handle_eval (lines 262-265)
    // Using invalid code to trigger a compilation error
    let result = handle_eval(EvalCommand {
        code: "invalid amber syntax @@#$$".to_string(),
        target: None,
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
    let result = execute_output(code, vec![], true, None);
    assert!(result.is_ok());
}

#[test]
fn test_execute_output_without_messages() {
    let code = "exit 0".to_string();
    let result = execute_output(code, vec![], false, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
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

#[test]
fn test_validate_input_existence_existing_file() {
    use crate::validate_input_existence;
    let input = PathBuf::from("src/tests/functional/test.ab");
    let result = validate_input_existence(&input);
    assert!(result.is_ok());
}

#[test]
fn test_validate_input_existence_missing_file() {
    use crate::validate_input_existence;
    let input = PathBuf::from("nonexistent_file.ab");
    let result = validate_input_existence(&input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Input does not exist");
}

#[test]
fn test_validate_output_dir_none() {
    use crate::validate_output_dir;
    let output: Option<PathBuf> = None;
    let result = validate_output_dir(&output);
    assert!(result.is_ok());
}

#[test]
fn test_validate_output_dir_existing_dir() {
    use crate::validate_output_dir;
    let temp_dir = tempdir().unwrap();
    let output = Some(temp_dir.path().to_path_buf());
    let result = validate_output_dir(&output);
    assert!(result.is_ok());
}

#[test]
fn test_validate_output_dir_not_a_dir() {
    use crate::validate_output_dir;
    let temp_dir = tempdir().unwrap();
    let output = Some(temp_dir.path().join("file.txt"));
    let result = validate_output_dir(&output);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Output is not a directory");
}

#[test]
fn test_create_output_dir_with_output_flag() {
    use crate::BuildCommand;
    use crate::create_output_dir;
    
    let cmd = BuildCommand {
        input: PathBuf::from("src/"),
        output: Some(PathBuf::from("out")),
        no_proc: vec![],
        minify: false,
        target: None,
        shebang: None,
    };
    
    let input_file = PathBuf::from("src/test.ab");
    let result = create_output_dir(&cmd, &input_file);
    assert_eq!(result, PathBuf::from("out/test.sh"));
}

#[test]
fn test_create_output_dir_without_output_flag() {
    use crate::BuildCommand;
    use crate::create_output_dir;
    
    let cmd = BuildCommand {
        input: PathBuf::from("src/"),
        output: None,
        no_proc: vec![],
        minify: false,
        target: None,
        shebang: None,
    };
    
    let input_file = PathBuf::from("src/test.ab");
    let result = create_output_dir(&cmd, &input_file);
    assert_eq!(result, PathBuf::from("src/test.sh"));
}

#[test]
fn test_resolve_command_target_both_none() {
    use crate::resolve_command_target;
    let result = resolve_command_target(None, None);
    assert!(result.is_none());
}

#[test]
fn test_resolve_command_target_command_some() {
    use crate::resolve_command_target;
    use crate::ShellType;
    let cmd_target = Some(ShellType::BashModern);
    let result = resolve_command_target(cmd_target, None);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), ShellType::BashModern);
}

#[test]
fn test_resolve_command_target_cli_some() {
    use crate::resolve_command_target;
    use crate::ShellType;
    let cli_target = Some(ShellType::Zsh);
    let result = resolve_command_target(None, cli_target);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), ShellType::Zsh);
}

#[test]
fn test_resolve_command_target_both_some() {
    use crate::resolve_command_target;
    use crate::ShellType;
    let cmd_target = Some(ShellType::BashModern);
    let cli_target = Some(ShellType::Zsh);
    let result = resolve_command_target(cmd_target, cli_target);
    assert!(result.is_some());
    // Command target takes precedence
    assert_eq!(result.unwrap(), ShellType::BashModern);
}

#[test]
#[cfg(not(windows))]
fn test_set_file_permission() {
    use crate::set_file_permission;
    use std::fs;
    use std::os::unix::prelude::PermissionsExt;
    
    let temp_dir = tempdir().unwrap();
    let test_file_path = temp_dir.path().join("test_script.sh");
    
    // Create a test file
    fs::write(&test_file_path, "#!/bin/bash\necho test").unwrap();
    
    // Open the file as set_file_permission expects
    let file = fs::File::open(&test_file_path).unwrap();
    let path = test_file_path.to_string_lossy().to_string();
    
    // Set permissions
    set_file_permission(&file, path);
    
    // Verify permissions are set to 0o755
    let metadata = fs::metadata(&test_file_path).unwrap();
    let mode = metadata.permissions().mode();
    
    // Check that execute bits are set (0o755 = rwxr-xr-x)
    assert_eq!(mode & 0o777, 0o755);
}

#[test]
fn test_build_file() {
    use crate::build_file;
    use crate::BuildCommand;
    use crate::ShellType;
    use std::fs;
    
    let temp_dir = tempdir().unwrap();
    let input_file = temp_dir.path().join("test.ab");
    let output_file = temp_dir.path().join("test.sh");
    
    // Create a simple Amber input file
    fs::write(&input_file, "echo(\"Hello World\")").unwrap();
    
    let command = BuildCommand {
        input: temp_dir.path().to_path_buf(),
        output: None,
        no_proc: vec![],
        minify: false,
        target: None,
        shebang: None,
    };
    
    let target: Option<ShellType> = None;
    
    // Build the file
    build_file(&command, &target, input_file, output_file.clone());
    
    // Verify output file was created
    assert!(output_file.exists());
    
    // Verify output contains expected shell code
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("echo"));
    assert!(content.contains("Hello World"));
}
