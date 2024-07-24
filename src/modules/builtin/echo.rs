use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
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
        format!("echo {}", value)
    }
}

impl DocumentationModule for Echo {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
