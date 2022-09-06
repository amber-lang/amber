use heraclitus_compiler::prelude::*;
use crate::{utils::{metadata::ParserMetadata, error::get_error_logger}, modules::{Type}};
use similar_string::find_best_similarity;

pub mod init;
pub mod set;
pub mod get;

pub fn variable_name_extensions() -> Vec<char> {
    vec!['_']
}

pub fn variable_name_keywords() -> Vec<&'static str> {
    vec!["true", "false", "null", "if", "loop", "break", "continue", "fun", "else", "let"]
}


pub fn handle_variable_reference(meta: &mut ParserMetadata, tok: Option<Token>, name: &str) -> Type {
    handle_identifier_name(meta, name, tok.clone());
    match meta.mem.get_variable(&name) {
        Some(variable_unit) => variable_unit.kind.clone(),
        None => {
            let message = format!("Variable '{}' does not exist", name);
            let details = ErrorDetails::from_token_option(tok);
            let mut error = get_error_logger(meta, details).attach_message(message);
            // Find other similar variable if exists
            if let Some(comment) = handle_similar_variable(meta, name) {
                error = error.attach_comment(comment);
            }
            error.show().exit();
            Type::Null
        }
    }
}

fn handle_similar_variable(meta: &mut ParserMetadata, name: &str) -> Option<String> {
    let vars = Vec::from_iter(meta.mem.get_available_variables());
    if let Some((match_name, score)) = find_best_similarity(name, &vars) {
        match score >= 0.75 {
            true => Some(format!("Did you mean '{match_name}'?")),
            false => None
        }
    } else { None }
}

pub fn handle_identifier_name(meta: &mut ParserMetadata, name: &str, tok: Option<Token>) {
    // Validate if the variable name uses the reserved prefix
    if name.chars().take(2).all(|chr| chr == '_') {
        let new_name = name.get(1..).unwrap();
        let message = format!("Indentifier '{name}' is not allowed");
        let comment = format!("Identifiers with double underscores are reserved for the compiler.\nConsider using '{new_name}' instead.");
        let details = ErrorDetails::from_token_option(tok.clone());
        get_error_logger(meta, details)
            .attach_message(message)
            .attach_comment(comment)
            .show().exit();
    }
    // Validate if the variable name is a keyword
    if variable_name_keywords().contains(&name) {
        let message = format!("Indentifier '{name}' is a reserved keyword");
        let details = ErrorDetails::from_token_option(tok);
        get_error_logger(meta, details)
            .attach_message(message)
            .show().exit();
    }
}

pub fn handle_add_variable(meta: &mut ParserMetadata, name: &str, kind: Type, tok: Option<Token>) {
    handle_identifier_name(meta, name, tok);
    meta.mem.add_variable(name, kind);
}