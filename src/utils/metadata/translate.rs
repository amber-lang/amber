use std::cmp;
use std::collections::VecDeque;

use super::ParserMetadata;
use crate::compiler::CompilerOptions;
use crate::modules::prelude::*;
use crate::translate::compute::ArithType;
use crate::utils::function_cache::FunctionCache;
use crate::utils::function_metadata::FunctionMetadata;

const INDENT_SPACES: &str = "    ";

pub struct TranslateMetadata {
    /// The arithmetic module that is used to evaluate math.
    pub arith_module: ArithType,
    /// A cache of defined functions - their body and metadata.
    pub fun_cache: FunctionCache,
    /// A queue of statements that are needed to be evaluated
    /// before current statement in order to be correct.
    pub stmt_queue: VecDeque<FragmentKind>,
    /// The metadata of the function that is currently being translated.
    pub fun_meta: Option<FunctionMetadata>,
    /// Used to determine the value or array being evaluated.
    pub value_id: usize,
    /// Determines whether the current context is a context in bash's `eval`.
    pub eval_ctx: bool,
    /// Determines whether the current context should be silenced.
    pub silenced: bool,
    /// The current indentation level.
    pub indent: i64,
    /// Determines if minify flag was set.
    pub minify: bool,
}

impl TranslateMetadata {
    pub fn new(meta: ParserMetadata, options: &CompilerOptions) -> Self {
        TranslateMetadata {
            arith_module: ArithType::BcSed,
            fun_cache: meta.fun_cache,
            fun_meta: None,
            stmt_queue: VecDeque::new(),
            value_id: 0,
            eval_ctx: false,
            silenced: false,
            indent: -1,
            minify: options.minify,
        }
    }

    pub fn single_indent() -> String {
        INDENT_SPACES.to_string()
    }

    pub fn gen_indent(&self) -> String {
        INDENT_SPACES.repeat(cmp::max(self.indent, 0) as usize)
    }

    #[inline]
    /// Create an intermediate variable and return it's variable expression
    pub fn push_intermediate_variable(&mut self, statement: VarStmtFragment) -> VarExprFragment {
        let expr = VarExprFragment::from_stmt(&statement);
        self.stmt_queue.push_back(statement.to_frag());
        expr
    }

    pub fn increase_indent(&mut self) {
        self.indent += 1;
    }

    pub fn decrease_indent(&mut self) {
        self.indent -= 1;
    }

    pub fn gen_value_id(&mut self) -> usize {
        let id = self.value_id;
        self.value_id += 1;
        id
    }

    pub fn gen_silent(&self) -> RawFragment {
        let expr = if self.silenced { " >/dev/null 2>&1" } else { "" };
        RawFragment::new(expr)
    }

    // Returns the appropriate amount of quotes with escape symbols.
    // This helps to avoid problems with `eval` expressions.
    pub fn gen_quote(&self) -> &'static str {
        if self.eval_ctx {
            "\\\""
        } else {
            "\""
        }
    }

    pub fn gen_dollar(&self) -> &'static str {
        if self.eval_ctx {
            "\\$"
        } else {
            "$"
        }
    }
}
