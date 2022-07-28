use heraclitus_compiler::prelude::*;
use super::literal::{
    bool::Bool,
    number::Number,
    text::Text
};

#[derive(Debug)]
enum StatementType {
    Bool(Bool),
    Number(Number),
    Text(Text)
}

#[derive(Debug)]
pub struct Statement {
    value: Option<StatementType>
}

impl Statement {
    fn statement_types(&self) -> Vec<StatementType> {
        vec![
            StatementType::Bool(Bool::new()),
            StatementType::Number(Number::new()),
            StatementType::Text(Text::new())
        ]
    }
    
    fn parse_statement(&mut self, meta: &mut DefaultMetadata, statement: StatementType) -> SyntaxResult {
        match statement {
            StatementType::Bool(bool) => self.get(meta, bool, StatementType::Bool),
            StatementType::Number(num) => self.get(meta, num, StatementType::Number),
            StatementType::Text(txt) => self.get(meta, txt, StatementType::Text)
        }
    }

    // Get result out of the provided module and save it in the internal state
    fn get<M,S>(&mut self, meta: &mut M, mut module: S, cb: impl Fn(S) -> StatementType) -> SyntaxResult
    where
        M: Metadata,
        S: SyntaxModule<M>
    {
        match syntax(meta, &mut module) {
            Ok(()) => {
                self.value = Some(cb(module));
                Ok(())    
            }
            Err(details) => Err(details)
        }
    }
}

impl SyntaxModule<DefaultMetadata> for Statement {
    fn new() -> Self {
        Statement {
            value: None
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