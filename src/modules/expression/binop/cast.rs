use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::AlreadyParsedExpr;
use crate::{docs::module::DocumentationModule, translate::module::TranslateModule};
use crate::utils::cc_flags::{get_ccflag_name, CCFlags};
use crate::utils::metadata::ParserMetadata;
use crate::utils::TranslateMetadata;
use crate::modules::types::{AlreadyParsedType, Type, Typed};

#[derive(Debug, Clone)]
pub struct Cast {
    pub expr: Box<AlreadyParsedExpr>,
    pub kind: AlreadyParsedType
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
            expr: Box::new(AlreadyParsedExpr::new()),
            kind: AlreadyParsedType::Generic
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
