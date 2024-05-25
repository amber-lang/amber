use std::collections::VecDeque;

use crate::{translate::compute::ArithType, utils::function_cache::FunctionCache};
use super::ParserMetadata;

pub struct TranslateMetadata {
    /// The arithmetic module that is used to evaluate math.
    pub arith_module: ArithType,
    /// A cache of defined functions - their body and metadata.
    pub fun_cache: FunctionCache,
    /// A queue of statements that are needed to be evaluated
    /// before current statement in order to be correct.
    pub stmt_queue: VecDeque<String>,
    /// The name of the function that is currently being translated.
    pub fun_name: Option<(String, usize, usize)>,
    /// Array id is used to determine the array that is being evaluated.
    pub array_id: usize,
    /// Value ID - used to store values in variables.
    pub value_id: usize,
    /// Determines whether the current context is a context in bash's `eval`.
    pub eval_ctx: bool,
    /// Determines whether the current context should be silenced.
    pub silenced: bool,
    /// The current indentation level.
    pub indent: i64,
    /// External commands used
    pub externs: Vec<String>
}

impl TranslateMetadata {
    pub fn new(meta: ParserMetadata) -> Self {
        TranslateMetadata {
            arith_module: ArithType::BcSed,
            fun_cache: meta.fun_cache,
            fun_name: None,
            stmt_queue: VecDeque::new(),
            array_id: 0,
            value_id: 0,
            eval_ctx: false,
            silenced: false,
            indent: -1,
            externs: vec![]
        }
    }

    pub fn gen_indent(&self) -> String {
        "    ".repeat(self.indent as usize)
    }

    pub fn increase_indent(&mut self) {
        self.indent += 1;
    }

    pub fn decrease_indent(&mut self) {
        self.indent -= 1;
    }

    pub fn gen_array_id(&mut self) -> usize {
        let id = self.array_id;
        self.array_id += 1;
        id
    }

    pub fn gen_value_id(&mut self) -> usize {
        let id = self.value_id;
        self.value_id += 1;
        id
    }

    pub fn gen_silent(&self) -> &'static str {
        if self.silenced { " > /dev/null 2>&1" } else { "" }
    }

    // Returns the appropriate amount of quotes with escape symbols.
    // This helps to avoid problems with `eval` expressions.
    pub fn gen_quote(&self) -> &'static str {
        if self.eval_ctx { "\\\"" } else { "\"" }
    }

    pub fn gen_subprocess(&self, stmt: &str) -> String {
        self.eval_ctx
            .then(|| format!("$(eval \"{}\")", stmt))
            .unwrap_or_else(|| format!("$({})", stmt))
    }

    pub fn gen_dollar(&self) -> &'static str {
        if self.eval_ctx { "\\$" } else { "$" }
    }
}
