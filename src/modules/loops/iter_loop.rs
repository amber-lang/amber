use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::types::{Typed, Type};
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::context::Context;
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
        token(meta, "for")?;
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
            meta.with_push_scope(|meta| {
                meta.add_var(&self.iter_name, self.iter_type.clone(), None, false);
                if let Some(index) = self.iter_index.as_ref() {
                    meta.add_var(index, Type::Num, None, false);
                }
                // Save loop context state and set it to true
                meta.with_context_fn(Context::set_is_loop_ctx, true, |meta| {
                    // Parse loop
                    syntax(meta, &mut self.block)?;
                    token(meta, "}")?;
                    Ok(())
                })?;
                Ok(())
            })?;
            Ok(())
        }, |pos| {
            error_pos!(meta, pos, "Syntax error in loop")
        })
    }
}

impl TranslateModule for IterLoop {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let (prefix, suffix) = self.surround_iter(meta);
        match self.iter_index.as_ref() {
            Some(index) => {
                let indent = TranslateMetadata::single_indent();
                [
                    format!("{index}=0;"),
                    prefix,
                    self.block.translate(meta),
                    format!("{indent}(( {index}++ )) || true"),
                    suffix,
                ].join("\n")
            },
            None => {
                [
                    prefix,
                    self.block.translate(meta),
                    suffix,
                ].join("\n")
            },
        }
    }
}

impl IterLoop {
    fn surround_iter(&self, meta: &mut TranslateMetadata) -> (String, String) {
        let name = &self.iter_name;
        if let Some(ExprType::LinesInvocation(value)) = &self.iter_expr.value {
            value.surround_iter(meta, name)
        } else {
            let expr = self.iter_expr.translate(meta);
            let prefix = format!("for {name} in {expr}; do");
            let suffix = String::from("done");
            (prefix, suffix)
        }
    }
}

impl DocumentationModule for IterLoop {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
