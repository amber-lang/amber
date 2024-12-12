use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::types::{Type, Typed};
use crate::utils::cc_flags::{get_ccflag_name, CCFlags};
use crate::utils::context::VariableDecl;
use crate::utils::metadata::ParserMetadata;
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
        // Literals
        "true", "false", "null",
        // Variable keywords
        "let", "as", "is", "const",
        // Control flow keywords
        "if", "then", "else",
        // Loop keywords
        "for", "loop", "break", "continue", "in",
        // Module keywords
        "pub", "import", "from",
        // Function keywords
        "fun", "return", "ref", "fail", "failed",
        // Types
        "Text", "Number", "Bool", "Null",
        // Command Modifiers
        "silent", "trust",
        // Misc
        "echo", "status", "nameof", "mv", "cd",
        "exit", "len",
    ]
}


pub fn handle_variable_reference(meta: &mut ParserMetadata, tok: &Option<Token>, name: &str) -> Result<VariableDecl, Failure> {
    handle_identifier_name(meta, name, tok.clone())?;
    match meta.get_var(name) {
        Some(variable_unit) => Ok(variable_unit.clone()),
        None => {
            let message = format!("Variable '{}' does not exist", name);
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
    // Validate if the variable name uses the reserved prefix
    if name.chars().take(2).all(|chr| chr == '_') && name.len() > 2 {
        let new_name = name.get(1..).unwrap();
        return error!(meta, tok => {
            message: format!("Identifier '{name}' is not allowed"),
            comment: format!("Identifiers with double underscores are reserved for the compiler.\nConsider using '{new_name}' instead.")
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

pub fn handle_index_accessor(meta: &mut ParserMetadata, range: bool) -> Result<Option<Expr>, Failure> {
    if token(meta, "[").is_ok() {
        let tok = meta.get_current_token();
        let mut index = Expr::new();
        syntax(meta, &mut index)?;
        if !allow_index_accessor(&index, range) {
            let expected = if range { "number or range" } else { "number (and not a range)" };
            let side = if range { "right" } else { "left" };
            let message = format!("Index accessor must be a {} for {} side of operation", expected, side);
            let comment = format!("The index accessor must be a {} not a {}", expected, index.get_type());
            return error!(meta, tok => { message: message, comment: comment });
        }
        token(meta, "]")?;
        return Ok(Some(index));
    }
    Ok(None)
}

fn allow_index_accessor(index: &Expr, range: bool) -> bool {
    match (&index.kind, &index.value) {
        (Type::Num, _) => true,
        (Type::Array(_), Some(ExprType::Range(_))) => range,
        _ => false,
    }
}
