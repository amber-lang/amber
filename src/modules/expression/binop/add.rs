use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::modules::expression::expr::Expr;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};

use super::BinOp;

#[derive(Debug, Clone)]
pub struct Add {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub kind: Type
}

impl Typed for Add {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl BinOp for Add {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "+")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Add {
    syntax_name!("Add");

    fn new() -> Self {
        Add {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::default()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.kind = handle_binop!(meta, "add", self.left, self.right, [
            Num,
            Text,
            Array
        ])?;
        Ok(())
    }
}

impl TranslateModule for Add {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate_eval(meta, false);
        let right = self.right.translate_eval(meta, false);
        let quote = meta.gen_quote();
        match self.kind {
            Type::Array(_) => {
                let id = meta.gen_array_id();
                meta.stmt_queue.push_back(format!("__AMBER_ARRAY_ADD_{id}=({left} {right})"));
                format!("{quote}${{__AMBER_ARRAY_ADD_{id}[@]}}{quote}")
            },
            Type::Text => format!("{}{}", left, right),
            _ => translate_computation(meta, ArithOp::Add, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Add {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
