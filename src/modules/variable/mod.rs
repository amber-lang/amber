use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::types::{Type, Typed};
use crate::utils::cc_flags::{get_ccflag_name, CCFlags};
use crate::utils::context::VariableDecl;
use crate::utils::metadata::ParserMetadata;
use crate::utils::is_all_caps;
use heraclitus_compiler::prelude::*;
use similar_string::find_best_similarity;

pub mod init;
pub mod set;
pub mod get;

pub fn variable_name_extensions() -> Vec<char> {
    vec!['_']
}

pub fn variable_name_keywords() -> Vec<&'static str> {
    vec![
        "Bool", "Null", "Number", "Text", "and", "as",
        "break", "cd", "const", "continue", "echo",
        "else", "exit", "exited", "fail", "failed",
        "false", "for", "from", "fun", "if",
        "import", "in", "is", "len", "let",
        "lines", "loop", "main", "mv", "nameof", "touch",
        "not", "null", "or", "pub", "ref",
        "return", "silent", "status", "sudo", "succeeded",
        "then", "trust", "true", "unsafe", "while",
    ]
}


pub fn handle_variable_reference(meta: &mut ParserMetadata, tok: &Option<Token>, name: &str) -> Result<VariableDecl, Failure> {
    handle_identifier_name(meta, name, tok.clone())?;
    match meta.get_var_used(name) {
        Some(variable_unit) => Ok(variable_unit.clone()),
        None => {
            let message = format!("Variable '{name}' does not exist");
            // Find other similar variable if exists
            if let Some(comment) = handle_similar_variable(meta, name) {
                error!(meta, tok.clone(), message, comment)
            } else {
                error!(meta, tok.clone(), message)
            }
        }
    }
}

pub fn prevent_constant_mutation(meta: &mut ParserMetadata, tok: &Option<Token>, name: &str, is_const: bool) -> SyntaxResult {
    if is_const {
        error!(meta, tok.clone(), format!("Cannot reassign constant '{name}'"))
    } else {
        Ok(())
    }
}

fn handle_similar_variable(meta: &ParserMetadata, name: &str) -> Option<String> {
    let vars = Vec::from_iter(meta.get_var_names());
    find_best_similarity(name, &vars)
        .and_then(|(match_name, score)| (score >= 0.75).then(|| format!("Did you mean '{match_name}'?")))
}

pub fn handle_identifier_name(meta: &mut ParserMetadata, name: &str, tok: Option<Token>) -> Result<(), Failure> {
    // Validate if the variable name uses the reserved prefix with fully uppercase names
    if name.chars().take(2).all(|chr| chr == '_') && name.len() > 2 && is_all_caps(name) {
        let new_name = name.get(2..).unwrap();
        return error!(meta, tok => {
            message: format!("Identifier '{name}' is not allowed"),
            comment: format!("Identifiers with double underscores cannot be fully uppercase.\nConsider using '{new_name}' instead.")
        })
    }
    if is_camel_case(name) && !meta.context.cc_flags.contains(&CCFlags::AllowCamelCase) {
        let flag = get_ccflag_name(CCFlags::AllowCamelCase);
        let msg = Message::new_warn_at_token(meta, tok.clone())
            .message(format!("Identifier '{name}' is not in snake case"))
            .comment([
                "We recommend using snake case with either all uppercase or all lowercase letters for consistency.",
                format!("To disable this warning use '{flag}' compiler flag").as_str()
            ].join("\n"));
        meta.add_message(msg);
    }
    // Validate if the variable name is a keyword
    if variable_name_keywords().contains(&name) {
        return error!(meta, tok, format!("Identifier '{name}' is a reserved keyword"))
    }
    Ok(())
}

fn is_camel_case(name: &str) -> bool {
    let mut is_lowercase = false;
    let mut is_uppercase = false;
    for chr in name.chars() {
        match chr {
            '_' => continue,
            _ if is_lowercase && is_uppercase => return true,
            _ if chr.is_lowercase() => is_lowercase = true,
            _ if chr.is_uppercase() => is_uppercase = true,
            _ => ()
        }
    }
    if is_lowercase && is_uppercase { return true }
    false
}

pub fn handle_index_accessor(meta: &mut ParserMetadata, _range: bool) -> Result<Option<Expr>, Failure> {
    if token(meta, "[").is_ok() {
        let mut index = Expr::new();
        syntax(meta, &mut index)?;
        token(meta, "]")?;
        return Ok(Some(index));
    }
    Ok(None)
}

pub fn validate_index_accessor(meta: &ParserMetadata, index: &Expr, range: bool, position: PositionInfo) -> SyntaxResult {
    if !allow_index_accessor(index, range) {
        let expected = if range { "integer or range" } else { "integer (and not a range)" };
        let side = if range { "right" } else { "left" };
        let message = format!("Index accessor must be an {expected} for {side} side of operation");
        let comment = format!("The index accessor must be an {} and not {}", expected, index.get_type());
        return error_pos!(meta, position => { message: message, comment: comment });
    }
    Ok(())
}

fn allow_index_accessor(index: &Expr, range: bool) -> bool {
    match (&index.kind, &index.value) {
        (Type::Int, _) => true,
        (Type::Array(_), Some(ExprType::Range(_))) => range,
        _ => false,
    }
}
