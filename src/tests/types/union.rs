use crate::modules::types::Type;

#[test]
fn partially_overlapping_types() {
    let one = Type::Union(vec![Type::Text, Type::Num]);
    let two = Type::Union(vec![Type::Num, Type::Null]);

    assert_ne!(one, two, "Text | Num must not be equal to Num | Null!")
}

#[test]
fn overlapping_types() {
    let one = Type::Union(vec![Type::Text, Type::Num]);
    let two = Type::Union(vec![Type::Text, Type::Num, Type::Null]);

    assert_eq!(one, two, "Text | Num must be equal to Text | Num | Null!")
}

#[test]
fn same_union() {
    let one = Type::Union(vec![Type::Text, Type::Num]);
    let two = Type::Union(vec![Type::Text, Type::Num]);

    assert_eq!(one, two, "Text | Num must be equal to Text | Num!")
}

#[test]
fn empty_union() {
    let one = Type::Union(vec![]);
    let two = Type::Union(vec![]);

    assert_eq!(one, two, "If one of unions is empty, it must always be equal to another")
}

#[test]
fn empty_and_normal_union() {
    let one = Type::Union(vec![Type::Text, Type::Num]);
    let two = Type::Union(vec![]);

    assert_eq!(one, two, "If one of unions is empty, it must always be equal to another")
}

#[test]
fn empty_union_and_normal_type() {
    let one = Type::Union(vec![]);
    let two = Type::Text;

    assert_ne!(one, two, "An empty union and one type are not equal")
}

#[test]
fn big_union() {
    let one = Type::Union(vec![Type::Text, Type::Text, Type::Text, Type::Text, Type::Text, Type::Text, Type::Text, Type::Num]);
    let two = Type::Union(vec![Type::Text, Type::Num]);

    assert_eq!(one, two, "Text | Text | ... | Text | Num and Text | Num must be equal!")
}

#[test]
fn normal_and_union() {
    let one = Type::Text;
    let two = Type::Union(vec![Type::Text, Type::Null]);

    assert_eq!(one, two, "Text and Text | Null must be equal!");
}

#[test]
fn normal_not_in_union() {
    let one = Type::Text;
    let two = Type::Union(vec![Type::Num, Type::Null]);

    assert_ne!(one, two, "Text and Num | Null must not be equal!");
}
