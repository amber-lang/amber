use crate::modules::prelude::*;
use crate::fragments;
use crate::modules::expression::binop::BinOp;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::{error_type_match, handle_binop};
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
    fn translate(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        let from = self.from.translate(meta);
        let to = if let Some(to) = self.to.get_integer_value() {
            if self.neq {
                RawFragment::new(&(to - 1).to_string()).to_frag()
            } else {
                RawFragment::new(&to.to_string()).to_frag()
            }
        } else {
            let to = self.to.translate(meta);
            if self.neq {
                translate_computation(meta, ArithOp::Sub, Some(to), Some(fragments!("1")))
            } else {
                to
            }
        };
        let expr = fragments!("seq ", from, " ", to);
        SubprocessFragment::new(expr).to_frag()
    }
}

impl Range {
    pub fn get_array_index(&self, meta: &mut TranslateMetadata) -> (TranslationFragment, TranslationFragment) {
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
                    RawFragment::new(&offset.to_string()).to_frag(),
                    RawFragment::new(&length.to_string()).to_frag()
                );
            }
        }
        // Make the upper bound exclusive.
        let upper = {
            let mut upper_val = self.to.translate(meta);
            if !self.neq {
                upper_val = translate_computation(meta, ArithOp::Add, Some(upper_val), Some(fragments!("1")));
            }
            let upper_id = Some(meta.gen_value_id());
            meta.push_stmt_variable("__slice_upper", upper_id, Type::Num, upper_val).to_frag()
        };

        // Cap the lower bound at zero.
        let offset = {
            let offset_id = Some(meta.gen_value_id());
            let offset_val = self.from.translate(meta);
            let offset_var = meta.push_stmt_variable("__slice_offset", offset_id, Type::Num, offset_val).to_frag();
            let offset_cap = fragments!("$((", offset_var.clone(), " > 0 ? ", offset_var, " : 0))");
            meta.push_stmt_variable("__slice_offset", offset_id, Type::Num, offset_cap).to_frag()
        };

        // Cap the slice length at zero.
        let length = {
            let length_id = Some(meta.gen_value_id());
            let length_val = translate_computation(meta, ArithOp::Sub, Some(upper), Some(offset.clone()));
            let length_var = meta.push_stmt_variable("__slice_length", length_id, Type::Num, length_val).to_frag();
            let length_cap = fragments!("$((", length_var.clone(), " > 0 ? ", length_var, " : 0))");
            meta.push_stmt_variable("__slice_length", length_id, Type::Num, length_cap).to_frag()
        };

        (offset, length)
    }
}

impl DocumentationModule for Range {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
