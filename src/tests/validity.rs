#![cfg(test)]
extern crate test_generator;
use test_generator::test_resources;
use crate::compiler::AmberCompiler;
use crate::test_amber;
use crate::Cli;
use std::fs;
use std::path::Path;

/*
 * Autoload the Amber test files for validity and match the output with the output.txt file
 */
#[test_resources("src/tests/validity/*.ab")]
fn validity_test(input: &str) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));

    let mut is_output = false;
    let mut output = "".to_owned();
    for line in code.lines() {
        if line.starts_with("// Output") {
            is_output = true;
            continue;
        } else if line.is_empty() && is_output {
            is_output = false;
            break;
        }

        if is_output {
            if ! output.is_empty() {
                output.push_str("\n");
            }
            output.push_str(&line.replace("//", "").trim());
        }
    }

    if output.is_empty() {
        output = match Path::new(&input.replace(".ab", ".output.txt")).exists() {
            true => fs::read_to_string(input.replace(".ab", ".output.txt"))
                .expect(&format!("Failed to open {input}.output.txt file")),
            _ => "Succeded".to_string()
        };
    }

    test_amber!(code, output);
}
