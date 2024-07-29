use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failed::Failed;
use crate::modules::expression::expr::Expr;
use crate::docs::module::DocumentationModule;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};

#[derive(Debug, Clone)]
pub struct Rm {
    path: Expr,
    modifier: CommandModifier,
    failed: Failed,
}

impl SyntaxModule<ParserMetadata> for Rm {
    syntax_name!("Remove");

    fn new() -> Self {
        Rm {
            path: Expr::new(),
            failed: Failed::new(),
            modifier: CommandModifier::new().parse_expr(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            let tok = meta.get_current_token();
            token(meta, "rm")?;
            syntax(meta, &mut self.path)?;
            let path_type = self.path.get_type();
            if path_type != Type::Text {
                return error!(meta, tok => {
                    message: "'rm' can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", path_type, Type::Text)
                });
            }
            syntax(meta, &mut self.failed)?;
            Ok(())
        })
    }
}

impl TranslateModule for Rm {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let path = self.path.translate(meta);
        let failed = self.failed.translate(meta);
        let mut is_silent = self.modifier.is_silent || meta.silenced;
        swap(&mut is_silent, &mut meta.silenced);
        let silent = meta.gen_silent();
        swap(&mut is_silent, &mut meta.silenced);
        format!("rm -rf {path}{silent}\n{failed}").trim_end().to_string()
    }
}

impl DocumentationModule for Rm {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}

