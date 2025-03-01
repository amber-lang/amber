use std::cmp;
use std::collections::VecDeque;

use super::ParserMetadata;
use crate::compiler::CompilerOptions;
use crate::fragments;
use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::translate::compute::ArithType;
use crate::translate::fragments::eval::EvalFragment;
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
    pub stmt_queue: VecDeque<TranslationFragment>,
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
    pub fn push_stmt_variable(&mut self, name: &str, id: Option<usize>, kind: Type, value: TranslationFragment) -> VarFragment {
        let (stmt, var) = self.gen_stmt_variable(name, id, kind, false, None, "=", value);
        self.stmt_queue.push_back(stmt);
        var
    }

    #[inline]
    pub fn push_stmt_variable_lazy(&mut self, name: &str, id: Option<usize>, kind: Type, value: TranslationFragment) -> VarFragment {
        let (stmt, var) = self.gen_stmt_variable_lazy(name, id, kind, false, None, "=", value);
        self.stmt_queue.push_back(stmt);
        var
    }

    pub fn gen_stmt_variable(
        &mut self,
        name: &str,
        id: Option<usize>,
        kind: Type,
        is_ref: bool,
        index: Option<TranslationFragment>,
        op: &str,
        value: TranslationFragment
    ) -> (TranslationFragment, VarFragment) {
        let is_array = kind.is_array();
        let variable = VarFragment::new(name, kind, is_ref, id);
        let frags = {
            let mut result = vec![];
            match is_ref {
                true => result.push(fragments!(raw: "${{{}}}", variable.get_name())),
                false => result.push(fragments!(raw: "{}", variable.get_name())),
            }
            if let Some(index) = index {
                result.push(fragments!("[", index, "]"));
            }
            result.push(fragments!(raw: "{}", op));
            if is_array {
                result.push(fragments!(raw: "("));
            }
            result.push(value);
            if is_array {
                result.push(fragments!(raw: ")"));
            }
            result
        };
        let stmt = CompoundFragment::new(frags).to_frag();
        (EvalFragment::new(stmt, is_ref).to_frag(), variable)
    }

    pub fn gen_stmt_variable_lazy(
        &mut self,
        name: &str,
        id: Option<usize>,
        kind: Type,
        is_ref: bool,
        index: Option<TranslationFragment>,
        op: &str,
        value: TranslationFragment
    ) -> (TranslationFragment, VarFragment) {
        match value {
            // If the value is already variable, then we don't need to assign it to a new variable.
            TranslationFragment::Var(var) => {
                (TranslationFragment::Empty, var)
            },
            _ => {
                self.gen_stmt_variable(name, id, kind, is_ref, index, op, value)
            }
        }
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
        RawFragment::new(if self.silenced { " > /dev/null 2>&1" } else { "" })
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
