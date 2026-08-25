use super::BinOp;
use crate::modules::builtin::shellname::Shellname;
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
    /// Returns Some(true) if always true, Some(false) if always false, None otherwise.
    pub fn analyze_control_flow(&self, meta: &TranslateMetadata) -> Option<bool> {
        // Check for shellname() != "literal" or "literal" != shellname() pattern
        let get_literal = |expr: &Expr| -> Option<String> {
            match &expr.value {
                Some(ExprType::Text(text)) => text.as_simple_string().map(|s| s.to_string()),
                _ => None,
            }
        };
        
        let target_family = meta.target.shell.family_name();
        
        match (&self.left.value, &self.right.value) {
            (Some(ExprType::Shellname(_)), _) => {
                get_literal(&self.right).map(|lit| target_family != lit.as_str())
            }
            (_, Some(ExprType::Shellname(_))) => {
                get_literal(&self.left).map(|lit| target_family != lit.as_str())
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
        // Check for shellname() != "literal" or "literal" != shellname() pattern
        let shellname_result = match (&self.left.value, &self.right.value) {
            (Some(ExprType::Shellname(_)), Some(ExprType::Text(_))) => {
                // shellname() != "literal"
                Shellname {}.try_fold_comparison(meta, &self.right, false)
            }
            (Some(ExprType::Text(_)), Some(ExprType::Shellname(_))) => {
                // "literal" != shellname()
                Shellname {}.try_fold_comparison(meta, &self.left, false)
            }
            _ => None,
        };
        
        if let Some(frag) = shellname_result {
            // Don't set shellname_used since we're folding to a constant
            return frag;
        }

        // Check for constant folding via analyze_control_flow
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
