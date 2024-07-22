use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::docs::module::DocumentationModule;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Cd {
    value: Expr
}

impl SyntaxModule<ParserMetadata> for Cd {
    syntax_name!("ChangeDirectory");

    fn new() -> Self {
        Cd {
            value: Expr::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "cd")?;
        syntax(meta, &mut self.value)?;
        Ok(())
    }
}

impl TranslateModule for Cd {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let value = self.value.translate(meta);
        format!("cd {} || exit", value)
    }
}

impl DocumentationModule for Cd {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
