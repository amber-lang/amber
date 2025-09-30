use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::raw_fragment;

// Helper function to check if a name is all uppercase
fn is_all_uppercase(name: &str) -> bool {
    name.chars()
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_uppercase())
}

#[derive(Clone)]
pub struct FunctionMetadata {
    name: String,
    id: usize,
    variant: usize,
    returns: Type,
}

impl FunctionMetadata {
    pub fn new<T: Into<String>>(name: T, id: usize, variant: usize, returns: &Type) -> Self {
        let name = name.into();
        let returns = returns.clone();
        FunctionMetadata { name, id, variant, returns }
    }

    pub fn mangled_name(&self) -> String {
        if is_all_uppercase(&self.name) {
            format!("__ret_{}{}_v{}", self.name, self.id, self.variant)
        } else {
            format!("ret_{}{}_v{}", self.name, self.id, self.variant)
        }
    }

    pub fn get_type(&self) -> Type {
        self.returns.clone()
    }

    pub fn default_return(&self) -> FragmentKind {
        if self.returns.is_array() {
            raw_fragment!("")
        } else {
            raw_fragment!("''")
        }
    }
}
