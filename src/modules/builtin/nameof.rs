use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Nameof {
    name: String,
    token: Option<Token>,
    global_id: Option<usize>,
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
            token: None,
            global_id: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "nameof")?;
        self.token = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        Ok(())
    }
}

impl TypeCheckModule for Nameof {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        match meta.get_var_used(&self.name) {
            Some(var_decl) => {
                self.name.clone_from(&var_decl.name);
                self.global_id = var_decl.global_id;
            }
            None => {
                return error!(meta, self.token.clone(), format!("Variable '{}' not found", self.name))
            }
        };
        meta.mark_var_modified(&self.name);
        Ok(())
    }
}

impl TranslateModule for Nameof {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        VarExprFragment::new(&self.name, Type::Text)
            .with_global_id(self.global_id)
            .with_render_type(VarRenderType::NameOf)
            .to_frag()
    }
}

impl DocumentationModule for Nameof {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
