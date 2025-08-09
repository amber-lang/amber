use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed, try_parse_type};
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Nameof {
    name: String,
    global_id: Option<usize>,
    is_function: bool,
    function_id: Option<usize>,
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
            global_id: None,
            is_function: false,
            function_id: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "nameof")?;
        
        // Check if we have parentheses: nameof(identifier) 
        let has_parentheses = meta.get_current_token()
            .map(|t| t.word == "(")
            .unwrap_or(false);
        
        if has_parentheses {
            // Parse: nameof(identifier)
            token(meta, "(")?;
            let name = variable(meta, variable_name_extensions())?;
            token(meta, ")")?;
            
            // Try to find as function first, then as variable
            if let Some(fun_decl) = meta.get_fun_declaration(&name) {
                self.name.clone_from(&fun_decl.name);
                self.function_id = Some(fun_decl.id);
                self.is_function = true;
                self.global_id = Some(fun_decl.id);
                Ok(())
            } else if let Some(var_decl) = meta.get_var(&name) {
                self.name.clone_from(&var_decl.name);
                self.global_id = var_decl.global_id;
                self.is_function = false;
                Ok(())
            } else {
                let tok = meta.get_current_token();
                error!(meta, tok, format!("Variable or function '{name}' not found"))
            }
        } else {
            // Parse: nameof identifier or nameof identifier(types...)
            let name = variable(meta, variable_name_extensions())?;
            
            // Check if we have function signature: identifier(Type1, Type2, ...)
            let has_function_signature = meta.get_current_token()
                .map(|t| t.word == "(")
                .unwrap_or(false);
            
            if has_function_signature {
                // Parse function signature: identifier(Type1, Type2, ...)
                token(meta, "(")?;
                let mut arg_types = Vec::new();
                
                // Parse argument types
                while meta.get_current_token().map(|t| t.word != ")").unwrap_or(false) {
                    let arg_type = try_parse_type(meta).map_err(|_| {
                        let tok = meta.get_current_token();
                        Failure::Loud(Message::new_err_at_token(meta, tok).message("Expected a type"))
                    })?;
                    arg_types.push(arg_type);
                    
                    // Check for comma or end of argument list
                    if meta.get_current_token().map(|t| t.word == ",").unwrap_or(false) {
                        token(meta, ",")?;
                    } else {
                        break;
                    }
                }
                
                token(meta, ")")?;
                
                // Find function with matching signature
                if let Some(fun_decl) = meta.get_fun_declaration(&name) {
                    // Check if argument types match
                    if arg_types.len() == fun_decl.arg_types.len() &&
                       arg_types.iter().zip(&fun_decl.arg_types).all(|(provided, expected)| {
                           provided == expected || provided.is_allowed_in(expected)
                       }) {
                        self.name.clone_from(&fun_decl.name);
                        self.function_id = Some(fun_decl.id);
                        self.is_function = true;
                        self.global_id = Some(fun_decl.id);
                        Ok(())
                    } else {
                        let tok = meta.get_current_token();
                        error!(meta, tok, format!("Function '{name}' with signature ({}) not found", 
                            arg_types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")))
                    }
                } else {
                    let tok = meta.get_current_token();
                    error!(meta, tok, format!("Function '{name}' not found"))
                }
            } else {
                // Parse: nameof identifier (try function first, then variable for backward compatibility)
                if let Some(fun_decl) = meta.get_fun_declaration(&name) {
                    self.name.clone_from(&fun_decl.name);
                    self.function_id = Some(fun_decl.id);
                    self.is_function = true;
                    self.global_id = Some(fun_decl.id);
                    Ok(())
                } else if let Some(var_decl) = meta.get_var(&name) {
                    self.name.clone_from(&var_decl.name);
                    self.global_id = var_decl.global_id;
                    self.is_function = false;
                    Ok(())
                } else {
                    let tok = meta.get_current_token();
                    error!(meta, tok, format!("Variable or function '{name}' not found"))
                }
            }
        }
    }
}

impl TranslateModule for Nameof {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        if self.is_function {
            // For functions, generate the name in the format: {name}__{id}_v{variant_id}
            // For now, use variant_id 0 as the default variant
            let function_name = format!("{}__{}_v0", self.name, self.function_id.unwrap_or(0));
            RawFragment::new(&function_name).to_frag()
        } else {
            // For variables, use the existing behavior
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
