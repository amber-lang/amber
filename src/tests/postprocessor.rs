
use crate::compiler::postprocess::PostProcessor;

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
        assert!(processor.execute(hello.clone()).is_ok(), "Postprocessor {} couldn't process hello world", processor.name)
    }
}
