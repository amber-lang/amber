use heraclitus_compiler::prelude::*;
use similar_string::find_best_similarity;
use crate::modules::types::Type;
use crate::utils::{ParserMetadata, error::get_error_logger};
use crate::modules::block::Block;

fn run_function_with_args(meta: &mut ParserMetadata, name: &str, args: &[Type], tok: Option<Token>) -> usize {
    let function = meta.mem.get_function(name).unwrap().clone();
    let mut block = Block::new();
    // Create a new parser metadata specific for the function parsing context
    let mut new_meta = meta.clone();
    let function_ctx = new_meta.function_ctx;
    new_meta.expr = function.body.clone();
    new_meta.set_index(0);
    new_meta.function_ctx = true;
    // Check if the function can exist
    if function.typed {
        if function.args.len() != args.len() {
            let err_details = ErrorDetails::from_token_option(tok.clone());
            let were_was = if args.len() == 1 { "was" } else { "were" };
            let error = format!("Function '{}' expects {} arguments, but {} {were_was} given", name, function.args.len(), args.len());
            get_error_logger(meta, err_details)
                .attach_message(error)
                .show()
                .exit();
        }
        for (index, (arg, kind)) in function.args.iter().enumerate() {
            if kind != &args[index] {
                let err_details = ErrorDetails::from_token_option(tok.clone());
                let error = format!("Function '{}' expects argument '{}' to be of type '{}', but '{}' was given", name, arg, kind, args[index]);
                get_error_logger(meta, err_details)
                    .attach_message(error)
                    .show()
                    .exit();
            }
        }
    }
    // Create a sub context for new variables
    new_meta.mem.push_scope();
    for (kind, (name, _generic)) in args.iter().zip(function.args.iter()) {
        new_meta.mem.add_variable(name, kind.clone());
    }
    // Parse the function body
    if let Ok(()) = syntax(&mut new_meta, &mut block) {
        // Pop function body
        new_meta.mem.pop_scope();
        new_meta.function_ctx = function_ctx;
        // Persist the new function instance
        meta.mem.add_function_instance(function.id, args, function.returns.clone(),  block)
    } else { 0 }
}

pub fn handle_function_reference(meta: &mut ParserMetadata, tok: Option<Token>, name: &str) {
    if meta.mem.get_function(name).is_none() {
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

pub fn handle_function_parameters(meta: &mut ParserMetadata, name: &str, args: &[Type], tok: Option<Token>) -> (Type, usize) {
    let function_unit = meta.mem.get_function(name).unwrap().clone();
    // TODO: Here is a good place to insert trace
    (function_unit.returns, run_function_with_args(meta, name, args, tok))
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