use std::collections::{HashMap, BTreeSet};
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
            if let Some(var) = scope.vars.get(name.as_ref()) {
                return Some(var);
            }
        }
        None
    }

    pub fn get_available_variables(&mut self) -> BTreeSet<&String> {
    let mut set = BTreeSet::new();
        for scope in self.mem.iter().rev() {
            for name in scope.vars.keys() {
                set.insert(name);
            }
        }
        set
    }
}