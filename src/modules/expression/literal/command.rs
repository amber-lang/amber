use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{Type, Typed}};
use crate::modules::expression::expr::Expr;

use super::parse_interpolated_region;

#[derive(Debug)]
pub struct Command {
    strings: Vec<String>,
    interps: Vec<Expr>
}

impl Typed for Command {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Command {
    syntax_name!("Command");

    fn new() -> Self {
        Command {
            strings: vec![],
            interps: vec![]
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        (self.strings, self.interps) = parse_interpolated_region(meta, '$')?;
        Ok(())
    }
}