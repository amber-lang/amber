use super::super::expr::Expr;
use super::UnOp;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::compare::ComparisonOperator;
use crate::translate::fragments::condition::ConditionFragment;
use crate::utils::{metadata::ParserMetadata, TranslateMetadata};
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;

use std::collections::HashMap;
use crate::modules::expression::BoolAnalysis;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "not"]
pub struct Not {
    expr: Box<Expr>,
}

impl Not {
    pub fn analyze_control_flow(&self) -> BoolAnalysis {
        let analysis = self.expr.analyze_control_flow();
        BoolAnalysis {
            known_value: analysis.known_value.map(|value| !value),
            depends_on_target: analysis.depends_on_target,
            has_side_effects: analysis.has_side_effects,
        }
    }

    pub fn extract_facts(&self) -> (HashMap<String, Type>, HashMap<String, Type>) {
        let (true_facts, false_facts) = self.expr.extract_facts();
        (false_facts, true_facts)
    }
}

impl Typed for Not {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl UnOp for Not {
    fn set_expr(&mut self, expr: Expr) {
        *self.expr = expr;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "not")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Not {
    syntax_name!("Not");

    fn new() -> Self {
        Not {
            expr: Box::new(Expr::new()),
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Not {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;
        Self::typecheck_allowed_types(
            meta,
            "logical negation",
            &self.expr,
            &[
                Type::Bool,
                Type::Text,
                Type::Array(Box::new(Type::Generic))
            ]
        )?;
        Ok(())
    }
}

impl TranslateModule for Not {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let analysis = self.analyze_control_flow();
        if let (Some(const_value), false) = (analysis.known_value, analysis.has_side_effects) {
            RawFragment::from((if const_value { "1" } else { "0" }).to_string()).to_frag()
        } else {
            let is_iterable = matches!(self.expr.kind, Type::Text | Type::Array(_));
            let expr = self.expr.translate(meta).with_condition(is_iterable);
            if is_iterable {
                ConditionFragment::new(None, ComparisonOperator::Not, expr).to_frag()
            } else {
                ArithmeticFragment::new(None, ArithOp::Not, expr).to_frag()
            }
        }
    }
}

crate::impl_documentation_noop!(Not);
