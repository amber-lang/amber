use crate::compiler::AmberCompiler;
use crate::Cli;
use crate::test_amber;

#[test]
#[should_panic(expected = "ERROR: Return type does not match function return type")]
fn function_with_wrong_typed_return() {
    let code = r#"
        pub fun test(): Num {
            return "Hello, World!"
        }
        echo test()
    "#;

    test_amber!(code, "Hello, World!");
}
