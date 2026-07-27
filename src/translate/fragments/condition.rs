use itertools::Itertools;

use super::fragment::{FragmentKind, FragmentRenderable};
use crate::{translate::compare::ComparisonOperator, utils::TranslateMetadata};

// Creates a condition fragment that is correctly escaped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionFragment {
    pub left: Box<Option<FragmentKind>>,
    pub right: Box<Option<FragmentKind>>,
    pub op: ComparisonOperator,
    pub quoted: bool,
    // Defines whether the fragment generates a raw test condition or a boolean value as integer
    pub with_subprocess: bool
}

impl ConditionFragment {
    pub fn new<T, U>(left: T, op: ComparisonOperator, right: U) -> Self
    where
        T: Into<Option<FragmentKind>>,
        U: Into<Option<FragmentKind>>,
    {
        ConditionFragment {
            left: Box::new(left.into()),
            right: Box::new(right.into()),
            quoted: true,
            with_subprocess: true,
            op,
        }
    }

    pub fn with_subprocess(mut self, subprocess: bool) -> Self {
        self.with_subprocess = subprocess;
        self
    }

    fn operator_to_string(&self) -> &'static str {
        match self.op {
            ComparisonOperator::Gt => ">",
            ComparisonOperator::Ge => ">=",
            ComparisonOperator::Lt => "<",
            ComparisonOperator::Le => "<=",
            ComparisonOperator::Eq => "==",
            ComparisonOperator::Neq => "!=",
            ComparisonOperator::And => "&&",
            ComparisonOperator::Or => "||",
            ComparisonOperator::Not => "!"
        }
    }
}

impl FragmentRenderable for ConditionFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let op = self.operator_to_string().to_string();
        let is_parent_condition = matches!(self.op, ComparisonOperator::And | ComparisonOperator::Or);
        let is_negated = matches!(self.op, ComparisonOperator::Not);
        let left = self.left.clone().unwrap_or_default()
            .with_quotes(self.quoted)
            .with_condition(is_parent_condition || is_negated)
            .to_string(meta);
        let right = self.right.unwrap_or_default()
            .with_quotes(self.quoted)
            .with_condition(is_parent_condition || is_negated)
            .to_string(meta);
        let mut expr = [
                    left,
                    op, 
                    right
                ].iter().filter(|x| !x.is_empty()).join(" ");
                
        expr = if is_parent_condition {
            format!("{{ {expr}; }}")
        } else if is_negated {
            expr
         } else  {
            format!("[[ {expr} ]]")
        };

        if self.with_subprocess {
            format!("$({expr} && echo 1 || echo 0)")
        } else {
            expr
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Condition(self)
    }
}
