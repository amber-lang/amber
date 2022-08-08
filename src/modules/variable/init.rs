use heraclitus_compiler::prelude::*;

use crate::modules::{Type, Typed};
use crate::modules::expression::expr::Expr;
use crate::utils::error::get_error_logger;
use crate::utils::metadata::ParserMetadata;
use super::variable_name_extensions;

#[derive(Debug)]
pub struct VariableInit {
    name: String,
    expr: Box<Expr>
}

impl SyntaxModule<ParserMetadata> for VariableInit {
    syntax_name!("Variable Initialize");

    fn new() -> Self {
        VariableInit {
            name: format!(""),
            expr: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "let")?;
        // Get the variable name
        self.name = variable(meta, variable_name_extensions())?;
        token(meta, "=")?;
        syntax(meta, &mut *self.expr)?;
        // Add a variable to the memory
        meta.var_mem.add_variable(self.name.clone(), self.expr.get_type());
        Ok(())
    }
}