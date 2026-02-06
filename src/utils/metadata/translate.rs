use std::cmp;
use std::collections::VecDeque;

use super::ParserMetadata;
use crate::compiler::CompilerOptions;
use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::raw_fragment;
use crate::translate::compute::ArithType;
use crate::utils::function_cache::FunctionCache;
use crate::utils::function_metadata::FunctionMetadata;
use crate::utils::is_all_caps;
use amber_meta::ContextManager;

const INDENT_SPACES: &str = "    ";

#[derive(ContextManager)]
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
    #[context]
    pub silenced: bool,
    /// Determines whether the current context stderr should be silenced.
    #[context]
    pub suppress: bool,
    /// Determines whether the current context should use sudo.
    #[context]
    pub sudoed: bool,
    /// The current indentation level.
    pub indent: i64,
    /// Determines if minify flag was set.
    pub minify: bool,
    /// Determines whether the current context is an expression context.
    #[context]
    pub expr_ctx: bool,
    /// Determines whether the compiler is in test mode.
    pub test_mode: bool,
    /// The name of the test to run.
    pub test_name: Option<String>,
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
            suppress: false,
            sudoed: false,
            indent: -1,
            minify: options.minify,
            expr_ctx: false,
            test_mode: options.test_mode,
            test_name: options.test_name.clone(),
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
    pub fn push_ephemeral_variable(&mut self, statement: VarStmtFragment) -> VarExprFragment {
        let is_local = self.fun_meta.is_some();
        let stmt = statement.with_ephemeral(true).with_local(is_local);
        let expr = VarExprFragment::from_stmt(&stmt);
        self.stmt_queue.push_back(stmt.to_frag());
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

    pub fn gen_silent(&self) -> FragmentKind {
        if self.silenced {
            raw_fragment!(">/dev/null 2>&1")
        } else {
            FragmentKind::Empty
        }
    }

    pub fn gen_suppress(&self) -> FragmentKind {
        if self.suppress {
            raw_fragment!(" 2>/dev/null")
        } else {
            FragmentKind::Empty
        }
    }

    pub fn gen_sudo_prefix(&mut self) -> FragmentKind {
        if self.sudoed {
            let var_name = "__sudo";
            let var_expr = VarExprFragment::new(var_name, Type::Text).with_quotes(false);
            var_expr.to_frag()
        } else {
            FragmentKind::Empty
        }
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

    /// Returns the variable prefix based on the name casing.
    /// Returns "__" for fully uppercase names, "" for others.
    pub fn gen_variable_prefix(&self, name: &str) -> &'static str {
        if is_all_caps(name) {
            "__"
        } else {
            ""
        }
    }
}
