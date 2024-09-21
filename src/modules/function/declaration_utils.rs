use heraclitus_compiler::prelude::*;
use crate::modules::types::Type;
use crate::utils::ParserMetadata;
use crate::modules::variable::handle_identifier_name;
use crate::utils::cc_flags::{CCFlags, get_ccflag_name};
use crate::utils::context::Context;
use crate::utils::function_interface::FunctionInterface;

pub fn skip_function_body(meta: &mut ParserMetadata) -> (usize, usize, bool) {
    let index_begin = meta.get_index();
    let mut is_failable = false;
    let mut scope = 1;
    while let Some(tok) = meta.get_current_token() {
        match tok.word.as_str() {
            "{" => scope += 1,
            "}" => scope -= 1,
            "fail" => is_failable = true,
            "?" => is_failable = true,
            _ => {}
        }
        if scope == 0 { break }
        meta.increment_index();
    }
    let index_end = meta.get_index();
    (index_begin, index_end, is_failable)
}

pub fn is_functions_comment_doc(meta: &mut ParserMetadata) -> bool {
    let index = meta.get_index();
    let mut is_comment_doc = true;
    // Multiple linebreaks are merged by heraclitus, so we need to check for them
    let mut last_line = 0;
    if let Some(tok) = meta.get_current_token() {
        if !tok.word.starts_with("///") {
            return false;
        }
    }
    while let Some(tok) = meta.get_current_token() {
        // If there was a longer line break, it means the comment ended
        if !is_comment_doc && tok.pos.0 != last_line + 1 {
            meta.set_index(index);
            return false;
        }
        if tok.word.starts_with("///") {
            is_comment_doc = true;
        }
        if tok.word.starts_with('\n') {
            if is_comment_doc {
                is_comment_doc = false;
                last_line = tok.pos.0;
            } else {
                meta.set_index(index);
                return false;
            }
        }
        if tok.word.starts_with("#[") {
            is_comment_doc = true;
        }
        if tok.word.starts_with("fun") {
            meta.set_index(index);
            return true;
        }
        meta.increment_index();
    }
    false
}

pub fn handle_existing_function(meta: &mut ParserMetadata, tok: Option<Token>) -> Result<(), Failure> {
    let name = tok.as_ref().unwrap().word.clone();
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
            .comment(format!("To suppress this warning, specify a return type for the function '{name}' or use '{flag_name}' compiler flag"));
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
