use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::modules::condition::failed::Failed;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Mv {
    from: Expr,
    to: Expr,
    failed: Failed,
}

impl SyntaxModule<ParserMetadata> for Mv {
    syntax_name!("MoveFiles");

    fn new() -> Self {
        Mv {
            from: Expr::new(),
            to: Expr::new(),
            failed: Failed::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "mv")?;
        syntax(meta, &mut self.from)?;
        syntax(meta, &mut self.to)?;
        syntax(meta, &mut self.failed)?;
        Ok(())
    }
}

impl TranslateModule for Mv {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let from = self.from.translate(meta);
        let to = self.to.translate(meta);
        format!("mv {} {} || exit", from, to)
    }
}
