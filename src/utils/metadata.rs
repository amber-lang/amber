use heraclitus_compiler::prelude::*;
use super::variable_memory::VariableMemory;

pub struct ParserMetadata {
    pub expr: Vec<Token>,
    index: usize,
    pub path: Option<String>,
    pub binop_border: Option<usize>,
    pub var_mem: VariableMemory,
    debug: Option<usize>
}

impl Metadata for ParserMetadata {
    fn new(expression: Vec<Token>, path: Option<String>) -> Self {
        ParserMetadata {
            expr: expression,
            index: 0,
            path,
            binop_border: None,
            debug: None,
            var_mem: VariableMemory::new()
        }
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index
    }

    fn get_token_at(&self, index: usize) -> Option<Token> {
        match self.expr.get(index) {
            Some(token) => Some(token.clone()),
            None => None
        }
    }
    fn get_debug(&mut self) -> Option<usize> {
        self.debug
    }
    fn set_debug(&mut self, indent: usize) {
        self.debug = Some(indent)
    }
}