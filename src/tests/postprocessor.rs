use crate::compiler::postprocessor::PostProcessor;
use crate::tests::eval_bash;

use super::compile_code;

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
