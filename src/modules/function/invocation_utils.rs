use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::Type;
use crate::utils::context::{FunctionDecl, VariableDecl, VariableDeclWarn};
use crate::utils::{pluralize, ParserMetadata};
use heraclitus_compiler::prelude::*;
use itertools::izip;
use similar_string::find_best_similarity;

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

pub fn run_function_with_args(
    meta: &mut ParserMetadata,
    mut fun: FunctionDecl,
    args: &[Type],
    tok: Option<Token>,
) -> Result<(Type, usize), Failure> {
    // Check if there are the correct amount of arguments
    if fun.args.len() != args.len() {
        let max_args = fun.args.len();
        let min_args =
            fun.args.len() - fun.args.iter().filter(|arg| arg.optional.is_some()).count();
        let opt_argument = if max_args > min_args {
            &format!(" ({max_args} optional)")
        } else {
            ""
        };
        // Determine the correct grammar
        let txt_arguments = pluralize(min_args, "argument", "arguments");
        let txt_given = pluralize(args.len(), "was given", "were given");
        // Return an error
        return error!(
            meta,
            tok,
            format!(
                "Function '{}' expects {} {txt_arguments}{opt_argument}, but {} {txt_given}",
                fun.name,
                min_args,
                args.len()
            )
        );
    }
    // Check if the function argument types match
    if fun.is_args_typed {
        for (index, (arg, given_type)) in izip!(fun.args.iter(), args.iter()).enumerate() {
            let arg_name = &arg.name;
            let arg_type = &arg.kind;
            if !given_type.is_allowed_in(arg_type) {
                let fun_name = &fun.name;
                let ordinal = ordinal_number(index);
                return error!(meta, tok, format!("{ordinal} argument '{arg_name}' of function '{fun_name}' expects type '{arg_type}', but '{given_type}' was given"));
            }
        }
    }
    let mut context = meta.fun_cache.get_context(fun.id).unwrap().clone();
    let mut block = meta
        .fun_cache
        .get_block(fun.id)
        .unwrap()
        .clone()
        .with_needs_noop()
        .with_no_syntax();
    let mut args_global_ids = vec![];

    // Check if the function is already being parsed (recursion)
    // If so, return the variant id that is currently being parsed
    if let Some(variant_id) = meta.parsing_functions.get(&(fun.id, args.to_vec())) {
        return Ok((fun.returns.clone(), *variant_id));
    }

    // Calculate the variant id
    let variant_id = meta.fun_cache.get_instances(fun.id).unwrap().len();
    meta.parsing_functions.insert((fun.id, args.to_vec()), variant_id);

    // Update the function's global scope with the current global scope's functions to support forward references (mutual recursion)
    if let Some(current_global_scope) = meta.context.scopes.first() {
        if let Some(fun_global_scope) = context.scopes.first_mut() {
            for (name, decl) in &current_global_scope.funs {
                if !fun_global_scope.funs.contains_key(name) {
                    fun_global_scope.funs.insert(name.clone(), decl.clone());
                }
            }
        }
    }
    // Capture caller trace and path for the correct trace in errors
    let caller_trace = meta.context.trace.clone();
    let caller_path = meta.context.path.clone();
    let call_site_pos = tok.as_ref().map(|t| PositionInfo::from_token(meta, Some(t.clone())));

    // Swap the contexts to use the function context
    let res = meta.with_context_ref(&mut context, |meta| {
        // Create a sub context for new variables
        meta.with_push_scope(true, |meta| {
            // Add the function itself to the scope to allow recursion
            meta.context.scopes.last_mut().unwrap().add_fun(fun.clone());

            for (kind, arg) in izip!(args, &fun.args) {
                let var = VariableDecl::new(arg.name.clone(), kind.clone())
                    .with_warn(VariableDeclWarn::from_token(meta, tok.clone()))
                    .with_ref(arg.is_ref);
                args_global_ids.push(meta.add_var(var));
            }
            // Set the expected return type if specified
            if fun.returns != Type::Generic {
                meta.context.fun_ret_type = Some(fun.returns.clone());
            }
            // Typecheck the function body
            if let Err(failure) = block.typecheck(meta) {
                match failure {
                    Failure::Loud(mut msg) => {
                        let mut new_trace = Vec::new();
                        if caller_path != meta.context.path {
                            new_trace.extend(caller_trace.clone());
                        }
                        if let Some(ref pos) = call_site_pos {
                            new_trace.push(pos.clone());
                        }
                        new_trace.extend(msg.trace);
                        msg.trace = new_trace;
                        return Err(Failure::Loud(msg));
                    }
                    _ => return Err(failure),
                }
            }
            Ok(())
        })?;
        Ok(())
    });

    meta.parsing_functions.remove(&(fun.id, args.to_vec()));
    res?;

    // Set the new return type or null if nothing was returned
    if let Type::Generic = fun.returns {
        fun.returns = context.fun_ret_type.clone().unwrap_or(Type::Null);
    };
    // Set the new argument types
    for (arg, new_type) in fun.args.iter_mut().zip(args.iter()) {
        arg.kind = new_type.clone();
    }
    // Persist the new function instance
    Ok((
        fun.returns.clone(),
        meta.add_fun_instance(fun.into_interface(), args_global_ids, block),
    ))
}

pub fn handle_function_reference(
    meta: &ParserMetadata,
    tok: Option<Token>,
    name: &str,
) -> Result<usize, Failure> {
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

pub fn handle_function_parameters(
    meta: &mut ParserMetadata,
    id: usize,
    fun: FunctionDecl,
    args: &[Type],
    vars: &[bool],
    tok: Option<Token>,
) -> Result<(Type, usize), Failure> {
    // Check if the function arguments that are references are passed as variables and not as values
    for (index, (arg, var)) in izip!(fun.args.iter(), vars.iter()).enumerate() {
        let is_ref = arg.is_ref;
        let arg_name = &arg.name;
        if is_ref && !var {
            let fun_name = &fun.name;
            let ordinal = ordinal_number(index);
            return error!(meta, tok, format!("Cannot pass {ordinal} argument '{arg_name}' as a reference to the function '{fun_name}' because it is not a variable"));
        }
    }
    // If the function was previously called with the same arguments, return the cached variant
    match meta
        .fun_cache
        .get_instances(id)
        .unwrap()
        .iter()
        .find(|fun| fun.args == args)
    {
        Some(fun) => Ok((fun.returns.clone(), fun.variant_id)),
        None => Ok(run_function_with_args(meta, fun, args, tok)?),
    }
}

fn handle_similar_function(meta: &ParserMetadata, name: &str) -> Option<String> {
    let vars = Vec::from_iter(meta.get_fun_names());
    find_best_similarity(name, &vars).and_then(|(match_name, score)| {
        (score >= 0.75).then(|| format!("Did you mean '{match_name}'?"))
    })
}
