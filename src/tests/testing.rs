use crate::testing::{find_amber_files, get_tests_to_run, handle_test};
use crate::TestCommand;
use std::path::PathBuf;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_amber_files_dir_with_amber_files() {
        let dir = PathBuf::from("src/tests/validity");
        let mut files = vec![];

        let result = find_amber_files(&dir, &mut files);

        assert!(result.is_ok());
        assert!(!files.is_empty());
    }

    #[test]
    fn test_get_tests_to_run_single_file() {
        let test_file = PathBuf::from("src/tests/validity/test_named_syntax.ab");

        let command = TestCommand {
            input: test_file.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = get_tests_to_run(&command);

        assert!(result.is_ok());
        let tests = result.unwrap();
        assert!(tests.len() >= 2);
    }

    #[test]
    fn test_get_tests_to_run_with_pattern() {
        let test_file = PathBuf::from("src/tests/validity/test_named_syntax.ab");

        let command = TestCommand {
            input: test_file.clone(),
            args: vec!["foo".to_string()],
            no_proc: Vec::new(),
        };

        let result = get_tests_to_run(&command);

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_tests_to_run_invalid_file() {
        let command = TestCommand {
            input: PathBuf::from("/non/existent/path.ab"),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = get_tests_to_run(&command);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_tests_to_run_dir() {
        let test_dir = PathBuf::from("src/tests/validity");

        let command = TestCommand {
            input: test_dir.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = get_tests_to_run(&command);

        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_test_with_named_test() {
        let test_file = PathBuf::from("src/tests/validity/test_named_syntax.ab");

        let command = TestCommand {
            input: test_file.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = handle_test(command);

        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_test_with_get_non_existent_tests_folder() {
        let test_dir = PathBuf::from("/non/existent/directory/abc123");

        let command = TestCommand {
            input: test_dir.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = handle_test(command);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_handle_test_with_empty_output() {
        let test_file = PathBuf::from("src/tests/testing/empty_out.ab");

        let command = TestCommand {
            input: test_file.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = handle_test(command);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_handle_test_with_skipped_message() {
        let test_file = PathBuf::from("src/tests/validity/block_test.ab");

        let command = TestCommand {
            input: test_file.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = handle_test(command);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_get_tests_to_run_tokenize_error() {
        let test_file = PathBuf::from("src/tests/testing/tokenize_error.ab");

        let command = TestCommand {
            input: test_file.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = get_tests_to_run(&command);

        assert!(result.is_err());
    }

    #[test]
    fn test_handle_test_empty_dir() {
        let temp_empty = std::env::temp_dir().join("amber_empty_test_12345");

        std::fs::create_dir_all(&temp_empty).ok();

        let command = TestCommand {
            input: temp_empty.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = handle_test(command);

        std::fs::remove_dir_all(&temp_empty).ok();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_handle_test_message_without_text() {
        let test_file = PathBuf::from("src/tests/testing/parse_no_msg.ab");
        let command = TestCommand {
            input: test_file.clone(),
            args: vec![],
            no_proc: Vec::new(),
        };

        let result = handle_test(command);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

}
