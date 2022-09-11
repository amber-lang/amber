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
}

pub fn parse_type(meta: &mut ParserMetadata) -> Result<Type,ErrorDetails> {
    let tok = meta.get_current_token();
    match tok {
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
                _ => Err(ErrorDetails::from_token_option(Some(token)))
            }
        },
        None => Err(ErrorDetails::with_eof())
    }
}