use std::fs;
use heraclitus_compiler::prelude::*;
use crate::compiler::AmberCompiler;
use crate::modules::block::Block;
use crate::modules::variable::variable_name_extensions;
use crate::utils::exports::{Exports, ExportUnit};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::import_string::ImportString;

#[derive(Debug, Clone)]
pub struct Import {
    path: ImportString,
    token_import: Option<Token>,
    token_path: Option<Token>,
    is_all: bool,
    export_defs: Vec<(String, Option<String>, Option<Token>)>
}

impl Import {
    fn handle_export(&mut self, meta: &mut ParserMetadata, exports: Exports) -> SyntaxResult {
        let exports = exports.get_exports().iter().cloned();
        for (name, alias, tok) in self.export_defs.drain(..) {
            let mut found = false;
            for export_unit in exports.clone() {
                match export_unit {
                    ExportUnit::Function(mut func) => {
                        if &func.name == &name {
                            found = true;
                            func.name = alias.unwrap_or(name.clone());
                            if !meta.mem.add_existing_function_declaration(func) {
                                return error!(meta, tok => {
                                    message: format!("Function '{}' is already defined", name)
                                })
                            }
                            break
                        }
                    }
                }
            }
            if !found {
                return error!(meta, tok => {
                    message: format!("Export '{}' not found in module '{}'", &name, self.path.value),
                    comment: "Exports are case-sensitive"
                })
            }   
        }
        if self.is_all {
            for export in exports {
                match export {
                    ExportUnit::Function(mut func_decl) => {
                        let name = func_decl.name.clone();
                        func_decl.is_public = false;
                        if !meta.mem.add_existing_function_declaration(func_decl) {
                            return error!(meta, self.token_import.clone() => {
                                message: format!("Function '{}' is already defined", name)
                            })
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn add_import(&mut self, meta: &mut ParserMetadata, path: &str) -> SyntaxResult {
        if meta.import_history.add_import(meta.get_path(), path.to_string()).is_none() {
            return error!(meta, self.token_path.clone() => {
                message: "Circular import detected",
                comment: "Please remove the circular import"
            })
        }
        Ok(())
    }

    fn resolve_import(&mut self, meta: &mut ParserMetadata) -> Result<String, Failure> {
        match fs::read_to_string(self.path.value.clone()) {
            Ok(content) => Ok(content),
            Err(err) => error!(meta, self.token_path.clone() => {
                message: format!("Could not read file '{}'", self.path.value),
                comment: err.to_string()
            })
        }
    }

    fn handle_import(&mut self, meta: &mut ParserMetadata, imported_code: String) -> SyntaxResult {
        match meta.import_history.get_export(Some(self.path.value.clone())) {
            Some(exports) => self.handle_export(meta, exports),
            None => self.handle_compile_code(meta, imported_code)
        }
    }

    fn handle_compile_code(&mut self, meta: &mut ParserMetadata, imported_code: String) -> SyntaxResult {
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
                meta.push_trace(PositionInfo::from_token(meta, self.token_import.clone()));
                meta.path = Some(self.path.value.clone());
                meta.code = Some(imported_code);
                meta.expr = tokens;
                meta.set_index(0);
                meta.mem.scopes = vec![];
                syntax(meta, &mut block)?;
                // Restore snapshot of current file
                meta.mem.scopes = scopes;
                meta.code = code;
                meta.path = path;
                meta.expr = expr;
                meta.set_index(index);
                meta.pop_trace();
                // Finalize importing phase
                meta.import_history.add_import_block(Some(self.path.value.clone()), block);
                meta.import_history.add_export(Some(self.path.value.clone()), meta.mem.exports.clone());
                self.handle_export(meta, meta.mem.exports.clone())?;
                // Restore exports
                meta.mem.exports = exports;
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
            path: ImportString::new(),
            token_import: None,
            token_path: None,
            is_all: false,
            export_defs: vec![]
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.token_import = meta.get_current_token();
        token(meta, "import")?;
        if meta.mem.get_depth() > 1 {
            return error!(meta, self.token_import.clone(), "Imports must be in the global scope")
        }
        match token(meta, "*") {
            Ok(_) => self.is_all = true,
            Err(_) => {
                token(meta, "{")?;
                let mut exports = vec![];
                loop {
                    let tok = meta.get_current_token();
                    let name = variable(meta, variable_name_extensions())?;
                    let alias = match token(meta, "as") {
                        Ok(_) => Some(variable(meta, variable_name_extensions())?),
                        Err(_) => None
                    };
                    exports.push((name, alias, tok));
                    match token(meta, ",") {
                        Ok(_) => {},
                        Err(_) => break
                    }
                }
                self.export_defs = exports;
                token(meta, "}")?;
            }
        }
        token(meta, "from")?;
        self.token_path = meta.get_current_token();
        syntax(meta, &mut self.path)?;
        // Import code from file or standard library
        let imported_code = if self.path.value == "[standard library]" {
            self.add_import(meta, "[standard library]")?;
            AmberCompiler::import_std()
        } else {
            self.add_import(meta, &self.path.value.clone())?;
            self.resolve_import(meta)?
        };
        self.handle_import(meta, imported_code)?;
        Ok(())
    }
}

impl TranslateModule for Import {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        "".to_string()
    }
}