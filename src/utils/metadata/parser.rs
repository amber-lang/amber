use heraclitus_compiler::prelude::*;
use crate::utils::memory::Memory;

#[derive(Clone, Debug)]
pub struct ParserMetadata {
    pub expr: Vec<Token>,
    index: usize,
    pub path: Option<String>,
    pub code: Option<String>,
    pub binop_border: Option<usize>,
    pub mem: Memory,
    debug: Option<usize>,
    pub trace: Vec<ErrorDetails>,
    pub loop_ctx: bool,
    pub function_ctx: bool
}

impl ParserMetadata {
    pub fn push_trace(&mut self, details: ErrorDetails) {
        self.trace.push(details);
    }

    pub fn pop_trace(&mut self) -> Option<ErrorDetails> {
        self.trace.pop()
    }
}

impl Metadata for ParserMetadata {
    fn new(tokens: Vec<Token>, path: Option<String>, code: Option<String>) -> Self {
        ParserMetadata {
            expr: tokens,
            index: 0,
            path,
            code,
            binop_border: None,
            mem: Memory::new(),
            debug: None,
            trace: Vec::new(),
            loop_ctx: false,
            function_ctx: false
        }
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index
    }

    fn get_token_at(&self, index: usize) -> Option<Token> {
        self.expr.get(index).cloned()
    }

    fn get_debug(&mut self) -> Option<usize> {
        self.debug
    }

    fn set_debug(&mut self, indent: usize) {
        self.debug = Some(indent)
    }

    fn get_code(&self) -> Option<&String> {
        self.code.as_ref()
    }

    fn get_path(&self) -> Option<String> {
        self.path.clone()
    }
}