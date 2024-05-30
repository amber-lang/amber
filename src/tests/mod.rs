pub mod validity;
pub mod stdlib;

#[macro_export]
macro_rules! test_amber {
    ($code:expr, $result:expr) => {
        {
            match AmberCompiler::new($code.to_string(), None).test_eval() {
                Ok(result) => assert_eq!(result.trim_end_matches('\n'), $result),
                Err(err) => panic!("ERROR: {}", err.message.unwrap())
            }

        }
    };
}
