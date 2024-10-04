use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Typed, Type};
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::block::Block;

#[derive(Debug, Clone)]
pub struct IterLoop {
    block: Block,
    iter_expr: Expr,
    iter_index: Option<String>,
    iter_name: String,
    iter_type: Type
}

impl SyntaxModule<ParserMetadata> for IterLoop {
    syntax_name!("Iter Loop");

    fn new() -> Self {
        IterLoop {
            block: Block::new(),
            iter_expr: Expr::new(),
            iter_index: None,
            iter_name: String::new(),
            iter_type: Type::Generic
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "loop")?;
        self.iter_name = variable(meta, variable_name_extensions())?;
        if token(meta, ",").is_ok() {
            self.iter_index = Some(self.iter_name.clone());
            self.iter_name = variable(meta, variable_name_extensions())?;
        }
        token(meta, "in")?;
        context!({
            // Parse iterable
            let tok = meta.get_current_token();
            syntax(meta, &mut self.iter_expr)?;
            self.iter_type = match self.iter_expr.get_type() {
                Type::Array(kind) => *kind,
                _ => return error!(meta, tok, "Expected iterable"),
            };
            token(meta, "{")?;
            // Create iterator variable
            meta.push_scope();
            meta.add_var(&self.iter_name, self.iter_type.clone(), false, tok.clone());
            if let Some(index) = self.iter_index.as_ref() {
                meta.add_var(index, Type::Num, false, tok);
            }
            // Save loop context state and set it to true
            let mut new_is_loop_ctx = true;
            swap(&mut new_is_loop_ctx, &mut meta.context.is_loop_ctx);
            // Parse loop
            syntax(meta, &mut self.block)?;
            token(meta, "}")?;
            // Restore loop context state
            swap(&mut new_is_loop_ctx, &mut meta.context.is_loop_ctx);
            meta.pop_scope();
            Ok(())
        }, |pos| {
            error_pos!(meta, pos, "Syntax error in loop")
        })
    }
}

impl TranslateModule for IterLoop {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = &self.iter_name;
        let expr = self.iter_expr.translate(meta);
        match self.iter_index.as_ref() {
            Some(index) => {
                // Create an indentation for the index increment
                meta.increase_indent();
                let indent = meta.gen_indent();
                meta.decrease_indent();
                [
                    format!("{index}=0;"),
                    format!("for {name} in {expr}"),
                    "do".to_string(),
                    self.block.translate(meta),
                    format!("{indent}(( {index}++ )) || true"),
                    "done".to_string(),
                ].join("\n")
            },
            None => {
                [
                    format!("for {name} in {expr}"),
                    "do".to_string(),
                    self.block.translate(meta),
                    "done".to_string(),
                ].join("\n")
            },
        }
    }
}

impl DocumentationModule for IterLoop {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
