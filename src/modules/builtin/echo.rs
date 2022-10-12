use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Echo {
    value: Box<Expr>
}

impl SyntaxModule<ParserMetadata> for Echo {
    syntax_name!("Log");

    fn new() -> Self {
        Echo {
            value: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "echo")?;
        syntax(meta, &mut *self.value)?;
        Ok(())
    }
}

impl TranslateModule for Echo {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let value = self.value.translate(meta);
        // If it's a function invocation we want to strip down the inner command
        // This way the newline characters in the stdout don't get lost
        if self.value.is_function_invocation() && value.starts_with('$') {
            value.get(2..value.len() - 1).unwrap().to_string()
        } else {
            format!("echo {}", value)
        }
    }
}
