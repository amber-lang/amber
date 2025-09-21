use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::translate::compute::translate_float_computation;
use crate::modules::types::{Typed, Type};

use super::{BinOp, get_binop_position_info};

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

impl TypeCheckModule for Add {
    fn type_check(&mut self, ctx: &mut TypeContext) -> TypeCheckResult<Type> {
        use crate::modules::typeck::typecheck_binary_allowed_types;
        
        let left_type = self.left.get_type();
        let right_type = self.right.get_type();
        let pos = get_binop_position_info(ctx.metadata(), &self.left, &self.right);
        
        let allowed_types = [
            Type::Num,
            Type::Int,
            Type::Text,
            Type::array_of(Type::Generic),
        ];
        
        let result_type = typecheck_binary_allowed_types(
            ctx.metadata_mut(),
            "addition",
            &left_type,
            &right_type,
            &allowed_types,
            pos,
        )?;
        
        self.kind = result_type.clone();
        Ok(result_type)
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
        // Use the existing type check logic for now to maintain compatibility
        self.kind = Self::typecheck_allowed_types(meta, "addition", &self.left, &self.right, &[
            Type::Num,
            Type::Int,
            Type::Text,
            Type::array_of(Type::Generic),
        ])?;
        Ok(())
    }
}

impl TranslateModule for Add {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        match self.kind {
            Type::Array(_) => {
                let id = meta.gen_value_id();
                let value = fragments!(left, " ", right);
                let var_stmt = VarStmtFragment::new("__array_add", self.kind.clone(), value).with_global_id(id);
                meta.push_ephemeral_variable(var_stmt).to_frag()
            },
            Type::Text => fragments!(left, right),
            Type::Int => ArithmeticFragment::new(left, ArithOp::Add, right).to_frag(),
            Type::Num => translate_float_computation(meta, ArithOp::Add, Some(left), Some(right)),
            _ => unreachable!("Unsupported type {} in addition operation", self.kind)
        }
    }
}

impl DocumentationModule for Add {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
