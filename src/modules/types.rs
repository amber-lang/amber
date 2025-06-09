use std::fmt::Display;

use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use crate::utils::ParserMetadata;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Type {
    #[default] Null,
    Text,
    Bool,
    Num,
    Array(Box<Type>),
    Generic
}

impl Type {
    #[inline]
    pub fn array_of(kind: Type) -> Self {
        Self::Array(Box::new(kind))
    }

    pub fn is_subset_of(&self, other: &Type) -> bool {
        match (self, other) {
            (_, Type::Generic) => true,
            (Type::Array(current), Type::Array(other)) => {
                **current != Type::Generic && **other == Type::Generic
            },
            _ => false
        }
    }

    pub fn is_allowed_in(&self, other: &Type) -> bool {
        self == other || self.is_subset_of(other)
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Type::Array(_))
    }

    pub fn pretty_join(types: &[Self], op: &str) -> String {
        let mut all_types = types.iter().map(|kind| kind.to_string()).collect_vec();
        let last_item = all_types.pop();
        let comma_separated = all_types.iter().join(", ");
        if let Some(last) = last_item {
            if types.len() == 1 {
                last
            } else {
                [comma_separated, last].join(&format!(" {op} "))
            }
        } else {
            comma_separated
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Text => write!(f, "Text"),
            Type::Bool => write!(f, "Bool"),
            Type::Num => write!(f, "Num"),
            Type::Null => write!(f, "Null"),
            Type::Array(t) => if **t == Type::Generic {
                    write!(f, "[]")
                } else {
                    write!(f, "[{}]", t)
                },
            Type::Generic => write!(f, "Generic")
        }
    }
}

pub trait Typed {
    fn get_type(&self) -> Type;
}

// Tries to parse the type - if it fails, it fails loudly
pub fn parse_type(meta: &mut ParserMetadata) -> Result<Type, Failure> {
    let tok = meta.get_current_token();
    try_parse_type(meta)
        .map_err(|_| Failure::Loud(Message::new_err_at_token(meta, tok).message("Expected a data type")))
}

// Tries to parse the type - if it fails, it fails quietly
pub fn try_parse_type(meta: &mut ParserMetadata) -> Result<Type, Failure> {
    let tok = meta.get_current_token();
    let res = match tok.clone() {
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
                    if token(meta, "]").is_ok() {
                        Ok(Type::Array(Box::new(Type::Generic)))
                    } else {
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
    };

    res
}

#[cfg(test)]
mod tests {
    use super::Type;

    #[test]
    fn concrete_array_is_a_subset_of_generic_array() {
        let a = Type::Array(Box::new(Type::Text));
        let b = Type::Array(Box::new(Type::Generic));

        assert!(a.is_subset_of(&b));
    }

    #[test]
    fn generic_array_is_not_a_subset_of_concrete_array() {
        let a = Type::Array(Box::new(Type::Text));
        let b = Type::Array(Box::new(Type::Generic));

        assert!(!b.is_subset_of(&a));
    }

    #[test]
    fn concrete_array_is_not_a_subset_of_itself() {
        let a = Type::Array(Box::new(Type::Text));

        assert!(!a.is_subset_of(&a));
    }

    #[test]
    fn generic_array_is_not_a_subset_of_itself() {
        let a = Type::Array(Box::new(Type::Generic));

        assert!(!a.is_subset_of(&a));
    }
}
