use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{types::Type}};
use similar_string::find_best_similarity;

pub mod init;
pub mod set;
pub mod get;

pub fn variable_name_extensions() -> Vec<char> {
    vec!['_']
}

pub fn variable_name_keywords() -> Vec<&'static str> {
    vec!["true", "false", "null", "if", "loop", "break", "continue", "fun", "else", "let", "pub", "import", "from"]
}


pub fn handle_variable_reference(meta: &mut ParserMetadata, tok: Option<Token>, name: &str) -> Result<(Option<usize>, Type), Failure> {
    handle_identifier_name(meta, name, tok.clone())?;
    match meta.get_var(name) {
        Some(variable_unit) => Ok((variable_unit.global_id, variable_unit.kind.clone())),
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

fn handle_similar_variable(meta: &mut ParserMetadata, name: &str) -> Option<String> {
    let vars = Vec::from_iter(meta.get_var_names());
    if let Some((match_name, score)) = find_best_similarity(name, &vars) {
        match score >= 0.75 {
            true => Some(format!("Did you mean '{match_name}'?")),
            false => None
        }
    } else { None }
}

pub fn handle_identifier_name(meta: &mut ParserMetadata, name: &str, tok: Option<Token>) -> Result<(), Failure> {
    // Validate if the variable name uses the reserved prefix
    if name.chars().take(2).all(|chr| chr == '_') {
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
