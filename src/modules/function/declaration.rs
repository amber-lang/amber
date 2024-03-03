use std::collections::HashSet;
use std::mem::swap;

use heraclitus_compiler::prelude::*;
use itertools::izip;
use crate::docs::module::DocumentationModule;
use crate::modules::statement::comment_doc::CommentDoc;
use crate::modules::types::Type;
use crate::modules::variable::variable_name_extensions;
use crate::utils::cc_flags::get_ccflag_by_name;
use crate::utils::function_cache::FunctionInstance;
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
    pub is_public: bool,
    pub comment: Option<CommentDoc>
}

impl FunctionDeclaration {
    fn set_args_as_variables(&self, meta: &mut TranslateMetadata, function: &FunctionInstance, arg_refs: &[bool]) -> Option<String> {
        if !self.arg_names.is_empty() {
            meta.increase_indent();
            let mut result = vec![];
            for (index, (name, kind, is_ref)) in izip!(self.arg_names.clone(), &function.args, arg_refs).enumerate() {
                let indent = meta.gen_indent();
                match (is_ref, kind) {
                    (false, Type::Array(_)) => result.push(format!("{indent}local {name}=(\"${{!{}}}\")", index + 1)),
                    (true, Type::Array(_)) => {
                        result.push(format!("{indent}local __AMBER_ARRAY_{name}=\"${}[@]\"", index + 1));
                        result.push(format!("{indent}local {name}=${}", index + 1))
                    },
                    _ => result.push(format!("{indent}local {name}=${}", index + 1))
                }
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
            is_public: false,
            comment: None
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Parse the function comment
        if is_functions_comment_doc(meta) {
            let mut comment = CommentDoc::new();
            syntax(meta, &mut comment)?;
            self.comment = Some(comment);
        }
        let mut flags = HashSet::new();
        // Get all the user-defined compiler flags
        while let Ok(flag) = token_by(meta, |val| val.starts_with("#[")) {
            // Push to the flags vector as it is more safe in case of parsing errors
            flags.insert(get_ccflag_by_name(&flag[2..flag.len() - 1]));
        }
        let tok = meta.get_current_token();
        // Check if this function is public
        if token(meta, "pub").is_ok() {
            self.is_public = true;
        }
        if let Err(err) = token(meta, "fun") {
            if !flags.is_empty() {
                return error!(meta, tok, "Compiler flags can only be used in function declarations")
            }
            return Err(err)
        }
        // Check if we are in the global scope
        if !meta.is_global_scope() {
            return error!(meta, tok, "Functions can only be declared in the global scope")
        }
        // Get the function name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        handle_existing_function(meta, tok.clone())?;
        context!({
            // Set the compiler flags
            swap(&mut meta.context.cc_flags, &mut flags);
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
                let tok = meta.get_current_token();
                if token(meta, "=").is_ok() {
                    return error!(meta, tok, "Default values for function arguments are not yet supported")
                }
                match token(meta, ")") {
                    Ok(_) => break,
                    Err(_) => token(meta, ",")?
                };
            }
            // Optionally parse the return type
            match token(meta, ":") {
                Ok(_) => self.returns = parse_type(meta)?,
                Err(_) => self.returns = Type::Generic
            }
            // Parse the body
            token(meta, "{")?;
            let (index_begin, index_end, is_failable) = skip_function_body(meta);
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
                is_public: self.is_public,
                is_failable
            }, ctx)?;
            // Restore the compiler flags
            swap(&mut meta.context.cc_flags, &mut flags);
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
        let prev_fun_name = meta.fun_name.clone();
        // Translate each one of them
        for (index, function) in blocks.iter().enumerate() {
            let name = format!("{}__{}_v{}", self.name, self.id, index);
            meta.fun_name = Some((self.name.clone(), self.id, index));
            // Parse the function body
            result.push(format!("function {name} {{"));
            if let Some(args) = self.set_args_as_variables(meta, function, &self.arg_refs) {
                result.push(args);
            }
            result.push(function.block.translate(meta));
            result.push(meta.gen_indent() + "}");
        }
        // Restore the function name
        meta.fun_name = prev_fun_name;
        // Return the translation
        result.join("\n")
    }
}

impl DocumentationModule for FunctionDeclaration {
    fn document(&self) -> String {
        // TODO: Implement generating docs for functions
        unimplemented!()
    }
}
