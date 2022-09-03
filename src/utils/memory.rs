use heraclitus_compiler::prelude::*;
use std::collections::{HashMap, BTreeSet};
use crate::modules::Type;

#[derive(Clone, Debug)]
pub struct FunctionUnit {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub returns: Type,
    pub body: Vec<Token>,
    pub typed: bool
}

#[derive(Clone, Debug)]
pub struct VariableUnit {
    pub name: String,
    pub kind: Type
}

#[derive(Clone, Debug)]
pub struct ScopeUnit {
    pub vars: HashMap<String, VariableUnit>,
    pub funs: HashMap<String, FunctionUnit>
}

impl ScopeUnit {
    fn new() -> ScopeUnit {
        ScopeUnit {
            vars: HashMap::new(),
            funs: HashMap::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Memory {
    mem: Vec<ScopeUnit>
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            mem: vec![]
        }
    }

    pub fn push_scope(&mut self) {
        self.mem.push(ScopeUnit::new())
    }

    pub fn pop_scope(&mut self) -> Option<ScopeUnit> {
        self.mem.pop()
    }

    pub fn add_variable(&mut self, name: &str, kind: Type) -> bool {
        if self.get_function(&name).is_some() {
            return false;
        }
        let scope = self.mem.last_mut().unwrap();
        scope.vars.insert(name.to_string(), VariableUnit { name: name.to_string(), kind }).is_none()
    }

    pub fn get_variable(&mut self, name: &str) -> Option<&VariableUnit> {
        for scope in self.mem.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
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

    pub fn add_function(&mut self, name: &str, args: &[(String, Type)], returns: Type, body: Vec<Token>) -> bool {
        if self.get_variable(&name).is_some() {
            return false;
        }
        let typed = !args.iter().any(|(_, kind)| kind == &Type::Generic);
        let scope = self.mem.last_mut().unwrap();
        scope.funs.insert(name.to_string(), FunctionUnit {
            name: name.to_string(),
            args: args.to_vec(),
            returns: returns.clone(),
            body,
            typed
        }).is_none()
    }

    pub fn get_function(&mut self, name: &str) -> Option<&FunctionUnit> {
        for scope in self.mem.iter().rev() {
            if let Some(fun) = scope.funs.get(name) {
                return Some(fun);
            }
        }
        None
    }

    pub fn get_available_functions(&mut self) -> BTreeSet<&String> {
        let mut set = BTreeSet::new();
        for scope in self.mem.iter().rev() {
            for name in scope.funs.keys() {
                set.insert(name);
            }
        }
        set
    }
}