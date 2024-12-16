use std::collections::BTreeSet;

use heraclitus_compiler::prelude::*;
use amber_meta::ContextManager;
use crate::modules::block::Block;
use crate::modules::expression::expr::Expr;
use crate::modules::types::Type;
use crate::utils::context::{Context, ScopeUnit, VariableDecl, FunctionDecl};
use crate::utils::function_interface::FunctionInterface;
use crate::utils::import_cache::ImportCache;
use crate::utils::function_cache::FunctionCache;
use crate::utils::payload::Payload;

#[derive(Debug, ContextManager)]
pub struct ParserMetadata {
    /// Code if the parser is in eval mode
    pub eval_code: Option<String>,
    /// Used for debugging by Heraclitus
    pub debug: Option<usize>,
    /// Cache of already imported modules
    pub import_cache: ImportCache,
    /// Cache of already parsed functions
    pub fun_cache: FunctionCache,
    /// Global function id
    pub fun_id: usize,
    /// Global variable id
    pub var_id: usize,
    /// Context of the parser
    #[context]
    pub context: Context,
    /// List of all failure messages
    pub messages: Vec<Message>,
    /// Show standard library usage in documentation
    pub doc_usage: bool,
}

impl ParserMetadata {
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}

// Implement context methods
impl ParserMetadata {
    /* Scopes */

    /// Determines if the parser is in the global scope
    pub fn is_global_scope(&self) -> bool {
        self.context.scopes.len() == 1
    }

    /// Pushes a new scope to the stack
    pub fn with_push_scope<B>(&mut self, mut body: B) -> SyntaxResult
    where
        B: FnMut(&mut Self) -> SyntaxResult
    {
        self.context.scopes.push(ScopeUnit::new());
        let result = body(self);
        self.context.scopes.pop();
        result
    }

    /* Variables */

    /// Generate a new global variable id
    pub fn gen_var_id(&mut self) -> usize {
        let id = self.var_id;
        self.var_id += 1;
        id
    }

    /// Adds a variable to the current scope
    pub fn add_var(
        &mut self,
        name: &str,
        kind: Type,
        payload: Option<Payload>,
        is_const: bool,
    ) -> Option<usize> {
        let global_id = (self.is_global_scope() || self.is_shadowing_prev_scope(name)).then(|| self.gen_var_id());
        let scope = self.context.scopes.last_mut().unwrap();
        let var = VariableDecl {
            name: name.to_string(),
            kind,
            payload,
            global_id,
            is_ref: false,
            is_const,
        };
        scope.add_var(var);
        global_id
    }

    /// Adds a function parameter as variable to the current scope
    pub fn add_param(&mut self, name: &str, kind: Type, is_ref: bool) -> Option<usize> {
        let global_id = self.is_global_scope().then(|| self.gen_var_id());
        let scope = self.context.scopes.last_mut().unwrap();
        let var = VariableDecl {
            name: name.to_string(),
            kind,
            payload: None,
            global_id,
            is_ref,
            is_const: false,
        };
        scope.add_var(var);
        global_id
    }

    pub fn is_shadowing_prev_scope(&self, name: &str) -> bool {
        if self.context.scopes.len() > 1 {
            self.context.scopes.iter()
                .rev()
                .skip(1)
                .find_map(|scope| scope.get_var(name))
                .is_some()
        } else {
            false
        }
    }

    /// Gets a variable from the current scope or any parent scope
    pub fn get_var(&self, name: &str) -> Option<&VariableDecl> {
        self.context.scopes.iter().rev().find_map(|scope| scope.get_var(name))
    }

    /// Gets a variable from the current scope or any parent scope
    pub fn get_var_from_expr(&self, expr: &Expr) -> Option<&VariableDecl> {
        expr.get_original_name().and_then(|name| self.get_var(&name))
    }

    /// Gets variable names
    pub fn get_var_names(&self) -> BTreeSet<&String> {
        self.context.scopes.iter().rev().flat_map(|scope| scope.get_var_names()).collect()
    }

    /* Functions */

    /// Generate a new global function id
    pub fn gen_fun_id(&mut self) -> usize {
        let id = self.fun_id;
        self.fun_id += 1;
        id
    }

    /// Adds a function declaration to the current scope
    pub fn add_fun_declaration(&mut self, fun: FunctionInterface, ctx: Context) -> Option<usize> {
        let global_id = self.gen_fun_id();
        // Add the function to the public function list
        if fun.is_public {
            let decl = fun.clone().into_fun_declaration(global_id);
            self.context.pub_funs.push(decl);
        }
        // Add the function to the current scope
        let scope = self.context.scopes.last_mut().unwrap();
        scope.add_fun(fun.into_fun_declaration(global_id)).then(|| {
            // Add the function to the function cache
            self.fun_cache.add_declaration(global_id, ctx);
            global_id
        })
    }

    /// Adds a function declaration that that was already parsed - this function is probably imported
    pub fn add_fun_declaration_existing(&mut self, fun: FunctionDecl) -> Option<usize> {
        let global_id = self.gen_fun_id();
        // Add the function to the public function list
        if fun.is_public {
            self.context.pub_funs.push(fun.clone());
        }
        // Add the function to the current scope
        let scope = self.context.scopes.last_mut().unwrap();
        scope.add_fun(fun).then_some(global_id)
    }

    /// Adds a function instance to the cache
    /// This function returns the id of the function instance variant
    pub fn add_fun_instance(&mut self, fun: FunctionInterface, block: Block) -> usize {
        let id = fun.id.expect("Function id is not set");
        self.fun_cache.add_instance(id, fun.into_fun_instance(block))
    }

    /// Gets a function declaration from the current scope or any parent scope
    pub fn get_fun_declaration(&self, name: &str) -> Option<&FunctionDecl> {
        self.context.scopes.iter().rev().find_map(|scope| scope.get_fun(name))
    }

    /// Gets function names
    pub fn get_fun_names(&self) -> BTreeSet<&String> {
        self.context.scopes.iter().rev().flat_map(|scope| scope.get_fun_names()).collect()
    }
}

impl Metadata for ParserMetadata {
    fn new(tokens: Vec<Token>, path: Option<String>, code: Option<String>) -> Self {
        ParserMetadata {
            eval_code: code,
            debug: None,
            import_cache: ImportCache::new(path.clone()),
            fun_cache: FunctionCache::new(),
            fun_id: 0,
            var_id: 0,
            context: Context::new(path, tokens),
            messages: Vec::new(),
            doc_usage: false,
        }
    }

    fn get_token_at(&self, index: usize) -> Option<Token> {
        self.context.expr.get(index).cloned()
    }

    fn get_index(&self) -> usize {
        self.context.index
    }

    fn set_index(&mut self, index: usize) {
        self.context.index = index
    }

    fn get_debug(&mut self) -> Option<usize> {
        self.debug
    }

    fn set_debug(&mut self, indent: usize) {
        self.debug = Some(indent)
    }

    fn get_path(&self) -> Option<String> {
        self.context.path.clone()
    }

    fn get_code(&self) -> Option<&String> {
        self.eval_code.as_ref()
    }

    fn get_trace(&self) -> Vec<PositionInfo> {
        self.context.trace.clone()
    }
}
