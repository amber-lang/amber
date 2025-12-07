use crate::modules::loops::iter_loop::IterLoop;
use crate::modules::expression::binop::range::Range;
use crate::modules::prelude::*;
use crate::translate::fragments::get_variable_name;
use crate::modules::types::Type;
use crate::{fragments, raw_fragment};

/// Trait to handle range loop translations.
///
/// This trait isolates the logic for optimizing and translating
/// range-based loops (e.g. `for i in 0..10`) from the main IterLoop logic.
pub trait IterLoopRange {
    /// Attempts to optimize range loops at compile-time when bounds are integer literals.
    /// Falls back to dynamic range loop generation for variable bounds.
    fn translate_range_loop(
        &self, 
        range: &Range, 
        meta: &mut TranslateMetadata
    ) -> FragmentKind;

    /// Defines index iterator variable if used
    fn translate_range_loop_index_fragments(&self) -> (FragmentKind, FragmentKind);

    fn translate_dynamic_range_loop(
        &self, range: &Range,
        meta: &mut TranslateMetadata,
        from_var: FragmentKind,
        to_var: FragmentKind
    ) -> FragmentKind;

    fn translate_static_range_loop(
        &self,
        range: &Range,
        meta: &mut TranslateMetadata,
        from_val: isize,
        to_val: isize
    ) -> FragmentKind;
}

impl IterLoopRange for IterLoop {
    fn translate_range_loop(
        &self, 
        range: &Range, 
        meta: &mut TranslateMetadata
    ) -> FragmentKind {
        // Static range
        if let (Some(from_val), Some(to_val)) = (range.from.get_integer_value(), range.to.get_integer_value()) {
            return self.translate_static_range_loop(range, meta, from_val, to_val);
        }
        // Dynamic range
        let id = self.iter_global_id.expect("No global ID set for loop iterator");
        let from = range.from.translate(meta);
        let from_var = meta.push_ephemeral_variable(VarStmtFragment::new("__range_start", Type::Int, from).with_global_id(id))
            .with_quotes(false).to_frag();

        let to = range.to.translate(meta);
        let to_var = meta.push_ephemeral_variable(VarStmtFragment::new("__range_end", Type::Int, to).with_global_id(id))
            .with_quotes(false).to_frag();

        self.translate_dynamic_range_loop(range, meta, from_var, to_var)
    }

    /// Defines index iterator variable if used
    fn translate_range_loop_index_fragments(&self) -> (FragmentKind, FragmentKind) {
        match (self.iter_index.as_ref(), self.iter_index_global_id) {
            (Some(index), Some(global_id)) => {
                let idx_var = get_variable_name(index, Some(global_id));
                (raw_fragment!(", {idx_var}=0"), raw_fragment!(", {idx_var}++"))
            },
            _ => (FragmentKind::Empty, FragmentKind::Empty)
        }
    }

    fn translate_dynamic_range_loop(
        &self, range: &Range,
        meta: &mut TranslateMetadata,
        from_var: FragmentKind,
        to_var: FragmentKind
    ) -> FragmentKind {
        let id = self.iter_global_id.unwrap();
        let iter_name = raw_fragment!("{}", get_variable_name(&self.iter_name, self.iter_global_id));

        // Calculate direction
        // dir = from < to ? 1 : -1
        let dir_val = fragments!("$(( ", from_var.clone(), " <= ", to_var.clone(), " ? 1 : -1 ))");
        let dir_stmt = VarStmtFragment::new("__dir", Type::Int, dir_val).with_global_id(id);
        let dir_var = meta.push_ephemeral_variable(dir_stmt).with_quotes(false).to_frag();

        // Operator
        let op = raw_fragment!("{}", if range.neq { "<" } else { "<=" });
        let (index_init, index_update) = self.translate_range_loop_index_fragments();

        let body = self.block.translate(meta);

        let init = fragments!(iter_name.clone(), "=", from_var, index_init);
        // We do a trick here by multiplying by dir so that we can use the same comparison operator
        // iter_name * dir_var < to_var * dir_var
        let cond = fragments!(iter_name.clone(), " * ", dir_var.clone(), " ", op, " ", to_var, " * ", dir_var.clone());
        let update = fragments!(iter_name, "+=", dir_var, index_update);

        fragments!(
            "for (( ", init, "; ", cond, "; ", update, " )); do\n",
            body,
            "\ndone"
        )
    }

    fn translate_static_range_loop(
        &self,
        range: &Range,
        meta: &mut TranslateMetadata,
        from_val: isize,
        to_val: isize
    ) -> FragmentKind {
        if range.neq && from_val == to_val {
            return FragmentKind::Empty;
        }
        
        let iter_name = raw_fragment!("{}", get_variable_name(&self.iter_name, self.iter_global_id));
        let (index_init, index_update) = self.translate_range_loop_index_fragments();

        let body = self.block.translate(meta);
        let (op, step) = if from_val <= to_val {(
            raw_fragment!("{}", if range.neq { "<" } else { "<=" }),
            raw_fragment!("++")
        )} else {(
            raw_fragment!("{}", if range.neq { ">" } else { ">=" }),
            raw_fragment!("--")
        )};
        
        let init = fragments!(iter_name.clone(), "=", raw_fragment!("{from_val}"), index_init);
        let cond = fragments!(iter_name.clone(), " ", op, " ", raw_fragment!("{to_val}"));
        let update = fragments!(iter_name, step, index_update);

        fragments!(
            "for (( ", init, "; ", cond, "; ", update, " )); do\n",
            body,
            "\ndone"
        )
    }
}
