use heraclitus_compiler::prelude::*;
use std::collections::{HashMap, BTreeSet};
use crate::modules::{Type, block::Block};

use super::function_map::{FunctionMap, FunctionInstance};

// TODO: Change (args, returns) to function descriptor

#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub returns: Type,
    pub body: Vec<Token>,
    pub typed: bool,
    pub id: usize
}

#[derive(Clone, Debug)]
pub struct VariableDecl {
    pub name: String,
    pub kind: Type
}

#[derive(Clone, Debug)]
pub struct ScopeUnit {
    pub vars: HashMap<String, VariableDecl>,
    pub funs: HashMap<String, FunctionDecl>
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
    scopes: Vec<ScopeUnit>,
    // Map of all generated functions based on their invocations
    function_map: FunctionMap
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            scopes: vec![],
            function_map: FunctionMap::new()
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(ScopeUnit::new())
    }

    pub fn pop_scope(&mut self) -> Option<ScopeUnit> {
        self.scopes.pop()
    }

    pub fn add_variable(&mut self, name: &str, kind: Type) -> bool {
        if self.get_function(&name).is_some() {
            return false;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.vars.insert(name.to_string(), VariableDecl { name: name.to_string(), kind }).is_none()
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableDecl> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
                return Some(var);
            }
        }
        None
    }

    pub fn get_available_variables(&self) -> BTreeSet<&String> {
        let mut set = BTreeSet::new();
        for scope in self.scopes.iter().rev() {
            for name in scope.vars.keys() {
                set.insert(name);
            }
        }
        set
    }

    pub fn add_function(&mut self, name: &str, args: &[(String, Type)], returns: Type, body: Vec<Token>) -> Option<usize> {
        // Make sure that there is no variable with the same name
        if self.get_variable(&name).is_some() {
            return None;
        }
        let typed = !args.iter().any(|(_, kind)| kind == &Type::Generic);
        let scope = self.scopes.last_mut().unwrap();
        // Add function declaration
        let id = self.function_map.add_declaration();
        let success = scope.funs.insert(name.to_string(), FunctionDecl {
            name: name.to_string(),
            args: args.to_vec(),
            returns: returns.clone(),
            body,
            typed,
            id
        });
        // If this is a new function, return its id
        if success.is_none() {
            Some(id)
        }
        // If we are having a conflict
        else {
            None
        }
    }

    pub fn add_function_instance(&mut self, id: usize, args: &[Type], returns: Type, body: Block) -> usize {
        self.function_map.add_instance(id, FunctionInstance {
            args: args.to_vec(),
            returns,
            body
        })
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDecl> {
        for scope in self.scopes.iter().rev() {
            if let Some(fun) = scope.funs.get(name) {
                return Some(fun);
            }
        }
        None
    }

    pub fn get_function_instances(&self, id: usize) -> Option<&Vec<FunctionInstance>> {
        self.function_map.get(id)
    }

    pub fn get_available_functions(&self) -> BTreeSet<&String> {
        let mut set = BTreeSet::new();
        for scope in self.scopes.iter().rev() {
            for name in scope.funs.keys() {
                set.insert(name);
            }
        }
        set
    }
}