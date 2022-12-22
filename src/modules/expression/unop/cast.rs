use heraclitus_compiler::prelude::*;
use crate::{utils::{metadata::ParserMetadata, TranslateMetadata, cc_flags::{get_ccflag_name, CCFlags}}, modules::{types::{Type, Typed, parse_type}, expression::binop::parse_left_expr}, translate::{module::TranslateModule}};
use super::super::expr::Expr;

#[derive(Debug, Clone)]
pub struct Cast {
    expr: Box<Expr>,
    kind: Type
}

impl Typed for Cast {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Cast {
    syntax_name!("Cast");

    fn new() -> Self {
        Cast {
            expr: Box::new(Expr::new()),
            kind: Type::Generic
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        parse_left_expr(meta, &mut *self.expr, "as")?;
        let tok = meta.get_current_token();
        token(meta, "as")?;
        self.kind = parse_type(meta)?;
        if !meta.context.cc_flags.contains(&CCFlags::AllowAbsurdCast) {
            let flag_name = get_ccflag_name(CCFlags::AllowAbsurdCast);
            let l_type = self.expr.get_type();
            let r_type = self.kind.clone();
            let message = Message::new_warn_at_token(meta, tok)
                .message(format!("Casting a value of type '{l_type}' value to a '{r_type}' is not recommended"))
                .comment(format!("To suppress this warning, use ![{flag_name}] before the parent function declaration"));
            match self.kind {
                Type::Array(_) | Type::Null => meta.add_message(message),
                Type::Num => if self.expr.get_type() == Type::Text { meta.add_message(message) },
                _ => {}
            }
        }
        Ok(())
    }
}

impl TranslateModule for Cast {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        self.expr.translate(meta)
    }
}