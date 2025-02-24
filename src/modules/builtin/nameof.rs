use crate::docs::module::DocumentationModule;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::modules::function::invocation_utils::handle_function_parameters;
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
        if let Some(var_decl) = meta.get_var(&name) {
            self.name = if let Some(id) = var_decl.global_id {
                format!("__{id}_{}", var_decl.name)
            } else {
                var_decl.name.clone()
            };
            Ok(())
        } else if let Some(fun_decl) = meta.get_fun_declaration(&name) {
            self.name = format!("{}__{}_v0", fun_decl.name, fun_decl.id);

            // Create an empty call to ensure referenced function gets built
            let fun_decl2 = fun_decl.clone();
            let _ = handle_function_parameters(meta, fun_decl2.id, fun_decl2.clone(), &fun_decl2.arg_types, &[], None);
            Ok(())
        } else {
            let tok = meta.get_current_token();
            error!(meta, tok, format!("Variable or function '{name}' not found"))
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
