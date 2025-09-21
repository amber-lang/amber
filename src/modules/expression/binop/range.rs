use crate::modules::prelude::*;
use crate::{fragments, raw_fragment};
use crate::modules::expression::binop::BinOp;
use crate::modules::expression::expr::Expr;
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

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        // Type checking is now handled by TypeCheckModule
        Ok(())
    }
}

impl TypeCheckModule for Range {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        Self::typecheck_allowed_types(meta, "range operator", &self.from, &self.to, &[Type::Int])?;
        Ok(())
    }
}

impl TranslateModule for Range {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Try compile-time optimization first
        if let (Some(from_val), Some(to_val)) = (self.from.get_integer_value(), self.to.get_integer_value()) {
            return self.generate_compile_time_range(from_val, to_val);
        }
        
        // Fall back to runtime detection
        self.generate_runtime_range(meta)
    }
}

impl Range {
    /// Generate a range at compile time when both operands are numeric literals
    fn generate_compile_time_range(&self, from_val: isize, to_val: isize) -> FragmentKind {
        if self.is_reverse_range(from_val, to_val) {
            self.generate_reverse_seq(from_val, to_val)
        } else {
            self.generate_forward_seq(from_val, to_val)
        }
    }
    
    /// Generate a range at runtime when at least one operand is a variable
    fn generate_runtime_range(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let from = self.from.translate(meta);
        let to = self.to.translate(meta);
        let from_var = fragments!(from.clone());
        let to_var = fragments!(to.clone());
        
        let forward_to = self.adjust_end_for_forward_range(to.clone());
        let reverse_to = self.adjust_end_for_reverse_range(to.clone());
        
        let expr = fragments!(
            "if [ ", from_var.clone(), " -gt ", to_var.clone(), " ]; then seq ", from_var.clone(), " -1 ", reverse_to,
            "; else seq ", from_var, " ", forward_to, "; fi"
        );
        SubprocessFragment::new(expr).with_quotes(false).to_frag()
    }
    
    /// Check if this is a reverse range (start > end or equal with exclusive operator)
    fn is_reverse_range(&self, from_val: isize, to_val: isize) -> bool {
        from_val > to_val || (from_val == to_val && self.neq)
    }
    
    /// Generate a forward seq command for compile-time ranges
    fn generate_forward_seq(&self, from_val: isize, to_val: isize) -> FragmentKind {
        let to_adjusted = if self.neq { to_val - 1 } else { to_val };
        let expr = fragments!("seq ", raw_fragment!("{}", from_val), " ", raw_fragment!("{}", to_adjusted));
        SubprocessFragment::new(expr).with_quotes(false).to_frag()
    }
    
    /// Generate a reverse seq command for compile-time ranges
    fn generate_reverse_seq(&self, from_val: isize, to_val: isize) -> FragmentKind {
        let to_adjusted = if self.neq { to_val + 1 } else { to_val };
        let expr = fragments!("seq ", raw_fragment!("{}", from_val), " -1 ", raw_fragment!("{}", to_adjusted));
        SubprocessFragment::new(expr).with_quotes(false).to_frag()
    }
    
    /// Adjust the end value for forward ranges (runtime)
    fn adjust_end_for_forward_range(&self, to_raw: FragmentKind) -> FragmentKind {
        if self.neq {
            ArithmeticFragment::new(to_raw, ArithOp::Sub, fragments!("1")).to_frag()
        } else {
            to_raw
        }
    }
    
    /// Adjust the end value for reverse ranges (runtime)
    fn adjust_end_for_reverse_range(&self, to_raw: FragmentKind) -> FragmentKind {
        if self.neq {
            ArithmeticFragment::new(to_raw, ArithOp::Add, fragments!("1")).to_frag()
        } else {
            to_raw
        }
    }

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
