use crate::compiler::postprocessor::PostProcessor;
use crate::tests::eval_bash;

use super::compile_code;

#[test]
fn test_postprocessor_new_and_name() {
    let pp = PostProcessor::new("test_pp", "echo");

    assert_eq!(pp.name, "test_pp");
    assert_eq!(pp.bin.to_string_lossy(), "echo");
}

#[test]
fn test_postprocessor_is_available() {
    let available_pp = PostProcessor::new("test_echo", "echo");
    let unavailable_pp = PostProcessor::new("test_nonexistent", "nonexistent_binary_12345");

    assert!(available_pp.is_available(), "echo should be available");
    assert!(
        !unavailable_pp.is_available(),
        "nonexistent binary should not be available"
    );
}

#[test]
fn test_postprocessor_execute_simple_command() {
    let cat_pp = PostProcessor::new("test_cat", "cat");
    let test_input = "Hello World\n";

    let result = cat_pp.execute(test_input.to_string());

    assert!(result.is_ok(), "cat should execute successfully");
    let output = result.unwrap();
    assert!(
        output.trim_end().contains("Hello World"),
        "output should contain input"
    );
}

#[test]
fn test_postprocessor_execute_with_unavailable_processor() {
    let unavailable_pp = PostProcessor::new("test_nonexistent", "nonexistent_binary_12345");
    let test_input = "Hello World";

    let result = unavailable_pp.execute(test_input.to_string());

    assert!(
        result.is_ok(),
        "execute should not fail for unavailable processor"
    );
    assert_eq!(result.unwrap(), test_input, "should return original code");
}

#[test]
fn test_postprocessor_get_default() {
    let processors = PostProcessor::get_default();
    assert!(
        !processors.is_empty(),
        "get_default should return at least one processor"
    );
}

#[test]
fn test_postprocessor_filter_default() {
    use wildmatch::WildMatchPattern;

    let processors = PostProcessor::get_default();
    let filter = vec![WildMatchPattern::new("nonexistent")];
    let filtered = PostProcessor::filter_default(filter);
    assert_eq!(filtered.len(), processors.len());
}

#[test]
fn test_postprocessor_filter_default_removes_matching() {
    use wildmatch::WildMatchPattern;

    let filter = vec![WildMatchPattern::new("bshchk")];
    let filtered = PostProcessor::filter_default(filter);
    let bshchk_exists = filtered.iter().any(|pp| pp.name == "bshchk");
    assert!(!bshchk_exists, "bshchk should be filtered out");
}

#[test]
fn test_each_installed_postprocessor() {
    let hello = "echo \"Hello world!\"";
    let hello = compile_code(hello);

    let processors = PostProcessor::get_default();
    for processor in processors {
        if processor.is_available() {
            let res = processor.execute(hello.clone());
            assert!(
                res.is_ok(),
                "Postprocessor {} couldn't process hello world",
                processor.name
            );
            let res = res.unwrap();
            let (stdout, _) = eval_bash(res);
            assert_eq!(stdout, "Hello world!");
        }
    }
}
