use std::mem::swap;

use crate::docs::module::DocumentationModule;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failed::Failed;
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Mv {
    source: Expr,
    destination: Expr,
    modifier: CommandModifier,
    failed: Failed,
}

impl SyntaxModule<ParserMetadata> for Mv {
    syntax_name!("MoveFiles");

    fn new() -> Self {
        Mv {
            source: Expr::new(),
            destination: Expr::new(),
            failed: Failed::new(),
            modifier: CommandModifier::new().parse_expr(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            token(meta, "mv")?;
            syntax(meta, &mut self.source)?;
            syntax(meta, &mut self.destination)?;
            syntax(meta, &mut self.failed)?;
            Ok(())
        })
    }
}

impl TranslateModule for Mv {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let source = self.source.translate(meta);
        let destination = self.destination.translate(meta);
        let failed = self.failed.translate(meta);
        let mut is_silent = self.modifier.is_silent || meta.silenced;
        swap(&mut is_silent, &mut meta.silenced);
        let silent = meta.gen_silent();
        swap(&mut is_silent, &mut meta.silenced);
        format!("mv {source} {destination}{silent}\n{failed}")
            .trim_end()
            .to_string()
    }
}

impl DocumentationModule for Mv {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
