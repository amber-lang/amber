use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::utils::cc_flags::{get_ccflag_name, CCFlags};
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
        *self.expr = left;
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

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Cast {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;

        let pos = self.expr.get_position();
        if !meta.context.cc_flags.contains(&CCFlags::AllowAbsurdCast) {
            let flag_name = get_ccflag_name(CCFlags::AllowAbsurdCast);
            let l_type = self.expr.get_type();
            let r_type = self.kind.clone();
            let message = Message::new_warn_at_position(meta, pos)
                .message(format!("Casting a value of type '{l_type}' to '{r_type}' is not recommended"))
                .comment(format!("To suppress this warning, use '{flag_name}' compiler flag"));
            match (l_type, r_type) {
                (Type::Array(left), Type::Array(right)) => {
                    if *left != *right && !matches!(*left, Type::Bool | Type::Num) && !matches!(*right, Type::Bool | Type::Num) {
                        meta.add_message(message);
                    }
                },
                (Type::Array(_) | Type::Null, Type::Array(_) | Type::Null) => meta.add_message(message),
                (Type::Text, _) => {
                    if self.kind != Type::Text {
                        meta.add_message(message)
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }
}

impl TranslateModule for Cast {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        self.expr.translate(meta)
    }
}

impl DocumentationModule for Cast {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
