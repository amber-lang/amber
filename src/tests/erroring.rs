use super::script_test;
use super::TestOutcomeTarget;
use test_generator::test_resources;

/// Autoload the Amber test files in erroring, match the output in the comment
#[test_resources("src/tests/erroring/*.ab")]
fn test_erroring(input: &str) {
    script_test(input, TestOutcomeTarget::Failure);
}
