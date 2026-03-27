use std::fs;

use regex::Regex;
use test_generator::test_resources;

use crate::tests::{eval_amber, extract_output};

#[test_resources("src/tests/runtime/*.ab")]
fn test_runtime_errors(input: &str) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));

    let output = extract_output(&code);

    let re = Regex::new(&format!(r#"(?m)"?{output}"?$"#)).unwrap();

    match eval_amber(&code) {
        Ok((output, status)) => {
            assert_eq!(status.code(), Some(1));

            assert!(
                re.is_match(&output),
                "Expected stderr to end with the specific error, but got:\n{:?}",
                output
            );
        }
        _ => panic!("Expected runtime error."),
    }
}
