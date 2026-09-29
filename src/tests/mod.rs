extern crate test_generator;
use crate::compiler::{AmberCompiler, CompilerOptions};
use heraclitus_compiler::prelude::Message;
use itertools::Itertools;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};

pub mod cli;
pub mod compiling;
pub mod documentation;
mod erroring;
pub mod extra;
mod functional;
pub mod main_helpers;
pub mod modules;
pub mod grammar_ebnf;
pub mod optimizing;
pub mod postprocessor;
mod runtime;
mod stdlib;
mod test_mode;
mod testing;
pub mod translating;
mod validity;
mod warning;
mod io;

pub mod utils;

#[macro_export]
macro_rules! unwrap_fragment {
    ($expr:expr, $kind:ident) => {{
        match $expr {
            FragmentKind::$kind(fragment) => fragment,
            _ => panic!("Expected FragmentKind::{}", stringify!($kind)),
        }
    }};
}

const SUCCEEDED: &str = "Succeeded";

pub enum TestOutcomeTarget {
    Success,
    Failure,
}

pub fn eval_amber(code: &str) -> Result<(String, ExitStatus), Message> {
    let options = CompilerOptions::default();
    let mut compiler = AmberCompiler::new(code.to_string(), None, options);
    compiler.test_eval()
}

/// Tests script output in case of success or failure
pub fn test_amber(code: &str, result: &str, target: TestOutcomeTarget) {
    let evaluated = eval_amber(code);
    match target {
        TestOutcomeTarget::Success => match evaluated {
            Ok((stdout, _)) => {
                let stdout = stdout.trim_end_matches('\n');
                if stdout != SUCCEEDED {
                    let result = result.trim_end_matches('\n');
                    assert_eq!(stdout, result)
                }
            }
            Err(err) => {
                panic!("ERROR: {}", err.message.unwrap())
            }
        },
        TestOutcomeTarget::Failure => match evaluated {
            Ok((stdout, _)) => {
                panic!("Expected error, got: {stdout}")
            }
            Err(err) => {
                let message = err.message.expect("Error message expected");
                assert_eq!(message, result)
            }
        },
    }
}

pub fn compile_code<T: Into<String>>(code: T) -> String {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.into(), None, options);
    let (_, code) = compiler.compile().unwrap();
    code
}

pub fn eval_bash<T: Into<String>>(code: T) -> (String, String) {
    let mut cmd = AmberCompiler::find_shell(None).expect("Failed to find shell");
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
pub fn extract_output(code: impl Into<String>) -> String {
    code.into()
        .lines()
        .skip_while(|line| !line.starts_with("// Output"))
        .skip(1) // skip "// Output" itself
        .take_while(|line| !line.is_empty() && line.starts_with("//"))
        .map(|line| line.trim_start_matches("//").trim())
        .join("\n")
}

/// Extracts the expected output for `family` from `// Output [family]` blocks.
/// Returns None when the file declares no tagged blocks (plain `// Output` format).
/// Panics when tagged blocks are declared but none matches the current target.
pub fn extract_targeted_output(code: &str, family: &str) -> Option<String> {
    let lines: Vec<&str> = code.lines().collect();
    let mut tags: Vec<String> = Vec::new();
    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        match output_tag(lines[i]) {
            Some(tag) => {
                let mut content: Vec<&str> = Vec::new();
                i += 1;
                while i < lines.len()
                    && lines[i].starts_with("//")
                    && output_tag(lines[i]).is_none()
                {
                    content.push(lines[i].trim_start_matches("//").trim());
                    i += 1;
                }
                tags.push(tag.clone());
                blocks.push((tag, content.join("\n")));
            }
            None => i += 1,
        }
    }
    if blocks.is_empty() {
        return None;
    }
    match blocks.into_iter().find(|(tag, _)| tag == family) {
        Some((_, content)) => Some(content),
        None => panic!(
            "per-target output declared for [{}] but no block for target '{family}'",
            tags.join(", ")
        ),
    }
}

fn output_tag(line: &str) -> Option<String> {
    let tag = line
        .strip_prefix("// Output [")?
        .strip_suffix("]")?
        .to_string();
    if tag.is_empty() {
        return None;
    }
    Some(tag)
}

/// Inner test logic for testing script output in case of success or failure
pub fn script_test(input: &str, target: TestOutcomeTarget) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));
    // Per-target `// Output [target]` blocks take precedence over the plain
    // `// Output` block and the .output.txt fallback.
    let family = AmberCompiler::resolve_target_shell(None).family_name();
    let output = match extract_targeted_output(&code, family) {
        Some(targeted) => targeted,
        None => {
            let mut output = extract_output(&code);
            if output.is_empty() {
                let path = PathBuf::from(input.replace(".ab", ".output.txt"));
                output = if path.exists() {
                    fs::read_to_string(&path)
                        .unwrap_or_else(|_| panic!("Failed to open {} test file", path.display()))
                } else {
                    SUCCEEDED.to_string()
                };
            }
            output
        }
    };
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

    #[test]
    fn test_extract_targeted_output() {
        let code = "\nsome code\n// Output [bash]\n// 1\n// Output [zsh]\n// 0\n// Output [ksh]\n// 0\n\nmore code\n";
        assert_eq!(extract_targeted_output(code, "bash").as_deref(), Some("1"));
        assert_eq!(extract_targeted_output(code, "zsh").as_deref(), Some("0"));
        assert_eq!(extract_targeted_output(code, "ksh").as_deref(), Some("0"));
    }

    #[test]
    fn test_extract_targeted_output_empty_block() {
        let code = "// Output [bash]\n// 1\n// Output [zsh]\n// Output [ksh]\n";
        assert_eq!(extract_targeted_output(code, "bash").as_deref(), Some("1"));
        assert_eq!(extract_targeted_output(code, "zsh").as_deref(), Some(""));
        assert_eq!(extract_targeted_output(code, "ksh").as_deref(), Some(""));
    }

    #[test]
    fn test_extract_targeted_output_plain_format_returns_none() {
        let code = "// Output\n// plain output\n";
        assert_eq!(extract_targeted_output(code, "bash"), None);
    }

    #[test]
    #[should_panic(expected = "no block for target 'ksh'")]
    fn test_extract_targeted_output_missing_family_panics() {
        extract_targeted_output("// Output [bash]\n// 1\n", "ksh");
    }
}
