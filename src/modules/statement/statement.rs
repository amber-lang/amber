use heraclitus_compiler::prelude::*;
use crate::utils::metadata::ParserMetadata;
use crate::modules::expression::expr::Expr;
use crate::modules::variable::{
    init::VariableInit,
    set::VariableSet
};


#[derive(Debug)]
enum StatementType {
    Expr(Expr),
    VariableInit(VariableInit),
    VariableSet(VariableSet)
}

#[derive(Debug)]
pub struct Statement {
    value: Option<StatementType>
}

impl Statement {
    fn statement_types(&self) -> Vec<StatementType> {
        vec![
            // Variables
            StatementType::VariableInit(VariableInit::new()),
            StatementType::VariableSet(VariableSet::new()),
            // Expression
            StatementType::Expr(Expr::new())
        ]
    }
    
    fn parse_statement(&mut self, meta: &mut ParserMetadata, statement: StatementType) -> SyntaxResult {
        match statement {
            StatementType::Expr(st) => self.get(meta, st, StatementType::Expr),
            StatementType::VariableInit(st) => self.get(meta, st, StatementType::VariableInit),
            StatementType::VariableSet(st) => self.get(meta, st, StatementType::VariableSet)
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

impl SyntaxModule<ParserMetadata> for Statement {
    syntax_name!("Statement");

    fn new() -> Self {
        Statement {
            value: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
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