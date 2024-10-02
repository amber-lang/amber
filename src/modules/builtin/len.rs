use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::docs::module::DocumentationModule;
use crate::modules::types::{Type, Typed};
use crate::utils::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Len {
    value: Expr,
}

impl SyntaxModule<ParserMetadata> for Len {
    syntax_name!("Length");

    fn new() -> Self {
        Len {
            value: Expr::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "len")?;
        syntax(meta, &mut self.value)?;

        Ok(())
    }
}

impl TranslateModule for Len {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let path_type = self.value.get_type();
        let value = self.value.translate(meta);
        if path_type != Type::Text {
            format!("echo \"${{#{}}}\"", value).trim_end().to_string()
        } else {
            format!("echo \"${{#{}[@]}}\"", value).trim_end().to_string()
        }
    }
}

impl DocumentationModule for Len {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
