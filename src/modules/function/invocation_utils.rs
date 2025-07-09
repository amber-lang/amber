use itertools::izip;
use heraclitus_compiler::prelude::*;
use similar_string::find_best_similarity;
use crate::modules::block::Block;
use crate::modules::types::Type;
use crate::utils::{pluralize, ParserMetadata};
use crate::utils::context::FunctionDecl;

// Convert a number to an ordinal number
// Eg. 1 -> 1st, 2 -> 2nd, 3 -> 3rd, 4 -> 4th
fn ordinal_number(index: usize) -> String {
    let index = index + 1;
    let mut result = index.to_string();
    let last_digit = index % 10;
    if last_digit == 1 {
        result.push_str("st");
    } else if last_digit == 2 {
        result.push_str("nd");
    } else if last_digit == 3 {
        result.push_str("rd");
    } else {
        result.push_str("th");
    }
    result
}

fn run_function_with_args(meta: &mut ParserMetadata, mut fun: FunctionDecl, args: &[Type], tok: Option<Token>) -> Result<(Type, usize), Failure> {
    // Check if there are the correct amount of arguments
    if fun.arg_names.len() != args.len() {
        let max_args = fun.arg_names.len();
        let min_args = fun.arg_names.len() - fun.arg_optionals.len();
        let opt_argument = if max_args > min_args {&format!(" ({max_args} optional)")} else {""};
        // Determine the correct grammar
        let txt_arguments = pluralize(min_args, "argument", "arguments");
        let txt_given = pluralize(args.len(), "was given", "were given");
        // Return an error
        return error!(meta, tok, format!("Function '{}' expects {} {txt_arguments}{opt_argument}, but {} {txt_given}", fun.name, min_args, args.len()))
    }
    // Check if the function argument types match
    if fun.is_args_typed {
        for (index, (arg_name, arg_type, given_type)) in izip!(fun.arg_names.iter(), fun.arg_types.iter(), args.iter()).enumerate() {
            if !given_type.is_allowed_in(arg_type) {
                let fun_name = &fun.name;
                let ordinal = ordinal_number(index);
                return error!(meta, tok, format!("{ordinal} argument '{arg_name}' of function '{fun_name}' expects type '{arg_type}', but '{given_type}' was given"))
            }
        }
    }
    let mut context = meta.fun_cache.get_context(fun.id).unwrap().clone();
    let mut block = Block::new().with_needs_noop();
    // Swap the contexts to use the function context
    meta.with_context_ref(&mut context, |meta| {
        // Create a sub context for new variables
        meta.with_push_scope(|meta| {
            for (kind, name, is_ref) in izip!(args, &fun.arg_names, &fun.arg_refs) {
                meta.add_param(name, kind.clone(), *is_ref);
            }
            // Set the expected return type if specified
            if fun.returns != Type::Generic {
                meta.context.fun_ret_type = Some(fun.returns.clone());
            }
            // Parse the function body
            syntax(meta, &mut block)?;
            Ok(())
        })?;
        Ok(())
    })?;
    // Set the new return type or null if nothing was returned
    if let Type::Generic = fun.returns {
        fun.returns = context.fun_ret_type.clone().unwrap_or(Type::Null);
    };
    // Set the new argument types
    fun.arg_types = args.to_vec();
    // Persist the new function instance
    Ok((fun.returns.clone(), meta.add_fun_instance(fun.into_interface(), block)))
}

pub fn handle_function_reference(meta: &ParserMetadata, tok: Option<Token>, name: &str) -> Result<usize, Failure> {
    match meta.get_fun_declaration(name) {
        Some(fun_decl) => Ok(fun_decl.id),
        None => {
            let message = format!("Function '{name}' does not exist");
            // Find other similar variable if exists
            if let Some(comment) = handle_similar_function(meta, name) {
                error!(meta, tok, message, comment)
            } else {
                error!(meta, tok, message)
            }
        }
    }
}

pub fn handle_function_parameters(meta: &mut ParserMetadata, id: usize, fun: FunctionDecl, args: &[Type], vars: &[bool], tok: Option<Token>) -> Result<(Type, usize), Failure> {
    // Check if the function arguments that are references are passed as variables and not as values
    for (index, (is_ref, arg_name, var)) in izip!(fun.arg_refs.iter(), fun.arg_names.iter(), vars.iter()).enumerate() {
        if *is_ref && !var {
            let fun_name = &fun.name;
            let ordinal = ordinal_number(index);
            return error!(meta, tok, format!("Cannot pass {ordinal} argument '{arg_name}' as a reference to the function '{fun_name}' because it is not a variable"))
        }
    }
    // If the function was previously called with the same arguments, return the cached variant
    match meta.fun_cache.get_instances(id).unwrap().iter().find(|fun| fun.args == args) {
        Some(fun) => Ok((fun.returns.clone(), fun.variant_id)),
        None => Ok(run_function_with_args(meta, fun, args, tok)?)
    }
}

fn handle_similar_function(meta: &ParserMetadata, name: &str) -> Option<String> {
    let vars = Vec::from_iter(meta.get_fun_names());
    find_best_similarity(name, &vars)
        .and_then(|(match_name, score)| (score >= 0.75).then(|| format!("Did you mean '{match_name}'?")))
}
