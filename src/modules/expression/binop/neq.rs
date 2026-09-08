use super::BinOp;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::compare::{translate_array_equality, ComparisonOperator};
use crate::translate::compute::{translate_float_computation, ArithOp};
use crate::translate::fragments::condition::ConditionFragment;
use crate::utils::TranslateMetadata;
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "neq"]
#[kind = "binary_op"]
pub struct Neq {
    left: Box<Expr>,
    right: Box<Expr>,
}

impl Neq {
    /// Check if this inequality comparison can be folded to a constant.
    pub fn analyze_control_flow(&self, meta: &TranslateMetadata) -> Option<bool> {
        let target_family = meta.target.shell.family_name();

        // Optimize away `shellname() != "<shell>"` if possible at compile time
        match (&self.left.value, &self.right.value) {
            (Some(ExprType::Shellname(_)), Some(ExprType::Text(text))) => {
                text.as_simple_string().map(|str| str != target_family)
            }
            (Some(ExprType::Text(text)), Some(ExprType::Shellname(_))) => {
                text.as_simple_string().map(|str| str != target_family)
            }
            _ => None,
        }
    }
}

impl Typed for Neq {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Neq {
    fn set_left(&mut self, left: Expr) {
        *self.left = left;
    }

    fn set_right(&mut self, right: Expr) {
        *self.right = right;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "!=")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Neq {
    syntax_name!("Neq");

    fn new() -> Self {
        Neq {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Neq {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.left.typecheck(meta)?;
        self.right.typecheck(meta)?;
        Self::typecheck_equality(meta, &mut self.left, &mut self.right)?;
        Ok(())
    }
}

impl TranslateModule for Neq {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Check for constant folding
        if let Some(constant_value) = self.analyze_control_flow(meta) {
            return RawFragment::from((if constant_value { "1" } else { "0" }).to_string()).to_frag();
        }

        let left = self.left.translate(meta).with_quotes(false);
        let right = self.right.translate(meta).with_quotes(false);
        match (self.left.get_type(), self.right.get_type()) {
            (Type::Num, _) | (_, Type::Num) => translate_float_computation(meta, ArithOp::Neq, Some(left), Some(right)),
            (Type::Int, _) | (Type::Bool, _) => ArithmeticFragment::new(left, ArithOp::Neq, right).to_frag(),
            (Type::Array(_), _) => {
                if let (FragmentKind::VarExpr(left), FragmentKind::VarExpr(right)) = (left, right) {
                    translate_array_equality(left, right, true)
                } else {
                    unreachable!(
                        "Arrays are always represented as variable expressions when used as values"
                    )
                }
            }
            _ => ConditionFragment::new(left, ComparisonOperator::Neq, right)
                .to_frag()
        }
    }
}

crate::impl_documentation_noop!(Neq);
