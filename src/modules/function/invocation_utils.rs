use heraclitus_compiler::prelude::*;
use similar_string::find_best_similarity;
use crate::modules::types::Type;
use crate::utils::ParserMetadata;
use crate::modules::block::Block;

fn run_function_with_args(meta: &mut ParserMetadata, name: &str, args: &[Type], tok: Option<Token>) -> Result<usize, Failure> {
    let function = meta.mem.get_function(name).unwrap().clone();
    let mut block = Block::new();
    // Create a new parser metadata specific for the function parsing context
    let mut new_meta = function.meta.clone();
    let function_ctx = new_meta.function_ctx;
    new_meta.expr = function.body.clone();
    new_meta.set_index(0);
    new_meta.function_ctx = true;
    new_meta.mem.set_function_map(meta);
    // Check if the function can exist
    if function.typed {
        if function.args.len() != args.len() {
            return error!(meta, tok, format!("Function '{}' expects {} arguments, but {} were given", name, function.args.len(), args.len()))
        }
        for (index, (arg, kind)) in function.args.iter().enumerate() {
            if kind != &args[index] {
                return error!(meta, tok, format!("Argument '{}' of function '{}' expects type '{}', but '{}' was given", arg, name, kind, args[index]))
            }
        }
    }
    // Create a sub context for new variables
    new_meta.mem.push_scope();
    for (kind, (name, _generic)) in args.iter().zip(function.args.iter()) {
        new_meta.mem.add_variable(name, kind.clone());
    }
    // Parse the function body
    syntax(&mut new_meta, &mut block)?;
    // Pop function body
    new_meta.mem.pop_scope();
    new_meta.function_ctx = function_ctx;
    // Update function map
    meta.mem.set_function_map(&new_meta);
    // Persist the new function instance
    Ok(meta.mem.add_function_instance(function.id, args, function.returns,  block))
}

pub fn handle_function_reference(meta: &mut ParserMetadata, tok: Option<Token>, name: &str) -> Result<(), Failure> {
    if meta.mem.get_function(name).is_none() {
        let message = format!("Function '{}' does not exist", name);
        // Find other similar variable if exists
        return if let Some(comment) = handle_similar_function(meta, name) {
            error!(meta, tok, message, comment)
        } else {
            error!(meta, tok, message)
        }
    }
    Ok(())
}

pub fn handle_function_parameters(meta: &mut ParserMetadata, name: &str, args: &[Type], tok: Option<Token>) -> Result<(Type, usize), Failure> {
    let function_unit = meta.mem.get_function(name).unwrap().clone();
    // TODO: Here is a good place to insert trace
    Ok((function_unit.returns, run_function_with_args(meta, name, args, tok)?))
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