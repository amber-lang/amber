use heraclitus_compiler::prelude::*;
use similar_string::find_best_similarity;
use crate::modules::Type;
use crate::utils::{ParserMetadata, error::get_error_logger};
use super::block::Block;
use super::variable::handle_identifier_name;

pub mod declaration;
pub mod invocation;

pub fn skip_function_body(meta: &mut ParserMetadata) {
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
}

pub fn handle_existing_function(meta: &mut ParserMetadata, tok: Option<Token>) {
    let name = tok.as_ref().unwrap().word.clone();
    if let Some(_) = meta.mem.get_function(&name) {
        let message = format!("Function '{}' already exists", name);
        let details = ErrorDetails::from_token_option(tok);
        get_error_logger(meta, details)
            .attach_message(message)
            .show()
            .exit();
    }
}

pub fn handle_add_function(meta: &mut ParserMetadata, name: &str, args: &[(String, Type)], returns: Type, tok: Option<Token>, body: Vec<Token>) -> usize {
    handle_identifier_name(meta, name, tok.clone());
    let any_generic = args.iter().any(|(_, kind)| kind == &Type::Generic);
    let any_typed = args.iter().any(|(_, kind)| kind != &Type::Generic);
    // Either all arguments are generic or typed
    if any_typed && (any_generic || returns == Type::Generic) {
        get_error_logger(meta, ErrorDetails::from_token_option(tok.clone()))
            .attach_message(format!("Function '{}' has a mix of generic and typed arguments", name))
            .attach_comment("Please decide whether to use generics or types for all arguments")
            .show()
            .exit();
    }
    // Try to add the function to the memory
    match meta.mem.add_function(name, args, returns, body) {
        // Return the id of the function
        Some(id) => id,
        // If the function already exists, show an error
        None => {
            get_error_logger(meta, ErrorDetails::from_token_option(tok))
                .attach_message(format!("Function '{}' already exists", name))
                .show()
                .exit();
            0
        }
    }
}

pub fn run_function_with_args(meta: &mut ParserMetadata, name: &str, args: &[Type]) -> usize {
    let function = meta.mem.get_function(name).unwrap().clone();
    let mut block = Block::new();
    // Create a new parser metadata specific for the function parsing context
    let mut new_meta = meta.clone();
    new_meta.expr = function.body.clone();
    new_meta.set_index(0);
    // Create a sub context for new variables
    new_meta.mem.push_scope();
    for (kind, (name, _generic)) in args.iter().zip(function.args.iter()) {
        new_meta.mem.add_variable(name, kind.clone());
    }
    // Parse the function body
    if let Ok(()) = syntax(&mut new_meta, &mut block) {
        // Pop function body
        new_meta.mem.pop_scope();
        // Persist the new function instance
        meta.mem.add_function_instance(function.id, &args, Type::Text,  block)
    } else { 0 }
}

pub fn handle_function_reference(meta: &mut ParserMetadata, tok: Option<Token>, name: &str) {
    if meta.mem.get_function(&name).is_none() {
        let message = format!("Function '{}' does not exist", name);
        let details = ErrorDetails::from_token_option(tok);
        let mut error = get_error_logger(meta, details).attach_message(message);
        // Find other similar variable if exists
        if let Some(comment) = handle_similar_function(meta, name) {
            error = error.attach_comment(comment);
        }
        error.show().exit();
    }
}

pub fn handle_function_parameters(meta: &mut ParserMetadata, name: &str, args: &[Type]) -> (Type, usize) {
    let function_unit = meta.mem.get_function(&name).unwrap().clone();
    // TODO: Here is a good place to insert trace
    (function_unit.returns.clone(), run_function_with_args(meta, name, args))
}

fn handle_similar_function(meta: &mut ParserMetadata, name: &str) -> Option<String> {
    let vars = Vec::from_iter(meta.mem.get_available_functions());
    if let Some((match_name, score)) = find_best_similarity(name, &vars) {
        match score >= 0.75 {
            true => Some(format!("Did you mean '{match_name}'?")),
            false => None
        }
    } else { None }
}