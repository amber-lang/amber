/// Tests for Amber scripts that check the standard library functionality.
use super::script_test;
use super::TestOutcomeTarget;
use test_generator::test_resources;

/// Autoload the Amber test files in stdlib, match the output in the comment
#[test_resources("src/tests/stdlib/*.ab")]
fn test_stdlib(input: &str) {
    script_test(input, TestOutcomeTarget::Success);
}

/// Unit tests for stdlib module
mod stdlib_tests {
    use crate::stdlib::resolve;
    use crate::tests::compile_code;
    use crate::tests::eval_bash;

    #[test]
    fn test_temp_dir_create_auto_deletes_every_directory() {
        let code = r#"
            import { temp_dir_create } from "std/fs"
            main {
                echo(temp_dir_create("amber-auto-delete-XXXX", true, true)?)
                echo(temp_dir_create("amber-auto-delete-XXXX", true)?)
                echo(temp_dir_create("amber-auto-delete-XXXX", true, true)?)
            }
        "#;
        let (stdout, stderr) = eval_bash(compile_code(code));
        assert_eq!(stderr, "");
        let dirs: Vec<&str> = stdout.lines().collect();
        assert_eq!(dirs.len(), 3);
        for dir in dirs {
            assert!(
                !std::path::Path::new(dir).exists(),
                "{dir} should be removed on exit"
            );
        }
    }

    #[test]
    fn test_resolve_existing_module() {
        let result = resolve("math");
        assert!(result.is_some(), "should find existing module");
        let content = result.unwrap();
        assert!(!content.is_empty(), "should have content");
    }

    #[test]
    fn test_resolve_non_existing_module() {
        let result = resolve("non/existing/path");
        assert!(result.is_none(), "should not find non-existing module");
    }

    #[test]
    fn test_resolve_test_module() {
        let result = resolve("test");
        assert!(result.is_some(), "should find module");
    }
}
