use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::tests::eval_bash;

#[test]
fn test_test_mode() {
    let code = r#"
        import { assert } from "std/test"
        test {
            echo "Test executed"
            assert(true)?
        }
        main {
            echo "Main executed"
        }
    "#;
    
    let options = CompilerOptions::from_args(&[], false, true, None);
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let (messages, bash_code) = compiler.compile().unwrap();
    assert!(messages.is_empty());
    
    let (stdout, stderr) = eval_bash(bash_code);
    assert_eq!(stdout, "Test executed");
    assert_eq!(stderr, "");
}

#[test]
fn test_default_mode_skips_test() {
    let code = r#"
        import { assert } from "std/test"
        test {
            echo "Test executed"
            assert(false)?
        }
        main {
            echo "Main executed"
        }
    "#;
    
    let options = CompilerOptions::from_args(&[], false, false, None);
    let compiler = AmberCompiler::new(code.to_string(), None, options);
    let (messages, bash_code) = compiler.compile().unwrap();
    assert!(messages.is_empty());
    
    let (stdout, stderr) = eval_bash(bash_code);
    assert_eq!(stdout, "Main executed");
    assert_eq!(stderr, "");
}
