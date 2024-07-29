use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::handle_binop;
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Add {
    pub left: Box<AlreadyParsedExpr>,
    pub right: Box<AlreadyParsedExpr>,
    pub kind: Type
}

impl Typed for Add {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Add {
    syntax_name!("Add");

    fn new() -> Self {
        Add {
            left: Box::new(AlreadyParsedExpr::new()),
            right: Box::new(AlreadyParsedExpr::new()),
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let message = {
            let l_type = self.left.get_type();
            let r_type = self.right.get_type();
            format!("Cannot add value of type '{l_type}' with value of type '{r_type}'")
        };
        let comment = "You can only add values of type 'Num', 'Text' or 'Array' together.";
        self.kind = handle_binop!(meta, self.left, self.right, message, comment, [
            Type::Num,
            Type::Text,
            Type::Array(_)
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
