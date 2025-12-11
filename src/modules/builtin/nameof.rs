use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;
use crate::raw_fragment;

#[derive(Debug, Clone)]
pub struct Nameof {
    name: String,
    token: Option<Token>,
    global_id: Option<usize>,
    function_info: Option<(usize, usize)>,
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
            function_info: None,
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
                meta.mark_var_modified(&self.name);
            }
            None => {
                // If variable not found, try to find a function
                match meta.get_fun_declaration(&self.name) {
                    Some(fun_decl) => {
                        // Check if the function is strictly typed
                        if !fun_decl.args.iter().all(|arg| arg.kind.is_strictly_typed()) {
                            return error!(meta, self.token.clone(), 
                                format!("Function '{}' is not strictly typed", self.name),
                                "All function parameters have to be of concrete type"
                            )
                        }
                        // Since strictly typed functions are compiled eagerly, we can assume variant 0 exists
                        self.function_info = Some((fun_decl.id, 0));
                    }
                    None => return error!(meta, self.token.clone(), format!("Variable or function '{}' not found", self.name))
                }
            }
        };
        Ok(())
    }
}

impl TranslateModule for Nameof {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if let Some((id, variant)) = self.function_info {
            let prefix = meta.gen_variable_prefix(&self.name);
            let name = format!("{}{}__{}_v{}", prefix, self.name, id, variant);
            raw_fragment!("{}", name)
        } else {
            VarExprFragment::new(&self.name, Type::Text)
                .with_global_id(self.global_id)
                .with_render_type(VarRenderType::NameOf)
                .to_frag()
        }
    }
}

impl DocumentationModule for Nameof {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
