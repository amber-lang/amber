use heraclitus_compiler::prelude::*;
use super::literal::{
    bool::Bool,
    number::Number,
    text::Text
};
use super::parenthesis::Parenthesis;

#[derive(Debug, Clone)]
pub enum ExprId {
    Bool,
    Number,
    Text,
    Parenthesis
}

#[derive(Debug)]
pub enum ExprType {
    Bool(Bool),
    Number(Number),
    Text(Text),
    Parenthesis(Parenthesis)
}

#[derive(Debug)]
pub struct Expr {
    value: Option<ExprType>,
    exclude: Option<ExprId>
}

impl Expr {
    fn statement_types(&self) -> Vec<ExprType> {
        vec![
            ExprType::Bool(Bool::new()),
            ExprType::Number(Number::new()),
            ExprType::Text(Text::new()),
            ExprType::Parenthesis(Parenthesis::new())
        ]
    }
    
    fn parse_statement(&mut self, meta: &mut DefaultMetadata, statement: ExprType) -> SyntaxResult {
        match statement {
            ExprType::Bool(bool) => self.get(meta, bool, ExprType::Bool, ExprId::Bool),
            ExprType::Number(num) => self.get(meta, num, ExprType::Number, ExprId::Number),
            ExprType::Text(txt) => self.get(meta, txt, ExprType::Text, ExprId::Text),
            ExprType::Parenthesis(p) => self.get(meta, p, ExprType::Parenthesis, ExprId::Parenthesis)
        }
    }

    // Exclude some syntax module
    pub fn exclude(&mut self, expr_id: ExprId) {
        self.exclude = Some(expr_id);
    }

    // Get result out of the provided module and save it in the internal state
    fn get<M,S>(&mut self, meta: &mut M, mut module: S, cb: impl Fn(S) -> ExprType, id: ExprId) -> SyntaxResult
    where
        M: Metadata,
        S: SyntaxModule<M>
    {
        // Check if exclusion occurs
        if let Some(excludes) = &self.exclude {
            if excludes.clone() as usize == id as usize {
                return Err(ErrorDetails::from_metadata(meta))
            }
        }
        // Match syntax
        match syntax(meta, &mut module) {
            Ok(()) => {
                self.value = Some(cb(module));
                Ok(())    
            }
            Err(details) => Err(details)
        }
    }
}

impl SyntaxModule<DefaultMetadata> for Expr {
    fn new() -> Self {
        Expr {
            value: None,
            exclude: None
        }
    }

    fn parse(&mut self, meta: &mut DefaultMetadata) -> SyntaxResult {
        let mut error = None;
        let statements = self.statement_types();
        for statement in statements {
            match self.parse_statement(meta, statement) {
                Ok(()) => return Ok(()),
                Err(details) => error = Some(details)
            }
        }
        Err(error.unwrap())
    }
}
