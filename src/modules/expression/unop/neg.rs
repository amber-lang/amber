use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::unop::UnOp;
use crate::modules::prelude::{ArithmeticFragment, FragmentKind, FragmentRenderable, RawFragment};
use crate::modules::types::{Type, Typed};
use crate::modules::typecheck::TypeCheckModule;
use crate::translate::compute::{translate_float_computation, ArithOp};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::ParserMetadata;
use crate::utils::TranslateMetadata;
use heraclitus_compiler::prelude::*;
use std::ops::Neg as _;

#[derive(Debug, Clone)]
pub struct Neg {
    expr: Box<Expr>
}

impl Typed for Neg {
    fn get_type(&self) -> Type {
        self.expr.get_type()
    }
}

impl UnOp for Neg {
    fn set_expr(&mut self, expr: Expr) {
        *self.expr = expr;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "-")?;
        // Let the number be parsed with a minus sign instead of a negation operator.
        // This allows numbers to parse as `-42` instead of `$(( - 42 ))`
        if meta.get_current_token().map(|tok| tok.word.parse::<usize>().is_ok()).unwrap_or(false) {
            meta.set_index(meta.get_index().saturating_sub(1));
            return Err(Failure::Quiet(PositionInfo::from_metadata(meta)))
        }
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Neg {
    syntax_name!("Neg");

    fn new() -> Self {
        Neg {
            expr: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Neg {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;
        Self::typecheck_allowed_types(meta, "arithmetic negation", &self.expr, &[Type::Num, Type::Int])?;
        Ok(())
    }
}

impl TranslateModule for Neg {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let expr = self.expr.translate(meta);
        match self.expr.get_type() {
            Type::Int => ArithmeticFragment::new(None, ArithOp::Neg, expr).to_frag(),
            _ => translate_float_computation(meta, ArithOp::Neg, None, Some(expr))
        }
    }
}

impl Neg {
    pub fn get_integer_value(&self) -> Option<isize> {
        self.expr.get_integer_value().map(isize::neg)
    }

    pub fn get_array_index(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if let Some(expr) = self.get_integer_value() {
            RawFragment::from(expr.to_string()).to_frag()
        } else {
            self.translate(meta)
        }
    }
}

impl DocumentationModule for Neg {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
