use heraclitus_compiler::prelude::*;
use crate::{utils::{metadata::ParserMetadata, error::get_error_logger}, modules::{Type}};
use similar_string::find_best_similarity;

pub mod init;
pub mod set;
pub mod get;

pub fn variable_name_extensions() -> Vec<char> {
    vec!['_']
}

pub fn handle_variable_reference(meta: &mut ParserMetadata, token: Option<Token>, name: String) -> Type {
    match meta.var_mem.get_variable(name.clone()) {
        Some(variable_unit) => return variable_unit.kind.clone(),
        None => {
            let message = format!("Variable '{}' does not exist", name.clone());
            let details = ErrorDetails::from_token_option(token);
            let mut error = get_error_logger(meta, details).attach_message(message);
            // Find other similar variable if exists
            if let Some(comment) = handle_similar_variable(meta, name.clone()) {
                error = error.attach_comment(comment);
            }
            error.show().exit();
            Type::Void
        }
    }
}

fn handle_similar_variable(meta: &mut ParserMetadata, name: String) -> Option<String> {
    let vars = Vec::from_iter(meta.var_mem.get_available_variables());
    let (match_name, score) = find_best_similarity(name, &vars);
    match score >= 0.75 {
        true => Some(format!("Did you mean '{match_name}'?")),
        false => None
    }
}
