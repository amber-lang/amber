use heraclitus_compiler::prelude::*;
use crate::utils::{metadata::ParserMetadata, context::VariableDecl};
use similar_string::find_best_similarity;
use crate::modules::types::{Typed, Type};

use super::expression::expr::Expr;

pub mod init;
pub mod set;
pub mod get;

pub fn variable_name_extensions() -> Vec<char> {
    vec!['_']
}

pub fn variable_name_keywords() -> Vec<&'static str> {
    vec![
        // Literals
        "true", "false", "null",
        // Variable keywords
        "let", "as", "is",
        // Control flow keywords
        "if", "then", "else",
        // Loop keywords
        "loop", "break", "continue", "in",
        // Module keywords
        "pub", "import", "from",
        // Function keywords
        "fun", "return", "ref", "fail", "failed",
        // Types
        "Text", "Number", "Bool", "Null",
        // Command Modifiers
        "silent", "unsafe",
        // Misc
        "echo", "status", "nameof"
    ]
}


pub fn handle_variable_reference(meta: &ParserMetadata, tok: Option<Token>, name: &str) -> Result<VariableDecl, Failure> {
    handle_identifier_name(meta, name, tok.clone())?;
    match meta.get_var(name) {
        Some(variable_unit) => Ok(variable_unit.clone()),
        None => {
            let message = format!("Variable '{}' does not exist", name);
            // Find other similar variable if exists
            if let Some(comment) = handle_similar_variable(meta, name) {
                error!(meta, tok, message, comment)
            } else {
                error!(meta, tok, message)
            }
        }
    }
}

fn handle_similar_variable(meta: &ParserMetadata, name: &str) -> Option<String> {
    let vars = Vec::from_iter(meta.get_var_names());
    find_best_similarity(name, &vars)
        .and_then(|(match_name, score)| (score >= 0.75).then(|| format!("Did you mean '{match_name}'?")))
}

pub fn handle_identifier_name(meta: &ParserMetadata, name: &str, tok: Option<Token>) -> Result<(), Failure> {
    // Validate if the variable name uses the reserved prefix
    if name.chars().take(2).all(|chr| chr == '_') && name.len() > 2 {
        let new_name = name.get(1..).unwrap();
        return error!(meta, tok => {
            message: format!("Indentifier '{name}' is not allowed"),
            comment: format!("Identifiers with double underscores are reserved for the compiler.\nConsider using '{new_name}' instead.")
        })
    }
    // Validate if the variable name is a keyword
    if variable_name_keywords().contains(&name) {
        return error!(meta, tok, format!("Indentifier '{name}' is a reserved keyword"))
    }
    Ok(())
}

pub fn handle_index_accessor(meta: &mut ParserMetadata) -> Result<Option<Expr>, Failure> {
    if token(meta, "[").is_ok() {
        let tok = meta.get_current_token();
        let mut index = Expr::new();
        syntax(meta, &mut index)?;
        if index.get_type() != Type::Num {
            return error!(meta, tok => {
                message: format!("Index accessor must be a number"),
                comment: format!("The index accessor must be a number, not a {}", index.get_type())
            })
        }
        token(meta, "]")?;
        return Ok(Some(index));
    }
    Ok(None)
}