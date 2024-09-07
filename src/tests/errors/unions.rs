use crate::tests::test_amber;

#[test]
#[should_panic(expected = "ERROR: 1st argument 'param' of function 'abc' expects type 'Text | Null', but 'Num' was given")]
fn invalid_union_type_eq_normal_type() {
    let code = r#"
        fun abc(param: Text | Null) {}
        abc("")
        abc(123)
    "#;
    test_amber(code, "");
}

#[test]
#[should_panic(expected = "ERROR: 1st argument 'param' of function 'abc' expects type 'Text | Null', but 'Num | [Text]' was given")]
fn invalid_two_unions() {
    let code = r#"
        fun abc(param: Text | Null) {}
        abc(123 as Num | [Text])
    "#;
    test_amber(code, "");
}

#[test]
#[should_panic(expected = "ERROR: 1st argument 'param' of function 'abc' expects type 'Text | Num | Text? | Num? | [Null]', but 'Null' was given")]
fn big_union() {
    let code = r#"
        fun abc(param: Text | Num | Text? | Num? | [Null]) {}
        abc(null)
    "#;
    test_amber(code, "");
}