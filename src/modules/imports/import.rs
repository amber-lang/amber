use std::fs;
use heraclitus_compiler::prelude::*;
use crate::compiler::AmberCompiler;
use crate::modules::block::Block;
use crate::utils::exports::{Exports, ExportUnit};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::import_string::ImportString;

#[derive(Debug, Clone)]
pub struct Import {
    path: ImportString
}

impl Import {
    fn handle_export(&mut self, meta: &mut ParserMetadata, exports: Exports) -> SyntaxResult {
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
        Ok(())
    }

    fn add_import(&mut self, meta: &mut ParserMetadata, tok: Option<Token>, path: &str) -> SyntaxResult {
        if meta.import_history.add_import(meta.get_path(), path.to_string()).is_none() {
            return error!(meta, tok => {
                message: "Circular import detected",
                comment: "Please remove the circular import"
            })
        }
        Ok(())
    }

    fn resolve_import(&mut self, meta: &mut ParserMetadata, tok: Option<Token>) -> Result<String, Failure> {
        match fs::read_to_string(self.path.value.clone()) {
            Ok(content) => Ok(content),
            Err(err) => error!(meta, tok => {
                message: format!("Could not read file '{}'", self.path.value),
                comment: err.to_string()
            })
        }
    }

    fn handle_import(&mut self, meta: &mut ParserMetadata, tok: Option<Token>, imported_code: String) -> SyntaxResult {
        match meta.import_history.get_export(Some(self.path.value.clone())) {
            Some(exports) => self.handle_export(meta, exports),
            None => self.handle_compile_code(meta, tok, imported_code)
        }
    }

    fn handle_compile_code(&mut self, meta: &mut ParserMetadata, tok: Option<Token>, imported_code: String) -> SyntaxResult {
        match AmberCompiler::new(imported_code.clone(), Some(self.path.value.clone())).tokenize() {
            Ok(tokens) => {
                let mut block = Block::new();
                // Save snapshot of current file
                let code = meta.code.clone();
                let path = meta.path.clone();
                let expr = meta.expr.clone();
                let exports = meta.mem.exports.clone();
                let index = meta.get_index();
                let scopes = meta.mem.scopes.clone();
                // Parse the imported file
                meta.push_trace(PositionInfo::from_token(meta, tok));
                meta.path = Some(self.path.value.clone());
                meta.code = Some(imported_code);
                meta.expr = tokens;
                meta.set_index(0);
                meta.mem.scopes = vec![];
                syntax(meta, &mut block)?;
                meta.mem.scopes = scopes;
                meta.import_history.add_import_block(Some(self.path.value.clone()), block);
                meta.import_history.add_export(Some(self.path.value.clone()), meta.mem.exports.clone());
                self.handle_export(meta, meta.mem.exports.clone())?;
                // Restore snapshot of current file
                meta.code = code;
                meta.path = path;
                meta.expr = expr;
                meta.mem.exports = exports;
                meta.set_index(index);
                meta.pop_trace();
                Ok(())
            }
            Err(err) => {
                Err(Failure::Loud(err))
            }
        }
    }
}

impl SyntaxModule<ParserMetadata> for Import {
    syntax_name!("Import File");

    fn new() -> Self {
        Self {
            path: ImportString::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "import")?;
        let tok = meta.get_current_token();
        token(meta, "*")?;
        token(meta, "from")?;
        let tok_str = meta.get_current_token();
        syntax(meta, &mut self.path)?;
        let imported_code = if self.path.value == "[standard library]" {
            self.add_import(meta, tok_str, "[standard library]")?;
            AmberCompiler::import_std()
        } else {
            self.add_import(meta, tok_str.clone(), &self.path.value.clone())?;
            self.resolve_import(meta, tok_str)?
        };
        self.handle_import(meta, tok, imported_code)?;
        Ok(())
    }
}

impl TranslateModule for Import {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        "".to_string()
    }
}