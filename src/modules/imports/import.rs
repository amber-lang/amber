use std::fs;
use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::modules::block::Block;
use crate::modules::variable::variable_name_extensions;
use crate::stdlib;
use crate::utils::context::{Context, FunctionDecl};
use super::import_string::ImportString;

#[derive(Debug, Clone)]
pub struct Import {
    path: ImportString,
    token_import: Option<Token>,
    token_path: Option<Token>,
    is_all: bool,
    is_pub: bool,
    export_defs: Vec<(String, Option<String>, Option<Token>)>
}

impl Import {
    fn handle_export(&mut self, meta: &mut ParserMetadata, mut pub_funs: Vec<FunctionDecl>) -> SyntaxResult {
        if !self.is_all {
            for def in self.export_defs.iter() {
                let (name, alias, tok) = def.clone();
                let fun = match pub_funs.iter_mut().find(|fun| fun.name == name) {
                    Some(fun) => fun,
                    // Check if the function that is being imported is defined
                    None => return error!(meta, tok.clone() => {
                        message: format!("Function '{}' is not defined", name)
                    })
                };
                if let Some(alias) = alias {
                    fun.name = alias;
                }
                fun.is_public = self.is_pub;
                let name = fun.name.clone();
                // Check if current function name is already defined
                if meta.add_fun_declaration_existing(fun.clone()).is_none() {
                    return error!(meta, self.token_import.clone() => {
                        message: format!("Function '{}' is already defined", name)
                    })
                }
            }
        } else {
            for mut fun in pub_funs {
                // Determine if imported functions should be exported further
                fun.is_public = self.is_pub;
                let name = fun.name.clone();
                if meta.add_fun_declaration_existing(fun).is_none() {
                    return error!(meta, self.token_import.clone() => {
                        message: format!("Function '{}' is already defined", name)
                    })
                }
            }
        }
        Ok(())
    }

    fn add_import(&mut self, meta: &mut ParserMetadata, path: &str) -> SyntaxResult {
        if meta.import_cache.add_import_entry(meta.get_path(), path.to_string()).is_none() {
            return error!(meta, self.token_path.clone() => {
                message: "Circular import detected",
                comment: "Please remove the circular import"
            })
        }
        Ok(())
    }

    fn resolve_import(&mut self, meta: &ParserMetadata) -> Result<String, Failure> {
        if self.path.value.starts_with("std/") {
            match stdlib::resolve(self.path.value.replace("std/", "")) {
                Some(v) => Ok(v),
                None => error!(meta, self.token_path.clone(),
                    format!("Standard library module '{}' does not exist", self.path.value))
            }
        } else {
            match fs::read_to_string(self.path.value.clone()) {
                Ok(content) => Ok(content),
                Err(err) => error!(meta, self.token_path.clone() => {
                    message: format!("Could not read file '{}'", self.path.value),
                    comment: err.to_string()
                })
            }
        }
    }

    fn handle_import(&mut self, meta: &mut ParserMetadata, code: String) -> SyntaxResult {
        // If the import was already cached, we don't need to recompile it
        match meta.import_cache.get_import_pub_funs(Some(self.path.value.clone())) {
            Some(pub_funs) => self.handle_export(meta, pub_funs),
            None => self.handle_compile_code(meta, code)
        }
    }

    fn handle_compile_code(&mut self, meta: &mut ParserMetadata, code: String) -> SyntaxResult {
        let options = CompilerOptions::default();
        let compiler = AmberCompiler::new(code, Some(self.path.value.clone()), options);
        match compiler.tokenize() {
            Ok(tokens) => {
                let mut block = Block::new().with_no_syntax();
                // Save snapshot of current file
                let position = PositionInfo::from_token(meta, self.token_import.clone());
                let mut context = Context::new(Some(self.path.value.clone()), tokens)
                    .file_import(&meta.context.trace, position);
                meta.with_context_ref(&mut context, |meta| {
                    // Parse imported code
                    syntax(meta, &mut block)?;
                    block.typecheck(meta)
                })?;
                // Persist compiled file to cache
                meta.import_cache.add_import_metadata(Some(self.path.value.clone()), block, context.pub_funs.clone());
                // Handle exports (add to current file)
                self.handle_export(meta, context.pub_funs)?;
                Ok(())
            }
            Err(err) => Err(Failure::Loud(err))
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
            is_pub: false,
            export_defs: vec![]
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.is_pub = token(meta, "pub").is_ok();
        self.token_import = meta.get_current_token();
        token(meta, "import")?;
        match token(meta, "*") {
            Ok(_) => self.is_all = true,
            Err(_) => {
                token(meta, "{")?;
                let mut exports = vec![];
                if token(meta, "}").is_err() {
                    loop {
                        // Skip comments and newlines
                        if token_by(meta, |token| token.starts_with("//") || token.starts_with('\n')).is_ok() {
                            continue;
                        }
                        let tok = meta.get_current_token();
                        // Check for incorrect use of '*' inside import closure
                        if token(meta, "*").is_ok() {
                            return error!(meta, tok => {
                                message: "Invalid use of '*' in import closure",
                                comment: "Did you mean to 'import * from' instead?"
                            });
                        }
                        let name = variable(meta, variable_name_extensions())?;
                        let alias = match token(meta, "as") {
                            Ok(_) => Some(variable(meta, variable_name_extensions())?),
                            Err(_) => None
                        };
                        exports.push((name, alias, tok));
                        if token(meta, "}").is_ok() {
                            break;
                        }
                        match token(meta, ",") {
                            Ok(_) => {
                                // Skip comments and newlines after comma
                                while token_by(meta, |token| token.starts_with("//") || token.starts_with('\n')).is_ok() {
                                    // Keep consuming
                                }
                                if token(meta, "}").is_ok() {
                                    break
                                }
                            }
                            Err(_) => {
                                return error!(meta, meta.get_current_token(), "Expected ',' or '}' after import");
                            }
                        }
                    }
                } else {
                    let message = Message::new_warn_at_token(meta, self.token_import.clone())
                        .message("Empty import statement");
                    meta.add_message(message);
                }
                self.export_defs = exports;
            }
        }
        token(meta, "from")?;
        self.token_path = meta.get_current_token();
        syntax(meta, &mut self.path)?;
        Ok(())
    }
}

impl TypeCheckModule for Import {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if !meta.is_global_scope() {
            return error!(meta, self.token_import.clone(), "Imports must be in the global scope")
        }
        self.add_import(meta, &self.path.value.clone())?;
        let code = self.resolve_import(meta)?;
        self.handle_import(meta, code)?;
        Ok(())
    }
}

impl TranslateModule for Import {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        FragmentKind::Empty
    }
}

impl DocumentationModule for Import {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
