use crate::modules::expression::expr::Expr;
use crate::modules::function::core::signature::FunctionParam;
use crate::modules::parse_comma_separated;
use crate::modules::types::{parse_type, Type};
use crate::modules::variable::variable_name_extensions;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;

/// Parsed return type from function signature
pub struct ReturnTypeDecl {
    pub kind: Type,
    pub declared_failable: bool,
    /// Tokens are stored for subsequent erroring
    pub type_token: Option<Token>,
    pub question_token: Option<Token>,
}

/// Scans forward over a function body to see whether it can fail, returns to a provided index
pub fn scan_body_for_failure(meta: &mut ParserMetadata, return_to: usize) -> bool {
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
        if scope == 0 {
            break;
        }
        meta.increment_index();
    }
    meta.set_index(return_to);
    is_failable
}

/// Decides whether the `///` comment at the cursor documents a function.
pub fn is_functions_comment_doc(meta: &mut ParserMetadata) -> bool {
    let index = meta.get_index();
    let mut is_comment_doc = true;
    let mut last_line = 0;
    if let Some(tok) = meta.get_current_token() {
        if !tok.word.starts_with("///") {
            return false;
        }
    }
    while let Some(tok) = meta.get_current_token() {
        // If there was a longer break, it means the comment ended
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

/// Parses the parenthesised parameter list, including `ref` markers, type
/// annotations and default values.
pub fn parse_parameters(meta: &mut ParserMetadata) -> Result<Vec<FunctionParam>, Failure> {
    token(meta, "(")?;
    parse_comma_separated(meta, ")", |meta| {
        let is_ref = token(meta, "ref").is_ok();
        let name_token = meta.get_current_token();
        let name = variable(meta, variable_name_extensions())?;

        // Optionally parse the argument type
        let param_type = match token(meta, ":") {
            Ok(_) => parse_type(meta)?,
            Err(_) => Type::Generic,
        };

        // Optionally parse default value
        let default = match token(meta, "=") {
            Ok(_) => {
                let mut expr = Expr::new();
                syntax(meta, &mut expr)?;
                Some(expr)
            }
            Err(_) => None,
        };

        Ok(FunctionParam::new(name, param_type)
            .with_default(default)
            .with_ref(is_ref)
            .with_token(name_token))
    })
}

/// Parses the optional `: Type` and `?` failability marker.
pub fn parse_return_type(meta: &mut ParserMetadata) -> Result<ReturnTypeDecl, Failure> {
    let mut decl = ReturnTypeDecl {
        kind: Type::Generic,
        declared_failable: false,
        type_token: None,
        question_token: None,
    };
    if token(meta, ":").is_ok() {
        decl.type_token = meta.get_current_token();
        decl.kind = parse_type(meta)?;
        decl.question_token = meta.get_current_token();
        if token(meta, "?").is_ok() {
            decl.declared_failable = true;
        }
    }
    Ok(decl)
}
