use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use heraclitus_compiler::prelude::*;

use super::TypeOp;

use std::collections::HashMap;

use crate::modules::expression::expr::ExprType;

#[derive(Debug, Clone)]
pub struct Is {
    expr: Box<Expr>,
    kind: Type,
}

impl Is {
    pub fn analyze_control_flow(&self) -> Option<bool> {
        let expr_type = self.expr.get_type();

        // If types are identical, it's always true
        if expr_type == self.kind {
            return Some(true);
        }

        // If types cannot possibly intersect, it's always false
        if !expr_type.can_intersect(&self.kind) {
            return Some(false);
        }

        None
    }

    pub fn extract_facts(&self) -> (HashMap<String, Type>, HashMap<String, Type>) {
        if let Some(ExprType::VariableGet(var)) = &self.expr.value {
            let mut true_facts = HashMap::new();
            true_facts.insert(var.name.clone(), self.kind.clone());

            let mut false_facts = HashMap::new();
            // Calculate false facts (narrowing in else branch)
            if let Some(type_false) = self.expr.get_type().exclude(&self.kind) {
                false_facts.insert(var.name.clone(), type_false);
            }

            return (true_facts, false_facts);
        }
        (HashMap::new(), HashMap::new())
    }
}

impl Typed for Is {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

impl TypeOp for Is {
    fn set_left(&mut self, left: Expr) {
        *self.expr = left;
    }

    fn set_right(&mut self, right: Type) {
        self.kind = right;
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "is")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Is {
    syntax_name!("Add");

    fn new() -> Self {
        Is {
            expr: Box::new(Expr::new()),
            kind: Type::default(),
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Is {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)
    }
}

impl TranslateModule for Is {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        if self.expr.get_type() == self.kind {
            fragments!("1")
        } else {
            fragments!("0")
        }
    }
}

impl DocumentationModule for Is {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
