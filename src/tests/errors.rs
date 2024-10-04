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

#[test]
#[should_panic(expected = "ERROR: Variable declared but never assigned!")]
fn declared_but_not_assigned_var() {
    let code = r#"
        let declared_not_assigned: Text
    "#;

    test_amber(code, "");
}

#[test]
#[should_panic(expected = "Variable non_assigned_var accessed before it is assigned a value!")]
fn declared_accessed_before_assigned() {
    let code = r#"
        let non_assigned_var: Text
        echo non_assigned_var
    "#;

    test_amber(code, "");
}


#[test]
#[should_panic(expected = "ERROR: Cannot assign value of type 'Num' to a variable of type 'Text'")]
fn declared_assigned_invalid_type() {
    let code = r#"
        let invalid_type_var: Text
        invalid_type_var = 123
    "#;

    test_amber(code, "");
}
