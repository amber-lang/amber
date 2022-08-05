use std::collections::HashMap;
use crate::modules::Type;

pub struct VariableUnit {
    pub name: String,
    pub kind: Type
}

pub struct ScopeUnit {
    pub vars: HashMap<String, VariableUnit>
}

impl ScopeUnit {
    fn new() -> ScopeUnit {
        ScopeUnit {
            vars: HashMap::new()
        }
    }
}


pub struct VariableMemory {
    mem: Vec<ScopeUnit>
}

impl VariableMemory {
    pub fn new() -> VariableMemory {
        VariableMemory {
            mem: vec![]
        }
    }

    pub fn push_scope(&mut self) {
        self.mem.push(ScopeUnit::new())
    }

    pub fn pop_scope(&mut self) -> Option<ScopeUnit> {
        self.mem.pop()
    }

    pub fn add_variable(&mut self, name: String, kind: Type) -> bool {
        let scope = self.mem.last_mut().unwrap();
        scope.vars.insert(name.clone(), VariableUnit { name, kind }).is_none()
    }

    pub fn get_variable(&mut self, name: impl AsRef<str>) -> Option<&VariableUnit> {
        for scope in self.mem.iter().rev() {
            match scope.vars.get(name.as_ref()) {
                Some(unit) => return Some(unit),
                None => {}
            }
        }
        None
    }

    pub fn has_variable(&mut self, name: impl AsRef<str>) -> bool {
        self.get_variable(name).is_some()
    }
}