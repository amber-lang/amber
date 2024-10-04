use crate::modules::expression::expr::Expr;
use crate::modules::types::Type;
use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex, MutexGuard, PoisonError}};

use super::{cc_flags::CCFlags, function_interface::FunctionInterface};

#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub arg_names: Vec<String>,
    pub arg_types: Vec<Type>,
    pub arg_refs: Vec<bool>,
    pub arg_optionals: Vec<Expr>,
    pub returns: Type,
    pub is_args_typed: bool,
    pub is_public: bool,
    pub is_failable: bool,
    pub id: usize,
}

impl FunctionDecl {
    pub fn into_interface(self) -> FunctionInterface {
        FunctionInterface {
            id: Some(self.id),
            name: self.name,
            arg_names: self.arg_names,
            arg_types: self.arg_types,
            arg_refs: self.arg_refs,
            arg_optionals: self.arg_optionals,
            returns: self.returns,
            is_public: self.is_public,
            is_failable: self.is_failable,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VariableDecl {
    pub name: String,
    pub kind: Type,
    pub global_id: Option<usize>,
    pub is_ref: bool,
    pub is_empty: Arc<Mutex<bool>>,
    pub declared_at: PositionInfo,
}

impl VariableDecl {
    pub fn set_is_empty(&self, is_empty: bool) -> Result<(), ()> {
        if let Ok(mut lock) = self.is_empty.lock() {
            *lock = is_empty;
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn is_empty<'a>(&'a self) -> Result<bool, PoisonError<MutexGuard<'a, bool>>> {
        Ok(*(self.is_empty.lock()?))
    }
}

#[derive(Clone, Debug)]
pub struct ScopeUnit {
    pub vars: HashMap<String, VariableDecl>,
    pub funs: HashMap<String, FunctionDecl>,
}

/// Perform methods just on the scope
impl ScopeUnit {
    pub fn new() -> ScopeUnit {
        ScopeUnit {
            vars: HashMap::new(),
            funs: HashMap::new(),
        }
    }

    /* Variables */

    /// Persists a variable declaration in the scope
    pub fn add_var(&mut self, var: VariableDecl) {
        self.vars.insert(var.name.clone(), var);
    }

    /// Fetches a variable declaration from the scope
    pub fn get_var(&self, name: &str) -> Option<&VariableDecl> {
        self.vars.get(name)
    }

    /// Gets all the variable names in the scope
    pub fn get_var_names(&self) -> Vec<&String> {
        self.vars.keys().collect()
    }

    pub fn get_all_vars(&self) -> Vec<&VariableDecl> {
        self.vars.values().collect_vec()
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
    /// Determines if the context is in the main block
    pub is_main_ctx: bool,
    /// Determines if the context is in a trust block
    pub is_trust_ctx: bool,
    /// This is a list of ids of all the public functions in the file
    pub pub_funs: Vec<FunctionDecl>,
    /// The return type of the currently parsed function
    pub fun_ret_type: Option<Type>,
    /// List of compiler flags
    pub cc_flags: HashSet<CCFlags>,
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
            is_main_ctx: false,
            is_trust_ctx: false,
            pub_funs: vec![],
            fun_ret_type: None,
            cc_flags: HashSet::new(),
        }
    }

    pub fn function_invocation(mut self, expr: Vec<Token>) -> Self {
        self.is_fun_ctx = true;
        self.index = 0;
        self.expr = expr;
        self
    }

    pub fn file_import(mut self, trace: &[PositionInfo], position: PositionInfo) -> Self {
        // Initialize the trace
        self.trace = trace.to_vec();
        // Push the position to the trace
        self.trace.push(position);
        self
    }
}
