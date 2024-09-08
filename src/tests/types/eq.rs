use crate::modules::types::Type;

#[test]
fn normal_types_eq() {
    let types = vec![Type::Null, Type::Text, Type::Bool, Type::Num, Type::Generic];
    for typ in types {
        assert_eq!(typ, typ, "{typ} and {typ} must be equal!");
    }
}

#[test]
fn two_different_normal_types() {
    assert_ne!(Type::Null, Type::Bool);
}

#[test]
fn normal_and_failable_type() {
    assert_ne!(Type::Failable(Box::new(Type::Text)), Type::Text, "Text? and Text must not be equal!")
}

#[test]
fn array_and_normal_type() {
    assert_ne!(Type::Array(Box::new(Type::Bool)), Type::Bool);
}

#[test]
fn array_and_array_of_failables() {
    assert_ne!(Type::Array(Box::new(Type::Bool)), Type::Array(Box::new(Type::Failable(Box::new(Type::Bool)))));
}

#[test]
fn nested_array_normal_array_with_failable() {
    assert_ne!(Type::Array(Box::new(Type::Array(Box::new(Type::Bool)))), Type::Failable(Box::new(Type::Bool)));
}