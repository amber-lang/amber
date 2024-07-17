#![cfg(test)]
extern crate test_generator;
use test_generator::test_resources;
use crate::compiler::AmberCompiler;
use crate::test_amber;
use std::fs;

/*
 * Autoload the Amber test files for validity and match the output with the output.txt file
 */
#[test_resources("src/tests/validity/*.ab")]
fn validity_test(input: &str) {
    let code = fs::read_to_string(input)
    .expect(&format!("Failed to open {input} test file"));

    let output = fs::read_to_string(input.replace(".ab", ".output.txt"))
    .expect(&format!("Failed to open {input}.output.txt file"));

    test_amber!(code, output);
}
