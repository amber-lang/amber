use crate::compiler::AmberCompiler;
use crate::Cli;

pub mod cli;
pub mod formatter;
pub mod errors;
pub mod stdlib;
pub mod validity;

#[macro_export]
macro_rules! test_amber {
    ($code:expr, $result:expr) => {{
        match AmberCompiler::new($code.to_string(), None, Cli::default()).test_eval() {
            Ok(result) => assert_eq!(result.trim_end_matches('\n'), $result),
            Err(err) => panic!("ERROR: {}", err.message.unwrap()),
        }
    }};
}

pub fn compile_code<T: Into<String>>(code: T) -> String {
    AmberCompiler::new(code.into(), None, Cli::default()).compile().unwrap().1
}
