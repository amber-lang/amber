extern crate test_generator;
use crate::compiler::{AmberCompiler, CompilerOptions};
use heraclitus_compiler::prelude::Message;
use itertools::Itertools;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub mod cli;
pub mod extra;
pub mod postprocessor;
mod stdlib;
mod validity;
mod erroring;

const SUCCEEDED: &str = "Succeeded";

pub enum TestOutcomeTarget {
    Success,
    Failure,
}

fn eval_amber(code: &str) -> Result<String, Message> {
    let options = CompilerOptions::default();
    let mut compiler = AmberCompiler::new(code.to_string(), None, options);
    compiler.test_eval()
}

/// Tests script output in case of success or failure
pub fn test_amber(code: &str, result: &str, target: TestOutcomeTarget) {
    let evaluated = eval_amber(code);
    match target {
        TestOutcomeTarget::Success => match evaluated {
            Ok(stdout) => {
                let stdout = stdout.trim_end_matches('\n');
                if stdout != SUCCEEDED {
                    let result = result.trim_end_matches('\n');
                    assert_eq!(stdout, result)
                }
            }
            Err(err) => {
                panic!("ERROR: {}", err.message.unwrap())
            }
        }
        TestOutcomeTarget::Failure => match evaluated {
            Ok(stdout) => {
                panic!("Expected error, got: {}", stdout)
            }
            Err(err) => {
                let message = err.message.expect("Error message expected");
                assert_eq!(message, result)
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

/// Extracts the output from the comment of Amber code
fn extract_output(code: impl Into<String>) -> String {
    code.into()
        .lines()
        .skip_while(|line| !line.starts_with("// Output"))
        .skip(1) // skip "// Output" itself
        .take_while(|line| !line.is_empty() && line.starts_with("//"))
        .map(|line| trim_comment(line).trim_end())
        .join("\n")
}

fn trim_comment(line: &str) -> &str {
    let mut chars = line.chars();
    if chars.next() == Some('/') {
        if chars.next() == Some('/') {
            return if chars.next() == Some(' ') {
                &line[3..]
            } else {
                &line[2..]
            }
        }
    }
    line
}

/// Inner test logic for testing script output in case of success or failure
pub fn script_test(input: &str, target: TestOutcomeTarget) {
    let code = fs::read_to_string(input)
        .unwrap_or_else(|_| panic!("Failed to open {input} test file"));
    // Extract output from script comment
    let mut output = extract_output(&code);
    // If output is not in comment, try to read from .output.txt file
    if output.is_empty() {
        let path = PathBuf::from(input.replace(".ab", ".output.txt"));
        output = if path.exists() {
            fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to open {} test file", path.display()))
        } else {
            SUCCEEDED.to_string()
        };
    }
    test_amber(&code, &output, target);
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_extract_output() {
        assert_eq!(
            extract_output(
                r#"
some header
// some comment
// Output
//
//no space
// one space
//  two spaces
//   three spaces
//

more code
not output

// Output
// another output is invalid
        "#
            ),
            "\nno space\none space\n two spaces\n  three spaces\n"
        );
    }
}
