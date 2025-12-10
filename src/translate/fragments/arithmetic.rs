use itertools::Itertools;

use crate::{translate::compute::ArithOp, utils::TranslateMetadata};
use super::fragment::{FragmentKind, FragmentRenderable};

// Creates a subprocess fragment that is correctly escaped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArithmeticFragment {
    pub left: Box<Option<FragmentKind>>,
    pub right: Box<Option<FragmentKind>>,
    pub op: ArithOp,
    pub quoted: bool,
}

impl ArithmeticFragment {
    pub fn new<T, U>(left: T, op: ArithOp, right: U) -> Self
    where T: Into<Option<FragmentKind>>, U: Into<Option<FragmentKind>> {
        ArithmeticFragment {
            left: Box::new(left.into()),
            right: Box::new(right.into()),
            quoted: true,
            op,
        }
    }

    pub fn with_quotes(mut self, quoted: bool) -> Self {
        self.quoted = quoted;
        self
    }

    fn operator_to_string(&self) -> &'static str {
        match self.op {
            ArithOp::Add => "+",
            ArithOp::Sub => "-",
            ArithOp::Mul => "*",
            ArithOp::Div => "/",
            ArithOp::Modulo => "%",
            ArithOp::Neg => "-",
            ArithOp::Gt => ">",
            ArithOp::Ge => ">=",
            ArithOp::Lt => "<",
            ArithOp::Le => "<=",
            ArithOp::Eq => "==",
            ArithOp::Neq => "!=",
            ArithOp::Not => "!",
            ArithOp::And => "&&",
            ArithOp::Or => "||"
        }
    }
}

impl FragmentRenderable for ArithmeticFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let dollar = meta.gen_dollar();
        let op = self.operator_to_string().to_string();
        let left = self.left.unwrap_or_default().with_quotes(false);
        let right = self.right.unwrap_or_default().with_quotes(false);
        
        // ShellCheck SC2004: We enable is_math_var to avoid unnecessary ${} wrapping in arithmetic expressions
        let left = if let FragmentKind::VarExpr(var) = left {
            var.with_math_var(true).to_string(meta)
        } else {
            left.to_string(meta)
        };

        let right = if let FragmentKind::VarExpr(var) = right {
            var.with_math_var(true).to_string(meta)
        } else {
            right.to_string(meta)
        };

        let quote = if self.quoted { meta.gen_quote() } else { "" };
        let expr = [left, op, right].iter().filter(|x| !x.is_empty()).join(" ");
        format!("{quote}{dollar}(( {expr} )){quote}")
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Arithmetic(self)
    }
}
