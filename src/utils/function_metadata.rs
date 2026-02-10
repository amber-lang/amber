use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::raw_fragment;
use crate::utils::is_all_caps;

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
        FunctionMetadata {
            name,
            id,
            variant,
            returns,
        }
    }

    pub fn mangled_name(&self) -> String {
        if is_all_caps(&self.name) {
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
