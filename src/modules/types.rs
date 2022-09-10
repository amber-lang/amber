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

pub trait Typed {
    fn get_type(&self) -> Type;
}

pub fn parseType(meta: &mut ParserMetadata) -> Result<Type,ErrorDetails> {
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
                _ => Err(ErrorDetails::from_token_option(tok))
            }
        },
        None => Err(ErrorDetails::with_eof())
    }
}