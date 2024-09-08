use std::fmt::Display;

use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use crate::utils::ParserMetadata;

#[derive(Debug, Clone, Eq, Default)]
pub enum Type {
    #[default] Null,
    Text,
    Bool,
    Num,
    Union(Vec<Type>),
    Array(Box<Type>),
    Failable(Box<Type>),
    Generic
}

impl Type {
    fn eq_union_normal(one: &[Type], other: &Type) -> bool {
        one.iter().any(|x| (*x).to_string() == other.to_string())
    }

    fn eq_unions(one: &[Type], other: &[Type]) -> bool {
        let bigger;
        let smaller = if one.len() < other.len() { bigger = other; one } else { bigger = one; other };

        smaller.iter().all(|x| {
            Self::eq_union_normal(bigger, x)
        })
    }

    /**
        Hash is calculated in a hex number with 8 digits. Each digit has its own meaning.

        Last two digits are reserved for "singular" types, like text and/or bool.

        # Important
        This hash is not supposed to be readed.  
        It is generated in a way that it can't collide, but it is not supposed to be used for representing a type.

        ```
        0x00 00 00 00
          ^^ ^^ ^^ ^^ -- singular type indicator
                || -- -- number of nested arrays, such as [Num] will be 1, and [[[Num]]] will be 3.
             || --    -- modifier
          || --       -- reserved for future use
        ```

        ## Modifiers
        These modifiers are valid:

        | code | desc          |
        | ---- | ------------- |
        | `00` | no modifier   |
        | `01` | failable type |
     */
    fn type_hash(&self) -> u32 {
        match self {
            Type::Null    => 0x00000001,
            Type::Text    => 0x00000002,
            Type::Bool    => 0x00000003,
            Type::Num     => 0x00000004,
            Type::Generic => 0x00000005,

            Type::Array(t) => t.type_hash() + 0x00000100,

            Type::Failable(t) => {
                if let Type::Failable(_) = **t {
                    panic!("Failable types can't be nested!");
                }
                t.type_hash() + 0x00010000
            },

            Type::Union(_) => unreachable!("Type hash is not available for union types! Use the PartialEq trait instead"),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        if let Type::Union(union) = self {
            if let Type::Union(other) = other {
                return Type::eq_unions(union, other);
            } else {
                return Type::eq_union_normal(union, other);
            }
        }
        
        if let Type::Union(other) = other {
            Type::eq_union_normal(other, self)
        } else {
            self.type_hash() == other.type_hash()
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
            Type::Union(types) => write!(f, "{}", types.iter().map(|x| format!("{x}")).join(" | ")),
            Type::Array(t) => write!(f, "[{}]", t),
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

fn parse_type_tok(meta: &mut ParserMetadata, tok: Option<Token>) -> Result<Type, Failure> {
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

fn parse_one_type(meta: &mut ParserMetadata, tok: Option<Token>) -> Result<Type, Failure> {
    let res = parse_type_tok(meta, tok)?;
    if token(meta, "?").is_ok() {
        return Ok(Type::Failable(Box::new(res)))
    }
    Ok(res)
}

// Tries to parse the type - if it fails, it fails quietly
pub fn try_parse_type(meta: &mut ParserMetadata) -> Result<Type, Failure> {
    let tok = meta.get_current_token();
    let res = parse_one_type(meta, tok);

    if token(meta, "|").is_ok() {
        // is union type
        let mut unioned = vec![ res? ];
        loop {
            match parse_one_type(meta, meta.get_current_token()) {
                Err(err) => return Err(err),
                Ok(t) => unioned.push(t)
            };
            if token(meta, "|").is_err() {
                break;
            }
        }
        return Ok(Type::Union(unioned))
    }

    res
}
