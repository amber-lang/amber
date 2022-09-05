use heraclitus_compiler::prelude::*;
use crate::modules::{Typed};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use super::{variable_name_extensions, handle_add_variable};

#[derive(Debug, Clone)]
pub struct VariableInit {
    name: String,
    expr: Box<Expr>
}

impl SyntaxModule<ParserMetadata> for VariableInit {
    syntax_name!("Variable Initialize");

    fn new() -> Self {
        VariableInit {
            name: String::new(),
            expr: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "let")?;
        // Get the variable name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        token(meta, "=")?;
        syntax(meta, &mut *self.expr)?;
        // Add a variable to the memory
        handle_add_variable(meta, &self.name, self.expr.get_type(), tok);
        Ok(())
    }
}

impl TranslateModule for VariableInit {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = self.name.clone();
        let expr = self.expr.translate(meta);
        format!("{name}={expr}")
    }
}