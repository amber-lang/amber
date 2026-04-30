use std::fs;

use regex::Regex;
use test_generator::test_resources;

use crate::{compiler::{AmberCompiler, CompilerOptions}, tests::extract_output};

#[test_resources("src/tests/runtime/*.ab")]
fn test_runtime_errors(file: &str) {
    let code =
        fs::read_to_string(file).unwrap_or_else(|_| panic!("Failed to open {file} test file"));

    let output = extract_output(&code);

    let re = Regex::new(&format!(r#"(?m)"?{output}"?$"#)).unwrap();

    let target = AmberCompiler::resolve_target_shell(None);
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
