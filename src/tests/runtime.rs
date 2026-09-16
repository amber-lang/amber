use std::fs;

use regex::Regex;
use test_generator::test_resources;

use crate::{
    compiler::{AmberCompiler, CompilerOptions},
    tests::{extract_output, extract_targeted_output},
};

/// Shell error wording differs per target, so fixtures may declare
/// `// Output [bash|zsh|ksh]` blocks; fall back to the agnostic `// Output`.
fn extract_targeted_error(code: &str, target_family: &str) -> String {
    extract_targeted_output(code, target_family).unwrap_or_else(|| extract_output(code))
}

#[test_resources("src/tests/runtime/*.ab")]
fn test_runtime_errors(file: &str) {
    let code =
        fs::read_to_string(file).unwrap_or_else(|_| panic!("Failed to open {file} test file"));

    let target = AmberCompiler::resolve_target_shell(None);
    let expected_error = extract_targeted_error(&code, target.family_name());

    let re = Regex::new(&format!(r#"(?m)"?{expected_error}"?$"#)).unwrap();

    let options = CompilerOptions::default().with_target(Some(target));
    let mut compiler = AmberCompiler::new(code.to_string(), Some(file.to_string()), options);

    match compiler.test_eval() {
        Ok((output, status)) => {
            assert!(status.code().unwrap_or(0) > 0);

            assert!(
                re.is_match(&output),
                "Expected stderr to end with the specific error, but got:\n{:?}",
                output
            );
        }
        _ => panic!("Expected runtime error."),
    }
}
