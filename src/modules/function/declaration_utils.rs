use heraclitus_compiler::prelude::*;
use crate::modules::types::Type;
use crate::utils::ParserMetadata;
use crate::modules::variable::{handle_identifier_name};

#[derive(Debug, Clone)]
pub struct FunctionDeclSyntax {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub returns: Type,
    pub body: Vec<Token>,
    pub is_public: bool
}

pub fn skip_function_body(meta: &mut ParserMetadata) -> (usize, usize) {
    let index_begin = meta.get_index();
    let mut scope = 1;
    while let Some(tok) = meta.get_current_token() {
        match tok.word.as_str() {
            "{" => scope += 1,
            "}" => scope -= 1,
            _ => {}
        }
        if scope == 0 { break }
        meta.increment_index();
    }
    let index_end = meta.get_index();
    (index_begin, index_end)
}

pub fn handle_existing_function(meta: &mut ParserMetadata, tok: Option<Token>) -> Result<(), Failure> {
    let name = tok.as_ref().unwrap().word.clone();
    handle_identifier_name(meta, &name, tok.clone())?;
    if meta.mem.get_function(&name).is_some() {
        return error!(meta, tok, format!("Function '{}' already exists", name))
    }
    Ok(())
}

pub fn handle_add_function(meta: &mut ParserMetadata, tok: Option<Token>, decl: FunctionDeclSyntax) -> Result<usize, Failure> {
    let name = decl.name.clone();
    handle_identifier_name(meta, &name, tok.clone())?;
    let any_generic = decl.args.iter().any(|(_, kind)| kind == &Type::Generic);
    let any_typed = decl.args.iter().any(|(_, kind)| kind != &Type::Generic);
    // Either all arguments are generic or typed
    if any_typed && (any_generic || decl.returns == Type::Generic) {
        return error!(meta, tok => {
            message: format!("Function '{}' has a mix of generic and typed arguments", name),
            comment: "Please decide whether to use generics or types for all arguments"
        })
    }
    // Try to add the function to the memory
    match meta.mem.add_function_declaration(meta.clone(), decl) {
        // Return the id of the function
        Some(id) => Ok(id),
        // If the function already exists, show an error
        None => error!(meta, tok, format!("Function '{}' already exists", name))
    }
}