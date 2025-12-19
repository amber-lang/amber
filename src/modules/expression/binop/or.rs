use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::translate::compute::ArithOp;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Typed, Type};

use super::BinOp;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Or {
    left: Box<Expr>,
    right: Box<Expr>
}

impl Or {
    pub fn analyze_control_flow(&self) -> Option<bool> {
        let left = self.left.analyze_control_flow();
        let right = self.right.analyze_control_flow();
        match (left, right) {
             (Some(true), _) => Some(true),
             (_, Some(true)) => Some(true),
             (Some(false), Some(false)) => Some(false),
             _ => None
        }
    }

    pub fn extract_facts(&self) -> (HashMap<String, Type>, HashMap<String, Type>) {
        let (left_true, left_false) = self.left.extract_facts();
        let (right_true, right_false) = self.right.extract_facts();

        // Intersect true facts
        let mut true_facts = HashMap::new();
        for (name, left_kind) in left_true {
            if let Some(right_kind) = right_true.get(&name) {
                if left_kind == *right_kind {
                    true_facts.insert(name, left_kind);
                }
            }
        }

        // Merge false facts
        let mut false_facts = left_false;
        false_facts.extend(right_false);

        (true_facts, false_facts)
    }
}


impl Typed for Or {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl BinOp for Or {
    fn set_left(&mut self, left: Expr) {
        *self.left = left;
    }

    fn set_right(&mut self, right: Expr) {
        *self.right = right;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "or")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Or {
    syntax_name!("Or");

    fn new() -> Self {
        Or {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new())
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Or {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.left.typecheck(meta)?;
        self.right.typecheck(meta)?;
        Self::typecheck_allowed_types(meta, "logical OR", &mut self.left, &mut self.right, &[
            Type::Bool,
        ])?;
        Ok(())
    }
}

impl TranslateModule for Or {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left = self.left.translate(meta);
        let right = self.right.translate(meta);
        ArithmeticFragment::new(left, ArithOp::Or, right).to_frag()
    }
}

impl DocumentationModule for Or {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
