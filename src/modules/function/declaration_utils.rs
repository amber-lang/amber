use heraclitus_compiler::prelude::*;
use crate::modules::types::Type;
use crate::utils::ParserMetadata;
use crate::modules::variable::{handle_identifier_name};
use crate::utils::cc_flags::{CCFlags, get_ccflag_name};
use crate::utils::context::Context;
use crate::utils::function_interface::FunctionInterface;

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
    if meta.get_fun_declaration(&name).is_some() {
        return error!(meta, tok, format!("Function '{}' already exists", name))
    }
    Ok(())
}

pub fn handle_add_function(meta: &mut ParserMetadata, tok: Option<Token>, fun: FunctionInterface, ctx: Context) -> Result<usize, Failure> {
    let name = fun.name.clone();
    handle_identifier_name(meta, &name, tok.clone())?;
    let any_generic = fun.arg_types.iter().any(|kind| kind == &Type::Generic);
    let any_typed = fun.arg_types.iter().any(|kind| kind != &Type::Generic);
    // Either all arguments are generic or typed
    if any_typed && any_generic {
        return error!(meta, tok => {
            message: format!("Function '{}' has a mix of generic and typed arguments", name),
            comment: "Please decide whether to use generics or types for all arguments"
        })
    }
    if any_typed && fun.returns == Type::Generic && !meta.context.cc_flags.contains(&CCFlags::AllowGenericReturn) {
        let flag_name = get_ccflag_name(CCFlags::AllowGenericReturn);
        let message = Message::new_warn_at_token(meta, tok.clone())
            .message("Function has typed arguments but a generic return type")
            .comment(format!("To surpress this warning, specify a return type for the function '{name}' or use #[{flag_name}] before the parent function declaration"));
        meta.add_message(message);
    }
    // Try to add the function to the memory
    match meta.add_fun_declaration(fun, ctx) {
        // Return the id of the function
        Some(id) => Ok(id),
        // If the function already exists, show an error
        None => error!(meta, tok, format!("Function '{}' already exists", name))
    }
}