use super::Type;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UnionType(pub Vec<Type>);

impl UnionType {
    pub fn has(&self, other: &Type) -> bool {
        self.0.iter().find(|x| **x == *other).is_some()
    }
}

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
        for typ in &value {
            if let Type::Union(_) = typ {
                unreachable!("Union types cannot be nested")
            }
        }
        
        Self(value)
    }
}
