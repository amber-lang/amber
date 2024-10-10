use crate::modules::types::Type;

#[derive(Clone)]
pub struct FunctionName {
    name: String,
    id: usize,
    variant: usize,
    returns: Type,
}

impl FunctionName {
    pub fn new(name: &str, id: usize, variant: usize, returns: &Type) -> Self {
        let name = String::from(name);
        let returns = returns.clone();
        FunctionName { name, id, variant, returns }
    }

    pub fn mangled_name(&self) -> String {
        format!("__AF_{}{}_v{}", self.name, self.id, self.variant)
    }

    pub fn default_value(&self) -> &'static str {
        if self.returns.is_array() {
            "()"
        } else {
            "''"
        }
    }
}
