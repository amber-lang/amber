use crate::modules::prelude::*;
use crate::modules::function::invocation_utils::run_function_with_args;
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
    function_variant: Option<(usize, usize)>,
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
            function_variant: None,
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
                let fun_decl_opt = meta.get_fun_declaration(&self.name).cloned();

                match fun_decl_opt {
                    Some(fun_decl) => {
                        // Check if the function is strictly typed
                        if !fun_decl.args.iter().all(|arg| arg.kind.is_strictly_typed()) {
                            return error!(meta, self.token.clone(), 
                                format!("Function '{}' must be strictly typed to be used with 'nameof'.", self.name),
                                "All function parameters have to be of concrete type"
                            )
                        }
                        let args_types: Vec<Type> = fun_decl.args.iter().map(|arg| arg.kind.clone()).collect();
                        let fun_instance = meta.fun_cache.get_instances(fun_decl.id).unwrap().iter().find(|fun| fun.args == args_types);
                        
                        // Check if the function variant is already compiled
                        let variant_id = match fun_instance {
                            Some(fun_instance) => fun_instance.variant_id,
                            None => {
                                // Compile the function on demand to get the variant ID
                                run_function_with_args(
                                    meta, 
                                    fun_decl.clone(), 
                                    &args_types, 
                                    self.token.clone()
                                )?.1
                            }
                        };

                        self.function_variant = Some((fun_decl.id, variant_id));
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
        if let Some((id, variant)) = self.function_variant {
            let prefix = meta.gen_variable_prefix(&self.name);
            let name = format!("{prefix}{}__{id}_v{variant}", self.name);
            raw_fragment!("{name}")
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
