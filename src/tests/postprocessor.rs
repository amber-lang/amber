
use crate::{compiler::postprocess::PostProcessor, tests::eval_bash};

use super::compile_code;

#[test]
fn default_ok() {
    let default = PostProcessor::get_default();
    
    let hello = "echo \"Hello world!\"";
    let hello = compile_code(hello);

    let mut unavailable = vec![];

    for processor in &default {
        if ! processor.is_available() {
            unavailable.push(processor.name.clone());
        }
    }

    assert!(unavailable.len() == 0, "These commands have to be in $PATH for this test to pass: {}", unavailable.join(", "));

    for processor in default {
        let res = processor.execute(hello.clone());
        assert!(res.is_ok(), "Postprocessor {} couldn't process hello world", processor.name);
        let res = res.unwrap();
        let (stdout, _) = eval_bash(res);
        assert_eq!(stdout, "Hello world!");
    }
}
