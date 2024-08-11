use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::{docs::module::DocumentationModule, translate::module::TranslateModule};
use crate::utils::cc_flags::{get_ccflag_name, CCFlags};
use crate::utils::metadata::ParserMetadata;
use crate::utils::TranslateMetadata;
use crate::modules::types::{Type, Typed};

use super::TypeOp;

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

impl TypeOp for Cast {
    fn set_left(&mut self, left: Expr) {
        self.expr = Box::new(left);
    }

    fn set_right(&mut self, right: Type) {
        self.kind = right;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "as")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Cast {
    syntax_name!("Cast");

    fn new() -> Self {
        Cast {
            expr: Box::new(Expr::new()),
            kind: Type::default()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let begin = meta.get_token_at(self.expr.pos.0);
        let end = meta.get_current_token();
        let pos = PositionInfo::from_between_tokens(meta, begin, end);
        if !meta.context.cc_flags.contains(&CCFlags::AllowAbsurdCast) {
            let flag_name = get_ccflag_name(CCFlags::AllowAbsurdCast);
            let l_type = self.expr.get_type();
            let r_type = self.kind.clone();
            let message = Message::new_warn_at_position(meta, pos)
                .message(format!("Casting a value of type '{l_type}' value to a '{r_type}' is not recommended"))
                .comment(format!("To suppress this warning, use '{flag_name}' compiler flag"));
            let (l_type, r_type) = match (l_type, r_type) {
                (Type::Failable(l), Type::Failable(r)) => (*l, *r),
                (Type::Failable(_), _) | (_, Type::Failable(_)) => {
                    meta.add_message(message);
                    return Ok(());
                },
                types => types
            };
            match (l_type, r_type) {
                (Type::Array(left), Type::Array(right)) => {
                    if *left != *right && !matches!(*left, Type::Bool | Type::Num) && !matches!(*right, Type::Bool | Type::Num) {
                        meta.add_message(message);
                    }
                },
                (Type::Array(_) | Type::Null, Type::Array(_) | Type::Null) => meta.add_message(message),
                (Type::Text, Type::Num) => { meta.add_message(message) },
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

impl DocumentationModule for Cast {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
