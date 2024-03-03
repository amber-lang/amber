use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use super::{super::expr::Expr, parse_left_expr, expression_arms_of_type};
use crate::modules::types::{Typed, Type};

#[derive(Debug, Clone)]
pub struct Add {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
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
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut self.left, "+")?;
        let tok = meta.get_current_token();
        token(meta, "+")?;
        syntax(meta, &mut *self.right)?;
        let l_type = self.left.get_type();
        let r_type = self.right.get_type();
        let message = format!("Cannot add value of type '{l_type}' with value of type '{r_type}'");
        let predicate = |kind| matches!(kind, Type::Num | Type::Text | Type::Array(_));
        self.kind = expression_arms_of_type(meta, &l_type, &r_type, predicate, tok, &message)?;
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
    fn document(&self) -> String {
        "".to_string()
    }
}
