use std::mem::swap;
use itertools::izip;
use heraclitus_compiler::prelude::*;
use similar_string::find_best_similarity;
use crate::modules::types::Type;
use crate::utils::ParserMetadata;
use crate::modules::block::Block;

fn run_function_with_args(meta: &mut ParserMetadata, name: &str, args: &[Type], tok: Option<Token>) -> Result<usize, Failure> {
    let fun = meta.get_fun_declaration(name).unwrap().clone();
    // Check if there are the correct amount of arguments
    if fun.arg_names.len() != args.len() {
        // Determine the correct grammar
        let txt_arguments = if fun.arg_names.len() == 1 { "argument" } else { "arguments" };
        let txt_given = if args.len() == 1 { "was given" } else { "were given" };
        // Return an error
        return error!(meta, tok, format!("Function '{}' expects {} {txt_arguments}, but {} {txt_given}", name, fun.arg_names.len(), args.len()))
    }
    // Check if the function argument types match
    if fun.is_args_typed {
        for (index, (arg, kind)) in fun.arg_names.iter().zip(fun.arg_types.iter()).enumerate() {
            if kind != &args[index] {
                return error!(meta, tok, format!("Argument '{}' of function '{}' expects type '{}', but '{}' was given", arg, name, kind, args[index]))
            }
        }
    }
    let mut ctx = meta.fun_cache.get_context(fun.id).unwrap().clone();
    let mut block = Block::new();
    // Swap the contexts to use the function context
    swap(&mut ctx, &mut meta.context);
    // Create a sub context for new variables
    meta.push_scope();
    for (kind, name, is_ref) in izip!(args, &fun.arg_names, &fun.arg_refs) {
        meta.add_param(name, kind.clone(), is_ref.clone());
    }
    // Parse the function body
    syntax(meta, &mut block)?;
    // Pop function body
    meta.pop_scope();
    // Restore old context
    swap(&mut ctx, &mut meta.context);
    // Persist the new function instance
    Ok(meta.add_fun_instance(fun.to_interface(), block))
}

pub fn handle_function_reference(meta: &mut ParserMetadata, tok: Option<Token>, name: &str) -> Result<usize, Failure> {
    match meta.get_fun_declaration(name) {
        Some(fun_decl) => Ok(fun_decl.id),
        None => {
            let message = format!("Function '{}' does not exist", name);
            // Find other similar variable if exists
            if let Some(comment) = handle_similar_function(meta, name) {
                error!(meta, tok, message, comment)
            } else {
                error!(meta, tok, message)
            }
        }
    }
}

pub fn handle_function_parameters(meta: &mut ParserMetadata, id: usize, name: &str, args: &[Type], tok: Option<Token>) -> Result<(Type, usize), Failure> {
    let function_unit = meta.get_fun_declaration(name).unwrap().clone();
    // If the function was previously called with the same arguments, return the cached variant
    match meta.fun_cache.get_instances(id).unwrap().iter().find(|fun| fun.args == args) {
        Some(fun) => Ok((function_unit.returns, fun.variant_id)),
        None => Ok((function_unit.returns, run_function_with_args(meta, name, args, tok)?))
    }
}

fn handle_similar_function(meta: &mut ParserMetadata, name: &str) -> Option<String> {
    let vars = Vec::from_iter(meta.get_fun_names());
    if let Some((match_name, score)) = find_best_similarity(name, &vars) {
        match score >= 0.75 {
            true => Some(format!("Did you mean '{match_name}'?")),
            false => None
        }
    } else { None }
}