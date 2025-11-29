/// Tests for Amber scripts that check for warning messages.
use super::script_test;
use super::TestOutcomeTarget;
use test_generator::test_resources;

/// Autoload the Amber test files in warning, match the output in the comment
#[test_resources("src/tests/warning/*.ab")]
fn test_warning(input: &str) {
    script_test(input, TestOutcomeTarget::Success);
}
