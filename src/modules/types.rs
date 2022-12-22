use std::fmt::Display;

use heraclitus_compiler::prelude::*;
use crate::utils::ParserMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Text,
    Bool,
    Num,
    Null,
    Array(Box<Type>),
    Generic
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Text => write!(f, "Text"),
            Type::Bool => write!(f, "Bool"),
            Type::Num => write!(f, "Num"),
            Type::Null => write!(f, "Null"),
            Type::Array(t) => write!(f, "[{}]", t),
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

// Tries to parse the type - if it fails, it fails loudly
pub fn parse_type(meta: &mut ParserMetadata) -> Result<Type, Failure> {
    let tok = meta.get_current_token();
    try_parse_type(meta).or_else(|_| error!(meta, tok, "Expected a data type"))
}

// Tries to parse the type - if it fails, it fails quietly
pub fn try_parse_type(meta: &mut ParserMetadata) -> Result<Type, Failure> {
    let tok = meta.get_current_token();
    match tok.clone() {
        Some(matched_token) => {
            match matched_token.word.as_ref() {
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
                "Null" => {
                    meta.increment_index();
                    Ok(Type::Null)
                },
                "[" => {
                    let index = meta.get_index();
                    meta.increment_index();
                    match try_parse_type(meta) {
                        Ok(Type::Array(_)) => error!(meta, tok, "Arrays cannot be nested due to the Bash limitations"),
                        Ok(result_type) => {
                            token(meta, "]")?;
                            Ok(Type::Array(Box::new(result_type)))
                        },
                        Err(_) => {
                            meta.set_index(index);
                            Err(Failure::Quiet(PositionInfo::at_eof(meta)))
                        }
                    }
                },
                // Error messages to help users of other languages understand the syntax
                text @ ("String" | "Char") => {
                    error!(meta, tok, format!("'{text}' is not a valid data type. Did you mean 'Text'?"))
                },
                number @ ("Number" | "Int" | "Float" | "Double") => {
                    error!(meta, tok, format!("'{number}' is not a valid data type. Did you mean 'Num'?"))
                },
                "Boolean" => {
                    error!(meta, tok, "'Boolean' is not a valid data type. Did you mean 'Bool'?")
                },
                array @ ("List" | "Array") => {
                    error!(meta, tok => {
                        message: format!("'{array}'<T> is not a valid data type. Did you mean '[T]'?"),
                        comment: "Where 'T' is the type of the array elements"
                    })
                },
                // The quiet error
                _ => Err(Failure::Quiet(PositionInfo::at_eof(meta)))
            }
        },
        None => {
            Err(Failure::Quiet(PositionInfo::at_eof(meta)))
        }
    }
}