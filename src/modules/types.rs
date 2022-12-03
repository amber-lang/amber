use std::fmt::Display;

use heraclitus_compiler::prelude::*;
use crate::utils::ParserMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Text,
    Bool,
    Num,
    Null,
    Generic
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Text => write!(f, "Text"),
            Type::Bool => write!(f, "Bool"),
            Type::Num => write!(f, "Num"),
            Type::Null => write!(f, "Null"),
            Type::Generic => write!(f, "Generic")
        }
    }
}

pub trait Typed {
    fn get_type(&self) -> Type;
    fn has_echo(&self) -> bool {
        false
    }
}

pub fn parse_type(meta: &mut ParserMetadata) -> Result<Type, Failure> {
    let tok = meta.get_current_token();
    match tok.clone() {
        Some(token) => {
            match token.word.as_ref() {
                "Text" => {
                    meta.increment_index();
                    Ok(Type::Text)
                },
                "Bool" => {
                    meta.increment_index();
                    Ok(Type::Bool)
                },
                "Num" => {
                    meta.increment_index();
                    Ok(Type::Num)
                },
                // Error messages to help users of other languages understand the syntax
                text @ ("String" | "Char") => {
                    meta.increment_index();
                    error!(meta, tok, format!("'{text}' is not a valid data type. Did you mean 'Text'?"))
                },
                number @ ("Number" | "Int" | "Float" | "Double") => {
                    meta.increment_index();
                    error!(meta, tok, format!("'{number}' is not a valid data type. Did you mean 'Num'?"))
                },
                "Boolean" => {
                    meta.increment_index();
                    error!(meta, tok, "'Boolean' is not a valid data type. Did you mean 'Bool'?")
                },
                // The actual error message
                _ => error!(meta, tok, "Expected a data type")
            }
        },
        None => Err(Failure::Quiet(PositionInfo::at_eof(meta)))
    }
}