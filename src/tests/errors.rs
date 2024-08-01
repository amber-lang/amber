use super::test_amber;

#[test]
#[should_panic(expected = "ERROR: Return type does not match function return type")]
fn function_with_wrong_typed_return() {
    let code = r#"
        pub fun test(): Num {
            return "Hello, World!"
        }
        echo test()
    "#;

    test_amber(code, "Hello, World!");
}

#[test]
#[should_panic(expected = "ERROR: Failable functions must return a Failable type")]
fn function_failable_with_typed_nonfailable_return() {
    let code = r#"
        pub fun test(): Null {
            fail 1
        }
        echo test() failed: echo "Failed"
    "#;

    test_amber(code, "Failed");
}

#[test]
#[should_panic(expected = "ERROR: Non-failable functions cannot return a Failable type")]
fn function_nonfailable_with_typed_failable_return() {
    let code = r#"
        pub fun test(): Null? {
            echo "Hello, World!"
        }
        echo test() failed: echo "Failed"
    "#;

    test_amber(code, "Hello, World!");
}

#[test]
#[should_panic(expected = "ERROR: Failable types cannot be used as arguments")]
fn function_with_failable_typed_arg() {
    let code = r#"
        pub fun test(a: Text?) {
            echo a
        }
        test("Hello, World!")
    "#;

    test_amber(code, "Hello, World!");
}
