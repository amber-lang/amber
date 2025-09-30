use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::raw_fragment;

#[derive(Clone)]
pub struct FunctionMetadata {
    name: String,
    id: usize,
    variant: usize,
    returns: Type,
    is_all_caps: bool,
}

impl FunctionMetadata {
    pub fn new<T: Into<String>>(name: T, id: usize, variant: usize, returns: &Type, is_all_caps: bool) -> Self {
        let name = name.into();
        let returns = returns.clone();
        FunctionMetadata { name, id, variant, returns, is_all_caps }
    }

    pub fn mangled_name(&self) -> String {
        if self.is_all_caps {
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
