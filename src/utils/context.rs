use heraclitus_compiler::prelude::*;
use std::collections::HashMap;
use crate::modules::types::Type;

use super::function_interface::FunctionInterface;

#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub arg_names: Vec<String>,
    pub arg_types: Vec<Type>,
    pub returns: Type,
    pub is_args_typed: bool,
    pub is_public: bool,
    pub id: usize
}

impl FunctionDecl {
    pub fn to_interface(self) -> FunctionInterface {
        FunctionInterface {
            id: Some(self.id),
            name: self.name,
            arg_names: self.arg_names,
            arg_types: self.arg_types,
            returns: self.returns,
            is_public: self.is_public
        }
    }
}

#[derive(Clone, Debug)]
pub struct VariableDecl {
    pub name: String,
    pub kind: Type,
    pub global_id: Option<usize>
}

#[derive(Clone, Debug)]
pub struct ScopeUnit {
    pub vars: HashMap<String, VariableDecl>,
    pub funs: HashMap<String, FunctionDecl>
}

/// Perform methods just on the scope
impl ScopeUnit {
    pub fn new() -> ScopeUnit {
        ScopeUnit {
            vars: HashMap::new(),
            funs: HashMap::new()
        }
    }

    /* Variables */

    /// Persists a variable declaration in the scope
    pub fn add_var(&mut self, name: &str, kind: Type, global_id: Option<usize>) {
        self.vars.insert(name.to_string(), VariableDecl {
            name: name.to_string(),
            kind,
            global_id
        });
    }

    /// Fetches a variable declaration from the scope
    pub fn get_var(&self, name: &str) -> Option<&VariableDecl> {
        self.vars.get(name)
    }

    /// Gets all the variable names in the scope
    pub fn get_var_names(&self) -> Vec<&String> {
        self.vars.keys().collect()
    }

    /* Functions */

    /// Persists a function declaration in the scope
    pub fn add_fun(&mut self, fun: FunctionDecl) -> bool {
        let name = fun.name.clone();
        self.funs.insert(name, fun).is_none()
    }

    /// Fetches a function declaration from the scope
    pub fn get_fun(&self, name: &str) -> Option<&FunctionDecl> {
        self.funs.get(name)
    }

    /// Gets all the function names in the scope
    pub fn get_fun_names(&self) -> Vec<&String> {
        self.funs.keys().collect()
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    /// The current index in the expression
    pub index: usize,
    /// The expression to be parsed
    pub expr: Vec<Token>,
    /// The path of the file
    pub path: Option<String>,
    /// Scopes saved in the context
    pub scopes: Vec<ScopeUnit>,
    /// A trace of the current position in the file
    pub trace: Vec<PositionInfo>,
    /// Determines if the context is in a function
    pub is_fun_ctx: bool,
    /// Determines if the context is in a loop
    pub is_loop_ctx: bool,
    /// This is a list of ids of all the public functions in the file
    pub pub_funs: Vec<FunctionDecl>
}

// FIXME: Move the scope related structures to the separate file
impl Context {
    pub fn new(path: Option<String>, expr: Vec<Token>) -> Self {
        Self {
            index: 0,
            expr,
            path,
            scopes: vec![],
            trace: vec![],
            is_fun_ctx: false,
            is_loop_ctx: false,
            pub_funs: vec![]
        }
    }

    pub fn function_invocation(mut self, expr: Vec<Token>) -> Self {
        self.is_fun_ctx = true;
        self.index = 0;
        self.expr = expr;
        self
    }

    pub fn file_import(mut self, trace: &Vec<PositionInfo>, position: PositionInfo) -> Self {
        // Initialize the trace
        self.trace = trace.clone();
        // Push the position to the trace
        self.trace.push(position);
        self
    }
}