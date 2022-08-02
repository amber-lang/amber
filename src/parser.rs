use heraclitus_compiler::prelude::*;

pub struct ParserMetadata {
    pub expr: Vec<Token>,
    index: usize,
    pub path: Option<String>,
    pub binop_border: Option<usize>
}

impl Metadata for ParserMetadata {
    fn new(expression: Vec<Token>, path: Option<String>) -> Self {
        ParserMetadata {
            expr: expression,
            index: 0,
            path,
            binop_border: None
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
}