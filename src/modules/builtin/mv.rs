use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::modules::condition::failed::Failed;
use crate::translate::module::TranslateModule;
use crate::docs::module::DocumentationModule;
use crate::modules::types::{Type, Typed};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::modules::command::modifier::CommandModifier;

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
            let mut path_type = self.source.get_type();
            if path_type != Type::Text {
                let position = self.source.get_position(meta);
                return error_pos!(meta, position => {
                    message: "Builtin function `mv` can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", path_type, Type::Text)
                });
            }
            syntax(meta, &mut self.destination)?;
            path_type = self.destination.get_type();
            if path_type != Type::Text {
                let position = self.destination.get_position(meta);
                return error_pos!(meta, position => {
                    message: "Builtin function `mv` can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", path_type, Type::Text)
                });
            }
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
        format!("mv {source} {destination}{silent}\n{failed}").trim_end().to_string()
    }
}

impl DocumentationModule for Mv {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
