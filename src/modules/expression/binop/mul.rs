use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::translate::compute::{translate_float_computation, ArithOp};
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Typed, Type};

use super::BinOp;

#[derive(Debug, Clone)]
pub struct Mul {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type
}

impl Typed for Mul {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl BinOp for Mul {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "*")?;
        Ok(())
    }
}

impl TypeCheckModule for Mul {
    fn type_check(&mut self, ctx: &mut TypeContext) -> TypeCheckResult<Type> {
        use crate::modules::typeck::typecheck_binary_allowed_types;
        use super::get_binop_position_info;
        
        let left_type = self.left.get_type();
        let right_type = self.right.get_type();
        let pos = get_binop_position_info(ctx.metadata(), &self.left, &self.right);
        
        let allowed_types = [Type::Num, Type::Int];
        
        let result_type = typecheck_binary_allowed_types(
            ctx.metadata_mut(),
            "multiplication",
            &left_type,
            &right_type,
            &allowed_types,
            pos,
        )?;
        
        self.kind = result_type.clone();
        Ok(result_type)
    }
}

impl SyntaxModule<ParserMetadata> for Mul {
    syntax_name!("Mul");

    fn new() -> Self {
        Mul {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::Generic
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.kind = Self::typecheck_allowed_types(meta, "multiplication", &self.left, &self.right, &[
            Type::Num,
            Type::Int,
        ])?;
        Ok(())
    }
}

impl TranslateModule for Mul {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        match self.kind {
            Type::Int => FragmentKind::Arithmetic(ArithmeticFragment::new(left, ArithOp::Mul, right)),
            Type::Num => translate_float_computation(meta, ArithOp::Mul, Some(left), Some(right)),
            _ => unreachable!("Unsupported type {} in multiplication operation", self.kind)
        }
    }
}

impl DocumentationModule for Mul {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
