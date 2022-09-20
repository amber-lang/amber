use std::fs;
use heraclitus_compiler::prelude::*;
use crate::cli::cli_interface::CLI;
use crate::modules::block::Block;
use crate::utils::exports::{Exports, ExportUnit};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::import_string::ImportString;

#[derive(Debug, Clone)]
pub struct Import {
    path: ImportString,
    block: Block
}

impl Import {
    fn handle_export(&mut self, meta: &mut ParserMetadata, exports: Exports) {
        for export in exports.get_exports().iter().cloned() {
            match export {
                ExportUnit::Function(mut func_decl) => {
                    func_decl.is_public = false;
                    if !meta.mem.add_existing_function_declaration(func_decl) {
                        unimplemented!("Function redefinition");
                    }
                }
            }
        }
    }

    fn handle_import(&mut self, meta: &mut ParserMetadata, tok: Option<Token>) -> SyntaxResult {
        let imported_code = fs::read_to_string(&self.path.value).unwrap();
        let cc = CLI::create_compiler(imported_code.clone());
        match cc.tokenize() {
            Ok(tokens) => {
                self.block.set_scopeless();
                // Save snapshot of current file
                let code = meta.code.clone();
                let path = meta.path.clone();
                let expr = meta.expr.clone();
                let exports = meta.mem.exports.clone();
                let index = meta.get_index();
                // Parse the imported file
                meta.push_trace(ErrorDetails::from_token_option(meta, tok));
                meta.path = Some(self.path.value.clone());
                meta.code = Some(imported_code);
                meta.expr = tokens;
                meta.set_index(0);
                syntax(meta, &mut self.block)?;
                self.handle_export(meta, meta.mem.exports.clone());
                // Restore snapshot of current file
                meta.code = code;
                meta.path = path;
                meta.expr = expr;
                meta.mem.exports = exports;
                meta.set_index(index);
                meta.pop_trace();
            },
            Err(error) => {
                CLI::tokenize_error(meta.path.clone(), imported_code, error);
            }
        }
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Import {
    syntax_name!("Import File");

    fn new() -> Self {
        Self {
            path: ImportString::new(),
            block: Block::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "import")?;
        let tok = meta.get_current_token();
        token(meta, "*")?;
        token(meta, "from")?;
        syntax(meta, &mut self.path)?;
        self.handle_import(meta, tok)?;
        Ok(())
    }
}

impl TranslateModule for Import {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let tr = self.block.translate(meta);
        dbg!(tr.clone());
        tr
    }
}