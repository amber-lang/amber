use heraclitus_compiler::prelude::*;
use crate::modules::types::Type;
use crate::modules::variable::variable_name_extensions;
use crate::utils::function_interface::FunctionInterface;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::block::Block;
use crate::modules::types::parse_type;

use super::declaration_utils::*;

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub arg_refs: Vec<bool>,
    pub arg_names: Vec<String>,
    pub arg_types: Vec<Type>,
    pub returns: Type,
    pub body: Block,
    pub id: usize,
    pub is_public: bool
}

impl FunctionDeclaration {
    fn set_args_as_variables(&self, meta: &mut TranslateMetadata) -> Option<String> {
        if !self.arg_names.is_empty() {
            meta.increase_indent();
            let mut result = vec![];
            for (index, name) in self.arg_names.clone().iter().enumerate() {
                let indent = meta.gen_indent();
                result.push(format!("{indent}{name}=${}", index + 1));
            }
            meta.decrease_indent();
            Some(result.join("\n"))
        } else { None }
    }
}

impl SyntaxModule<ParserMetadata> for FunctionDeclaration {
    syntax_name!("Function Declaration");

    fn new() -> Self {
        FunctionDeclaration {
            name: String::new(),
            arg_names: vec![],
            arg_types: vec![],
            arg_refs: vec![],
            returns: Type::Generic,
            body: Block::new(),
            id: 0,
            is_public: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Check if this function is public
        if token(meta, "pub").is_ok() {
            self.is_public = true;
        }
        token(meta, "fun")?;
        // Get the function name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        handle_existing_function(meta, tok.clone())?;
        context!({
            // Get the arguments
            token(meta, "(")?;
            loop {
                if token(meta, ")").is_ok() {
                    break
                }
                let is_ref = token(meta, "ref").is_ok();
                let name = variable(meta, variable_name_extensions())?;
                // Optionally parse the argument type
                match token(meta, ":") {
                    Ok(_) => {
                        self.arg_refs.push(is_ref);
                        self.arg_names.push(name.clone());
                        self.arg_types.push(parse_type(meta)?);
                    },
                    Err(_) => {
                        self.arg_refs.push(is_ref);
                        self.arg_names.push(name.clone());
                        self.arg_types.push(Type::Generic);
                    }
                }
                match token(meta, ")") {
                    Ok(_) => break,
                    Err(_) => token(meta, ",")?
                };
            }
            // Optionally parse the return type
            match token(meta, ":") {
                Ok(_) => self.returns = parse_type(meta)?,
                Err(_) => self.returns = Type::Text
            }
            // Parse the body
            token(meta, "{")?;
            let (index_begin, index_end) = skip_function_body(meta);
            // Create a new context with the function body
            let expr = meta.context.expr[index_begin..index_end].to_vec();
            let ctx = meta.context.clone().function_invocation(expr);
            token(meta, "}")?;
            // Add the function to the memory
            self.id = handle_add_function(meta, tok, FunctionInterface {
                id: None,
                name: self.name.clone(),
                arg_names: self.arg_names.clone(),
                arg_types: self.arg_types.clone(),
                arg_refs: self.arg_refs.clone(),
                returns: self.returns.clone(),
                is_public: self.is_public
            }, ctx)?;
            Ok(())
        }, |pos| {
            error_pos!(meta, pos, format!("Failed to parse function declaration '{}'", self.name))
        })
    }
}

impl TranslateModule for FunctionDeclaration {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let mut result = vec![];
        let blocks = meta.fun_cache.get_instances_cloned(self.id).unwrap();
        // Translate each one of them
        for (index, function) in blocks.iter().enumerate() {
            let name = format!("__{}_v{}", self.id, index);
            // Parse the function body
            result.push(format!("function {} {{", name));
            if let Some(args) = self.set_args_as_variables(meta) {
                result.push(args); 
            }
            result.push(function.block.translate(meta));
            result.push(meta.gen_indent() + "}");
        }
        // Return the translation
        result.join("\n")
    }
}