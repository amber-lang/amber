use crate::modules::prelude::*;
use crate::{fragments, raw_fragment};
use crate::modules::expression::binop::BinOp;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::types::{Type, Typed};
use crate::translate::compute::{translate_float_computation, ArithOp};
use heraclitus_compiler::prelude::*;
use std::cmp::max;

#[derive(Debug, Clone)]
pub struct Range {
    pub from: Box<Expr>,
    pub to: Box<Expr>,
    pub neq: bool
}

impl Typed for Range {
    fn get_type(&self) -> Type {
        Type::Array(Box::new(Type::Int))
    }
}

impl BinOp for Range {
    fn set_left(&mut self, left: Expr) {
        self.from = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.to = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "..")?;
        self.neq = token(meta, "=").is_err();
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Range {
    syntax_name!("Range");

    fn new() -> Self {
        Range {
            from: Box::new(Expr::new()),
            to: Box::new(Expr::new()),
            neq: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Self::typecheck_allowed_types(meta, "range operator", &self.from, &self.to, &[Type::Int])?;
        Ok(())
    }
}

impl TranslateModule for Range {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let from = self.from.translate(meta);
        let to = if let Some(ExprType::Integer(int)) = &self.to.value {
            match (int.value.parse::<i64>(), self.neq) {
                (Ok(value), true) => raw_fragment!("{}", value - 1),
                _ => self.to.translate(meta)
            }
        } else {
            let to = self.to.translate(meta);
            if self.neq {
                ArithmeticFragment::new(to, ArithOp::Sub, fragments!("1")).to_frag()
            } else {
                to
            }
        };
        let expr = fragments!("seq ", from, " ", to);
        SubprocessFragment::new(expr).with_quotes(false).to_frag()
    }
}

impl Range {
    pub fn get_array_index(&self, meta: &mut TranslateMetadata) -> (FragmentKind, FragmentKind) {
        if let Some(from) = self.from.get_integer_value() {
            if let Some(mut to) = self.to.get_integer_value() {
                // Make the upper bound exclusive.
                if !self.neq {
                    to += 1;
                }
                // Cap the lower bound at zero.
                let offset = max(from, 0);
                // Cap the slice length at zero.
                let length = max(to - offset, 0);
                return (
                    RawFragment::from(offset.to_string()).to_frag(),
                    RawFragment::from(length.to_string()).to_frag()
                );
            }
        }
        // Make the upper bound exclusive.
        let upper = {
            let upper_id = meta.gen_value_id();
            let mut upper_val = self.to.translate(meta);
            if !self.neq {
                upper_val = translate_float_computation(meta, ArithOp::Add, Some(upper_val), Some(fragments!("1")));
            }
            let upper_var_stmt = VarStmtFragment::new("__slice_upper", Type::Int, upper_val).with_global_id(upper_id);
            meta.push_ephemeral_variable(upper_var_stmt).to_frag()
        };

        // Cap the lower bound at zero.
        let offset = {
            let offset_id = meta.gen_value_id();
            let offset_val = self.from.translate(meta);
            let offset_var_stmt = VarStmtFragment::new("__slice_offset", Type::Int, offset_val).with_global_id(offset_id);
            let offset_var_expr = meta.push_ephemeral_variable(offset_var_stmt).to_frag();
            let offset_cap = fragments!("$((", offset_var_expr.clone().with_quotes(false), " > 0 ? ", offset_var_expr.with_quotes(false), " : 0))");
            let offset_var_stmt = VarStmtFragment::new("__slice_offset", Type::Int, offset_cap).with_global_id(offset_id);
            meta.push_ephemeral_variable(offset_var_stmt).to_frag()
        };

        // Cap the slice length at zero.
        let length = {
            let length_id = meta.gen_value_id();
            let length_val = translate_float_computation(meta, ArithOp::Sub, Some(upper), Some(offset.clone()));
            let length_var_stmt = VarStmtFragment::new("__slice_length", Type::Int, length_val).with_global_id(length_id);
            let length_var_expr = meta.push_ephemeral_variable(length_var_stmt).to_frag();
            let length_cap = fragments!("$((", length_var_expr.clone().with_quotes(false), " > 0 ? ", length_var_expr.with_quotes(false), " : 0))");
            let length_var_stmt = VarStmtFragment::new("__slice_length", Type::Int, length_cap).with_global_id(length_id);
            meta.push_ephemeral_variable(length_var_stmt).to_frag()
        };

        (offset, length)
    }
}

impl DocumentationModule for Range {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
