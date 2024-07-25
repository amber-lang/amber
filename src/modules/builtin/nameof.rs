use crate::docs::module::DocumentationModule;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Nameof {
    name: String,
}

impl Typed for Nameof {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Nameof {
    syntax_name!("Nameof");

    fn new() -> Self {
        Nameof {
            name: String::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "nameof")?;
        let name = variable(meta, variable_name_extensions())?;
        match meta.get_var(&name) {
            Some(var_decl) => {
                self.name.clone_from(&var_decl.name);
                if let Some(id) = var_decl.global_id {
                    self.name = format!("__{id}_{}", self.name);
                }
                Ok(())
            }
            None => {
                let tok = meta.get_current_token();
                error!(meta, tok, format!("Variable '{name}' not found"))
            }
        }
    }
}

impl TranslateModule for Nameof {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let quote = meta.gen_quote();
        let name = &self.name;
        format!("{quote}{name}{quote}")
    }
}

impl DocumentationModule for Nameof {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
