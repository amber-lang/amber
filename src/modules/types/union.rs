use super::Type;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UnionType(Vec<Type>);

impl Into<Vec<Type>> for UnionType {
    fn into(self) -> Vec<Type> {
        self.0
    }
}

impl <'a> Into<&'a Vec<Type>> for &'a UnionType {
    fn into(self) -> &'a Vec<Type> {
        &self.0
    }
}

impl From<Vec<Type>> for UnionType {
    fn from(value: Vec<Type>) -> Self {
        let mut value = value;
        value.sort();
        if value.len() < 2 {
            unreachable!("A union type must have at least two elements")
        }
        
        Self(value)
    }
}
