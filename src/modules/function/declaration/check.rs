use super::parse::ReturnTypeDecl;
use crate::modules::block::Block;
use crate::modules::function::core::signature::{FunctionParam, FunctionSignature};
use crate::modules::handle_symbol_scope_declaration;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::Typed;
use crate::modules::variable::handle_identifier_name;
use crate::utils::context::Context;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use std::collections::HashSet;

/// Rejects a function name that is already taken by a variable or function in the current scope.
pub fn handle_existing_function(
    meta: &mut ParserMetadata,
    tok: Option<Token>,
) -> Result<(), Failure> {
    let name = tok.as_ref().unwrap().word.clone();
    handle_symbol_scope_declaration(meta, &name, tok)
}

/// Checks that the body's failability agrees with the `?` marker on the return type
pub fn validate_failability(
    meta: &mut ParserMetadata,
    body_can_fail: bool,
    declared_failable: bool,
    returns: &ReturnTypeDecl,
) -> Result<(), Failure> {
    if body_can_fail && !declared_failable {
        return error!(
            meta,
            returns.type_token.clone(),
            "Failable functions must have a '?' after the type name"
        );
    }
    if !body_can_fail && declared_failable {
        return error!(
            meta,
            returns
                .question_token
                .clone()
                .or(returns.type_token.clone()),
            "Infallible functions must not have a '?' after the type name"
        );
    }
    Ok(())
}

/// Rejects a parameter list that names the same parameter twice.
pub fn validate_unique_param_names(
    meta: &mut ParserMetadata,
    params: &[FunctionParam],
) -> Result<(), Failure> {
    let mut seen = HashSet::new();
    for param in params {
        if !seen.insert(param.name.clone()) {
            return error!(
                meta,
                param.token.clone(),
                format!("Argument '{}' is already defined", param.name)
            );
        }
    }
    Ok(())
}

/// Typechecks default values and enforces rules around optional parameters
pub fn validate_optional_params(
    meta: &mut ParserMetadata,
    params: &mut [FunctionParam],
) -> Result<(), Failure> {
    let mut optional_started = false;
    for param in params {
        let Some(ref mut default) = param.default else {
            if optional_started {
                return error!(
                    meta,
                    param.token.clone(),
                    "All arguments following an optional argument must also be optional"
                );
            }
            continue;
        };

        // Check if ref arguments are optional
        if param.is_ref {
            return error!(meta, param.token.clone(), "A ref cannot be optional");
        }

        // Typecheck the optional argument expression first
        default.typecheck(meta)?;

        // Validate optional argument type
        if !default.get_type().is_allowed_in(&param.kind) {
            return error!(
                meta,
                param.token.clone(),
                "Optional argument does not match annotated type"
            );
        }

        optional_started = true;
    }
    Ok(())
}

/// Registers the signature in the current scope and seeds its cache entry.
pub fn register_function(
    meta: &mut ParserMetadata,
    tok: Option<Token>,
    fun: FunctionSignature,
    ctx: Context,
    block: Block,
) -> Result<(), Failure> {
    let name = fun.name.clone();
    handle_identifier_name(meta, &name, tok.clone())?;
    // Either all arguments are generic or typed
    if fun.has_mixed_typing() {
        return error!(meta, tok => {
            message: format!("Function '{}' has a mix of generic and typed arguments", name),
            comment: "Please decide whether to use generics or types for all arguments"
        });
    }
    // Try to add the function to the memory
    if meta.declare_function(fun, ctx, block) {
        Ok(())
    } else {
        // If the function already exists, show an error
        error!(meta, tok, format!("Function '{}' already exists", name))
    }
}
