use heraclitus_compiler::prelude::*;

use crate::raw_fragment;
use crate::modules::types::Type;
use crate::modules::block::Block;
use crate::modules::prelude::*;
use crate::utils::context::{VariableDecl, VariableDeclWarn};
use crate::utils::metadata::ParserMetadata;

use super::variable::variable_name_extensions;

#[derive(Debug, Clone)]
pub struct Main {
    pub args: Option<String>,
    pub args_tok: Option<Token>,
    pub args_global_id: Option<usize>,
    pub block: Block,
    pub token: Option<Token>,
    pub is_skipped: bool,
}

impl SyntaxModule<ParserMetadata> for Main {
    syntax_name!("Main");

    fn new() -> Self {
        Self {
            args: None,
            args_tok: None,
            args_global_id: None,
            block: Block::new().with_no_indent(),
            token: None,
            is_skipped: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.token = meta.get_current_token();
        token(meta, "main")?;
        // If this main is included in other file, skip it
        if !meta.context.trace.is_empty() {
            self.is_skipped = true;
        }
        context!({
            meta.context.is_main_ctx = true;
            if token(meta, "(").is_ok() {
                self.args_tok = meta.get_current_token();
                self.args = Some(variable(meta, variable_name_extensions())?);
                token(meta, ")")?;
            }
            // Parse the block
            syntax(meta, &mut self.block)?;
            meta.context.is_main_ctx = false;
            Ok(())
        }, |pos| {
            error_pos!(meta, pos, "Undefined syntax in main block")
        })
    }
}

impl TypeCheckModule for Main {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Main cannot be parsed inside of a block
        if !meta.is_global_scope() {
            return error!(meta, self.token.clone(), "Main must be in the global scope")
        }

        // Typecheck the main block content
        meta.with_push_scope(true, |meta| {
            // Create variables for main arguments
            for arg in self.args.iter() {
                let var = VariableDecl::new(arg.clone(), Type::Array(Box::new(Type::Text)))
                    .with_const(true)
                    .with_warn(VariableDeclWarn::from_token(meta, self.args_tok.clone()));
                self.args_global_id = Some(meta.add_var(var).unwrap());
            }
            // Typecheck the block
            self.block.typecheck(meta)?;
            Ok(())
        })
    }
}

impl TranslateModule for Main {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.is_skipped || meta.test_mode {
            FragmentKind::Empty
        } else {
            let quote = meta.gen_quote();
            let dollar = meta.gen_dollar();
            let global_id = meta.gen_value_id();
            let args = self.args.clone().map_or_else(
                || FragmentKind::Empty,
                |name| {
                    let id = self.args_global_id.unwrap_or(global_id);
                    raw_fragment!("declare -r {name}_{id}=({quote}{dollar}0{quote} {quote}{dollar}@{quote})")
                }
            );
            // Temporarily decrease the indentation level to counteract
            // the indentation applied by the block translation.  Unlike
            // other instances of code blocks, we do not want to indent
            // the code generated from the main block.
            meta.stmt_queue.push_back(args);
            self.block.translate(meta)
        }
    }
}

impl DocumentationModule for Main {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
