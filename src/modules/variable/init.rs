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

impl VariableInit {
    fn handle_memory(&mut self, meta: &mut ParserMetadata, token: Option<Token>, kind: Type) {
        if !meta.var_mem.add_variable(self.name.clone(), kind) {
            let message = format!("Cannot overwrite existing variable '{}'", self.name);
            let details = ErrorDetails::from_token_option(token);
            get_error_logger(meta, details)
                .attach_message(message)
                .show()
                .exit()
        }
    }
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
        // Save current token
        let tok = meta.get_current_token();
        // Get the variable name
        self.name = variable(meta, variable_name_extensions())?;
        token(meta, "=")?;
        syntax(meta, &mut *self.expr)?;
        self.handle_memory(meta, tok, self.expr.get_type());
        Ok(())
    }
}