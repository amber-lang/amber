use crate::modules::types::Type;

#[test]
fn two_normal_types() {
    assert_eq!(Type::Bool, Type::Bool);
}

#[test]
fn two_different_normal_types() {
    assert_ne!(Type::Null, Type::Bool);
}

#[test]
fn normal_and_failable_type() {
    assert_ne!(Type::Failable(Box::new(Type::Text)), Type::Text, "Text? and Text must not be equal!")
}