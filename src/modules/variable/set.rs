use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::utils::metadata::ParserMetadata;
use super::variable_name_extensions;

#[derive(Debug)]
pub struct VariableSet {
    name: String,
    value: Box<Expr>
}

impl SyntaxModule<ParserMetadata> for VariableSet {
    syntax_name!("Variable Set");

    fn new() -> Self {
        VariableSet {
            name: format!(""),
            value: Box::new(Expr::new())
        }
    }
    
    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.name = variable(meta, variable_name_extensions())?;
        token(meta, "=")?;
        syntax(meta, &mut *self.value)?;
        Ok(())
    }
}