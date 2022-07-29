use heraclitus_compiler::prelude::*;
use super::{
    experssion::Expr
};

#[derive(Debug)]
enum StatementType {
    Expr(Expr)
}

#[derive(Debug)]
pub struct Statement {
    value: Option<StatementType>
}

impl Statement {
    fn statement_types(&self) -> Vec<StatementType> {
        vec![
            StatementType::Expr(Expr::new())
        ]
    }
    
    fn parse_statement(&mut self, meta: &mut DefaultMetadata, statement: StatementType) -> SyntaxResult {
        match statement {
            StatementType::Expr(bool) => self.get(meta, bool, StatementType::Expr)
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