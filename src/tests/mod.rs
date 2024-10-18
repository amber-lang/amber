use crate::compiler::AmberCompiler;
use crate::Cli;
extern crate test_generator;
use itertools::Itertools;
use std::fs;
use std::path::PathBuf;

pub mod cli;
pub mod errors;
pub mod extra;
pub mod formatter;
pub mod types;
mod stdlib;
mod validity;

/// compare the output of the given code with the expected output
pub fn test_amber(code: impl Into<String>, result: impl AsRef<str>) {
    match AmberCompiler::new(code.into(), None, Cli::default()).test_eval() {
        Ok(eval_result) => assert_eq!(
            eval_result.trim_end_matches('\n'),
            result.as_ref().trim_end_matches('\n')
        ),
        Err(err) => panic!("ERROR: {}", err.message.unwrap()),
    }
}

pub fn compile_code<T: Into<String>>(code: T) -> String {
    AmberCompiler::new(code.into(), None, Cli::default())
        .compile()
        .unwrap()
        .1
}

/// Extracts the output from the comment of amber code
pub fn extract_output(code: impl Into<String>) -> String {
    code.into()
        .lines()
        .skip_while(|line| !line.starts_with("// Output"))
        .skip(1) // skip "// Output" itself
        .take_while(|line| !line.is_empty() && line.starts_with("//"))
        .map(|line| line.trim_start_matches("//").trim())
        .join("\n")
}

/// inner test logic for script tests, used by stdlib tests and validity tests
pub fn script_test(input: &str) {
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
            _ => "Succeded".to_string(),
        };
    }

    test_amber(code, output);
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
