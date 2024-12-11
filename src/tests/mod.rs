use crate::compiler::{AmberCompiler, CompilerOptions};
extern crate test_generator;
use heraclitus_compiler::prelude::Message;
use itertools::Itertools;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub mod cli;
pub mod extra;
pub mod postprocessor;
mod stdlib;
mod validity;
mod erroring;

pub enum TestOutcomeTarget {
    Success,
    Failure,
}

pub fn eval_amber_code(code: &str) -> Result<String, Message> {
    let options = CompilerOptions::default();
    let mut compiler = AmberCompiler::new(code.to_string(), None, options);
    compiler.test_eval()
}

/// Tests script output in case of success or failure
pub fn test_amber(code: &str, result: &str, target: TestOutcomeTarget) {
    match target {
        TestOutcomeTarget::Success => {
            match eval_amber_code(code) {
                Ok(eval_result) => assert_eq!(
                    eval_result.trim_end_matches('\n'),
                    result.trim_end_matches('\n'),
                ),
                Err(err) => panic!("ERROR: {}", err.message.unwrap()),
            }
        }
        TestOutcomeTarget::Failure => {
            match eval_amber_code(code) {
                Ok(eval_result) => panic!("Expected error, got: {}", eval_result),
                Err(err) => assert_eq!(err.message.expect("Error message expected"), result),
            }
        }
    }
}

pub fn compile_code<T: Into<String>>(code: T) -> String {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.into(), None, options);
    let (_, code) = compiler.compile().unwrap();
    code
}

pub fn eval_bash<T: Into<String>>(code: T) -> (String, String) {
    let mut cmd = Command::new("bash");
    cmd.arg("-c");
    cmd.arg(code.into());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.spawn().unwrap().wait_with_output().unwrap();

    (
        String::from_utf8(output.stdout).unwrap().trim_end().into(),
        String::from_utf8(output.stderr).unwrap().trim_end().into(),
    )
}

/// Extracts the output from the comment of amber code
fn extract_output(code: impl Into<String>) -> String {
    code.into()
        .lines()
        .skip_while(|line| !line.starts_with("// Output"))
        .skip(1) // skip "// Output" itself
        .take_while(|line| !line.is_empty() && line.starts_with("//"))
        .map(|line| line.trim_start_matches("//").trim())
        .join("\n")
}

/// Inner test logic for testing script output in case of success or failure
pub fn script_test(input: &str, target: TestOutcomeTarget) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));

    // extract Output from script comment
    let mut output = extract_output(&code);

    // if output is not in comment, try to read from .output.txt file
    if output.is_empty() {
        let output_path = PathBuf::from(input.replace(".ab", ".output.txt"));
        output = match output_path.exists() {
            true => fs::read_to_string(output_path)
                .unwrap_or_else(|_| panic!("Failed to open {input}.output.txt file")),
            _ => "Succeeded".to_string(),
        };
    }
    test_amber(&code, &output, target);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_output() {
        assert_eq!(
            extract_output(
                r#"
some header
// some comment
// Output
// expected
// output

theres more code
not output

// Output
// another output is invalid

        "#
            ),
            "expected\noutput"
        );
    }
}
