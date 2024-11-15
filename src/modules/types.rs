use std::fmt::Display;

use heraclitus_compiler::prelude::*;
use crate::utils::ParserMetadata;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Type {
    #[default] Null,
    Text,
    Bool,
    Num,
    Array(Box<Type>),
    Failable(Box<Type>),
    Generic
}

impl Type {
    pub fn is_subset_of(&self, other: &Type) -> bool {
        match (self, other) {
            (_, Type::Generic) => true,
            (Type::Array(current), Type::Array(other)) => {
                **current != Type::Generic && **other == Type::Generic
            }
            (current, Type::Failable(other)) if !matches!(current, Type::Failable(_)) => {
                current.is_allowed_in(other)
            },
            _ => false
        }
    }

    pub fn is_allowed_in(&self, other: &Type) -> bool {
        self == other || self.is_subset_of(other)
    }

    pub fn is_array(&self) -> bool {
        match self {
            Type::Array(_) => true,
            Type::Failable(inner) => inner.is_array(),
            _ => false,
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
            Type::Failable(t) => write!(f, "{}?", t),
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

    if token(meta, "?").is_ok() {
        return res.map(|t| Type::Failable(Box::new(t)))
    }

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

    #[test]
    fn non_failable_is_a_subset_of_failable() {
        let a = Type::Text;
        let b = Type::Failable(Box::new(Type::Text));

        assert!(a.is_subset_of(&b));
    }

    #[test]
    fn failable_is_not_a_subset_of_non_failable() {
        let a = Type::Text;
        let b = Type::Failable(Box::new(Type::Text));

        assert!(!b.is_subset_of(&a));
    }

    #[test]
    fn failable_is_not_a_subset_of_itself() {
        let a = Type::Failable(Box::new(Type::Text));

        assert!(!a.is_subset_of(&a));
    }
}
