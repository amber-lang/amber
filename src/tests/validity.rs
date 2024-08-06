use super::script_test;
use test_generator::test_resources;

/// Autoload the Amber test files in validity, match the output in the comment
#[test_resources("src/tests/validity/*.ab")]
fn test_validity(input: &str) {
    script_test(input);
}
