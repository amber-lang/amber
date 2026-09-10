use crate::modules::function::core::monomorphize::{Monomorphizer, Persistence};
use crate::modules::function::core::signature::{
    FunctionDeclId, FunctionSignature, FunctionVariantId,
};

use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::raw_fragment;
use crate::translate::fragments::get_function_name;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "nameof"]
#[kind = "builtin_expr"]
pub struct Nameof {
    name: String,
    token: Option<Token>,
    global_id: Option<usize>,
    function_variant: Option<(FunctionDeclId, FunctionVariantId)>,
}

impl Nameof {
    /// Validates if function declaration is strictly typed
    fn validate_strict_typing(
        &self,
        meta: &ParserMetadata,
        fun_signature: &FunctionSignature,
    ) -> SyntaxResult {
        if !fun_signature
            .params
            .iter()
            .all(|param| param.kind.is_strictly_typed())
        {
            return error!(
                meta,
                self.token.clone(),
                format!(
                    "Function '{}' must be strictly typed to be used with 'nameof'.",
                    self.name
                ),
                "All function parameters have to be of concrete type"
            );
        }
        Ok(())
    }
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
        let position = meta.get_index();
        token(meta, "nameof")?;
        self.token = meta.get_current_token();

        if token(meta, "(").is_ok() {
            self.name = variable(meta, variable_name_extensions())?;
            token(meta, ")")?;
        } else {
            let tok = meta.get_token_at(position);
            let warning = Message::new_warn_at_token(meta, tok)
                .message("Calling a builtin without parentheses is deprecated");
            meta.add_message(warning);
            self.name = variable(meta, variable_name_extensions())?;
        }
        Ok(())
    }
}

impl TypeCheckModule for Nameof {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Case for variable `nameof`
        if let Some(var_decl) = meta.get_var_used(&self.name) {
            self.name.clone_from(&var_decl.name);
            self.global_id = var_decl.global_id;
            meta.mark_var_modified(&self.name);
            return Ok(());
        }
        // Case for function `nameof`
        match meta.get_fun_declaration(&self.name).cloned() {
            Some(fun_decl) => {
                // Check if the function is strictly typed
                self.validate_strict_typing(meta, &fun_decl)?;
                let args_types: Vec<Type> = fun_decl.param_types();
                // Fetch a function variant which matches all the param types
                let fun_variant = meta
                    .fun_cache
                    .get_variants(fun_decl.id)
                    .unwrap()
                    .iter()
                    .find(|fun| fun.param_types == args_types);

                // Check if the function variant is already compiled
                let variant_id = match fun_variant {
                    Some(fun_variant) => fun_variant.id,
                    None => {
                        // Compile the function on demand to get the variant ID
                        let persistence = Persistence::from_meta(meta);
                        Monomorphizer::new(meta, fun_decl.clone(), &args_types, self.token.clone())
                            .run(persistence)?
                            .variant_id
                    }
                };

                self.function_variant = Some((fun_decl.id, variant_id));
                Ok(())
            }
            None => {
                error!(
                    meta,
                    self.token.clone(),
                    format!("Variable or function '{}' not found", self.name)
                )
            }
        }
    }
}

impl TranslateModule for Nameof {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        if let Some((id, variant)) = self.function_variant {
            let fun_name = get_function_name(&self.name, id, variant);
            raw_fragment!("{fun_name}")
        } else {
            VarExprFragment::new(&self.name, Type::Text)
                .with_global_id(self.global_id)
                .with_render_type(VarRenderType::NameOf)
                .to_frag()
        }
    }
}

crate::impl_documentation_noop!(Nameof);
