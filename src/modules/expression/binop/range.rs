use crate::docs::module::DocumentationModule;
use crate::modules::expression::binop::BinOp;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::ParserMetadata;
use crate::utils::TranslateMetadata;
use crate::{error_type_match, handle_binop};
use heraclitus_compiler::prelude::*;
use std::cmp::max;

#[derive(Debug, Clone)]
pub struct Range {
    from: Box<Expr>,
    to: Box<Expr>,
    neq: bool
}

impl Typed for Range {
    fn get_type(&self) -> Type {
        Type::Array(Box::new(Type::Num))
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
        handle_binop!(meta, "apply range operator for", self.from, self.to, [Num])?;
        Ok(())
    }
}

impl TranslateModule for Range {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let from = self.from.translate(meta);
        let to = if let Some(to) = self.to.get_integer_value() {
            if self.neq {
                (to - 1).to_string()
            } else {
                to.to_string()
            }
        } else {
            let to = self.to.translate(meta);
            if self.neq {
                translate_computation(meta, ArithOp::Sub, Some(to), Some("1".to_string()))
            } else {
                to
            }
        };
        let stmt = format!("seq {} {}", from, to);
        meta.gen_subprocess(&stmt)
    }
}

impl Range {
    pub fn get_array_index(&self, meta: &mut TranslateMetadata) -> (String, String) {
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
                return (offset.to_string(), length.to_string());
            }
        }
        let local = if meta.fun_meta.is_some() { "local " } else { "" };
        // Make the upper bound exclusive.
        let upper_name = format!("__SLICE_UPPER_{}", meta.gen_value_id());
        let mut upper_val = self.to.translate(meta);
        if !self.neq {
            upper_val = translate_computation(meta, ArithOp::Add, Some(upper_val), Some("1".to_string()));
        }
        meta.stmt_queue.push_back(format!("{local}{upper_name}={upper_val}"));
        // Cap the lower bound at zero.
        let offset_name = format!("__SLICE_OFFSET_{}", meta.gen_value_id());
        let offset_val = self.from.translate(meta);
        meta.stmt_queue.push_back(format!("{local}{offset_name}={offset_val}"));
        meta.stmt_queue.push_back(format!("{offset_name}=$(({offset_name} > 0 ? {offset_name} : 0))"));
        let offset_val = format!("${offset_name}");
        // Cap the slice length at zero.
        let length_name = format!("__SLICE_LENGTH_{}", meta.gen_value_id());
        let length_val = translate_computation(meta, ArithOp::Sub, Some(upper_val), Some(offset_val));
        meta.stmt_queue.push_back(format!("{local}{length_name}={length_val}"));
        meta.stmt_queue.push_back(format!("{length_name}=$(({length_name} > 0 ? {length_name} : 0))"));
        (format!("${offset_name}"), format!("${length_name}"))
    }
}

impl DocumentationModule for Range {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
