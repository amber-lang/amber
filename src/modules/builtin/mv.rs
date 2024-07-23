use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::modules::condition::failed::Failed;
use crate::translate::module::TranslateModule;
use crate::docs::module::DocumentationModule;
use crate::utils::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Mv {
    source: Expr,
    destination: Expr,
    failed: Failed,
}

impl SyntaxModule<ParserMetadata> for Mv {
    syntax_name!("MoveFiles");

    fn new() -> Self {
        Mv {
            source: Expr::new(),
            destination: Expr::new(),
            failed: Failed::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "mv")?;
        syntax(meta, &mut self.source)?;
        syntax(meta, &mut self.destination)?;
        syntax(meta, &mut self.failed)?;
        Ok(())
    }
}

impl TranslateModule for Mv {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let source = self.source.translate(meta);
        let destination = self.destination.translate(meta);
        format!("mv {} {}", source, destination)
    }
}

impl DocumentationModule for Mv {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
