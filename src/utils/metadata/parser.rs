use std::collections::{BTreeSet, HashMap};

use crate::modules::block::Block;
use crate::modules::types::Type;
use crate::utils::context::{Context, FunctionDecl, ScopeUnit, VariableDecl};
use crate::utils::function_cache::FunctionCache;
use crate::utils::function_interface::FunctionInterface;
use crate::utils::import_cache::ImportCache;
use amber_meta::ContextManager;
use heraclitus_compiler::prelude::*;

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
    /// List of functions that are currently being parsed
    pub parsing_functions: HashMap<(usize, Vec<Type>), usize>,
    /// List of test names found in the file
    pub test_names: Vec<String>,
    /// Stack of narrowed types for control flow analysis
    pub narrowed_types: Vec<HashMap<String, Type>>,
    /// Suppress warnings during monomorphic function re-typechecking
    #[context]
    pub suppress_warnings: bool,
    /// Skip persisting function instances during first-pass typechecking
    /// First pass is used for typechecking a function with declared and not concrete types
    /// This is used to generally assess if a function is valid and emit errors if it is not
    #[context]
    pub first_pass_ctx: bool,
    /// Whether sudo modifier is used anywhere in the code
    pub sudo_used: bool,
}

impl ParserMetadata {
    pub fn add_message(&mut self, message: Message) {
        // Skip warnings if we're in a suppressed context
        if self.suppress_warnings && matches!(message.kind, MessageType::Warning) {
            return;
        }
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
    pub fn with_push_scope<B>(&mut self, predicate: bool, mut body: B) -> SyntaxResult
    where
        B: FnMut(&mut Self) -> SyntaxResult,
    {
        if predicate {
            self.context.scopes.push(ScopeUnit::new());
        }
        let result = body(self);
        if predicate {
            let scope = self.context.scopes.pop().unwrap();
            // Check for unused variables and const correctness
            for (_, mut var) in scope.vars {
                if let Some(warn) = var.warn.as_mut() {
                    if warn.on_unused && !var.is_used && !var.name.starts_with('_') {
                        let message = Message::new_warn_at_position(self, warn.pos.take().unwrap())
                            .message(format!("Unused variable '{}'", var.name));
                        self.add_message(message);
                    } else if !var.is_const && !var.is_modified && warn.on_unmodified {
                        let message = Message::new_warn_at_position(self, warn.pos.take().unwrap())
                            .message(format!(
                                "Variable '{}' is never modified, consider using 'const'",
                                var.name
                            ));
                        self.add_message(message);
                    }
                }
            }
        }
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
    /// Adds a variable to the current scope
    pub fn add_var(&mut self, mut var: VariableDecl) -> Option<usize> {
        let global_id = self.gen_var_id();
        var.global_id = Some(global_id);
        if var.is_public {
            self.context.pub_vars.push(var.clone());
        }
        let scope = self.context.scopes.last_mut().unwrap();
        scope.add_var(var);
        Some(global_id)
    }

    /// Gets a variable from the current scope or any parent scope
    pub fn get_var(&self, name: &str) -> Option<&VariableDecl> {
        self.context
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get_var(name))
    }

    /// Gets variable names
    pub fn get_var_names(&self) -> BTreeSet<&String> {
        self.context
            .scopes
            .iter()
            .rev()
            .flat_map(|scope| scope.get_var_names())
            .collect()
    }

    /// Returns a variable and marks it as used
    pub fn get_var_used(&mut self, name: &str) -> Option<&VariableDecl> {
        self.mark_var_used(name);
        self.get_var(name)
    }

    /// Marks a variable as used
    fn mark_var_used(&mut self, name: &str) {
        for scope in self.context.scopes.iter_mut().rev() {
            if let Some(var) = scope.vars.get_mut(name) {
                var.is_used = true;
                return;
            }
        }
    }

    /// Marks a variable as modified
    pub fn mark_var_modified(&mut self, name: &str) {
        for scope in self.context.scopes.iter_mut().rev() {
            if let Some(var) = scope.vars.get_mut(name) {
                var.is_modified = true;
                return;
            }
        }
    }

    /// Updates the type of a variable
    pub fn update_var_type(&mut self, name: &str, new_type: Type) {
        for scope in self.context.scopes.iter_mut().rev() {
            if let Some(var) = scope.vars.get_mut(name) {
                var.kind = new_type;
                return;
            }
        }
    }

    /* Functions */

    /// Generate a new global function id
    pub fn gen_fun_id(&mut self) -> usize {
        let id = self.fun_id;
        self.fun_id += 1;
        id
    }

    /// Adds a function declaration to the current scope
    pub fn add_fun_declaration(
        &mut self,
        fun: FunctionInterface,
        ctx: Context,
        block: Block,
    ) -> Option<usize> {
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
            self.fun_cache.add_declaration(global_id, ctx, block);
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

    pub fn add_var_declaration_existing(&mut self, var: VariableDecl) -> Option<usize> {
        let global_id = self.gen_var_id();
        if var.is_public {
            self.context.pub_vars.push(var.clone());
        }
        let scope = self.context.scopes.last_mut().unwrap();
        scope.add_var(var).then_some(global_id)
    }

    /// Adds a function instance to the cache
    /// This function returns the id of the function instance variant
    pub fn add_fun_instance(
        &mut self,
        fun: FunctionInterface,
        args_global_ids: Vec<Option<usize>>,
        block: Block,
    ) -> usize {
        let id = fun.id.expect("Function id is not set");
        self.fun_cache
            .add_instance(id, fun.into_fun_instance(args_global_ids, block))
    }

    /// Gets a function declaration from the current scope or any parent scope
    pub fn get_fun_declaration(&self, name: &str) -> Option<&FunctionDecl> {
        self.context
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get_fun(name))
    }

    /// Gets function names
    pub fn get_fun_names(&self) -> BTreeSet<&String> {
        self.context
            .scopes
            .iter()
            .rev()
            .flat_map(|scope| scope.get_fun_names())
            .collect()
    }

    pub fn get_narrowed_type(&self, name: &str) -> Option<&Type> {
        self.narrowed_types
            .iter()
            .rev()
            .find_map(|map| map.get(name))
    }

    pub fn with_narrowed_scope<B>(
        &mut self,
        facts: HashMap<String, Type>,
        mut body: B,
    ) -> SyntaxResult
    where
        B: FnMut(&mut Self) -> SyntaxResult,
    {
        self.narrowed_types.push(facts);
        let result = body(self);
        self.narrowed_types.pop();
        result
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
            parsing_functions: HashMap::new(),
            test_names: Vec::new(),
            narrowed_types: Vec::new(),
            suppress_warnings: false,
            sudo_used: false,
            first_pass_ctx: false,
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
