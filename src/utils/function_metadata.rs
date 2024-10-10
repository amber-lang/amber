use crate::modules::types::Type;

#[derive(Clone)]
pub struct FunctionMetadata {
    name: String,
    id: usize,
    variant: usize,
    returns: Type,
}

impl FunctionMetadata {
    pub fn new(name: &str, id: usize, variant: usize, returns: &Type) -> Self {
        let name = String::from(name);
        let returns = returns.clone();
        FunctionMetadata { name, id, variant, returns }
    }

    pub fn mangled_name(&self) -> String {
        format!("__AF_{}{}_v{}", self.name, self.id, self.variant)
    }

    pub fn default_return(&self) -> &'static str {
        if self.returns.is_array() {
            "()"
        } else {
            "''"
        }
    }
}
