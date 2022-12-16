use std::collections::VecDeque;

use crate::{translate::compute::ArithType, utils::function_cache::FunctionCache};
use super::ParserMetadata;

pub struct TranslateMetadata {
    pub arith_module: ArithType,
    pub fun_cache: FunctionCache,
    pub stmt_queue: VecDeque<String>,
    pub indent: i64
}

impl TranslateMetadata {
    pub fn new(meta: ParserMetadata) -> Self {
        TranslateMetadata {
            arith_module: ArithType::BcSed,
            fun_cache: meta.fun_cache,
            stmt_queue: VecDeque::new(),
            indent: -1
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
}
