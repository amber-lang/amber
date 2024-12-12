use super::script_test;
use super::TestOutcomeTarget;
use test_generator::test_resources;

/// Autoload the Amber test files in stdlib, match the output in the comment
#[test_resources("src/tests/stdlib/*.ab")]
fn test_stdlib(input: &str) {
    script_test(input, TestOutcomeTarget::Success);
}
