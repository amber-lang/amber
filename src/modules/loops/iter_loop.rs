use heraclitus_compiler::prelude::*;

use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::prelude::{RawFragment, FragmentKind};
use crate::modules::types::{Typed, Type};
use crate::modules::variable::variable_name_extensions;
use crate::translate::fragments::get_variable_name;
use crate::translate::module::TranslateModule;
use crate::utils::context::{Context, VariableDecl, VariableDeclWarn};
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::block::Block;
use crate::{fragments, raw_fragment};
use crate::modules::prelude::*;
use crate::modules::expression::binop::range::Range;

#[derive(Debug, Clone)]
pub struct IterLoop {
    block: Block,
    iter_expr: Expr,
    iter_index: Option<String>,
    iter_index_global_id: Option<usize>,
    iter_name: String,
    iter_name_tok: Option<Token>,
    iter_global_id: Option<usize>,
    iter_type: Type,
    iter_index_tok: Option<Token>,
}

impl SyntaxModule<ParserMetadata> for IterLoop {
    syntax_name!("Iter Loop");

    fn new() -> Self {
        IterLoop {
            block: Block::new().with_needs_noop().with_condition(),
            iter_expr: Expr::new(),
            iter_index: None,
            iter_index_global_id: None,
            iter_name: String::new(),
            iter_name_tok: None,
            iter_global_id: None,
            iter_type: Type::Generic,
            iter_index_tok: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "for")?;
        self.iter_name_tok = meta.get_current_token();
        self.iter_name = variable(meta, variable_name_extensions())?;
        if token(meta, ",").is_ok() {
            self.iter_index = Some(self.iter_name.clone());
            self.iter_index_tok = self.iter_name_tok.clone();
            self.iter_name_tok = meta.get_current_token();
            self.iter_name = variable(meta, variable_name_extensions())?;
        }
        token(meta, "in")?;
        // Parse iterable expression
        syntax(meta, &mut self.iter_expr)?;
        // Parse loop body
        syntax(meta, &mut self.block)?;
        Ok(())
    }
}

impl TranslateModule for IterLoop {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let iter_path = self.translate_path(meta);
        
        // Optimize range loops
        if iter_path.is_none() {
            if let Some(ExprType::Range(range)) = &self.iter_expr.value {
                return self.translate_range_loop(range, meta);
            }
        }

        let iter_name = raw_fragment!("{}", get_variable_name(&self.iter_name, self.iter_global_id));

        let for_loop_prefix = match iter_path.is_some() {
            true => fragments!("while IFS= read -r ", iter_name, "; do"),
            false => fragments!("for ", iter_name, " in ", self.iter_expr.translate(meta), "; do"),
        };
        let for_loop_suffix = match iter_path.is_some() {
            true => fragments!("done <", iter_path.unwrap()),
            false => fragments!("done"),
        };

        match (self.iter_index.as_ref(), self.iter_index_global_id) {
            (Some(index), global_id) => {
                let indent = TranslateMetadata::single_indent();
                let index = get_variable_name(index, global_id);
                BlockFragment::new(vec![
                    RawFragment::from(format!("{index}=0;")).to_frag(),
                    for_loop_prefix,
                    self.block.translate(meta),
                    RawFragment::from(format!("{indent}(( {index}++ )) || true")).to_frag(),
                    for_loop_suffix,
                ], false).to_frag()
            },
            _ => {
                BlockFragment::new(vec![
                    for_loop_prefix,
                    self.block.translate(meta),
                    for_loop_suffix,
                ], false).to_frag()
            },
        }
    }
}

impl TypeCheckModule for IterLoop {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.iter_expr.typecheck(meta)?;

        // Determine iterator type after typechecking
        self.iter_type = match self.iter_expr.get_type() {
            Type::Array(kind) => *kind,
            _ => {
                let pos = self.iter_expr.get_position();
                return error_pos!(meta, pos, "Expected iterable");
            }
        };

        // Create iterator variable
        meta.with_push_scope(true, |meta| {
            let var = VariableDecl::new(self.iter_name.clone(), self.iter_type.clone())
                .with_warn(VariableDeclWarn::from_token(meta, self.iter_name_tok.clone()));
            self.iter_global_id = meta.add_var(var);
            if let Some(index) = self.iter_index.as_ref() {
                let var = VariableDecl::new(index.clone(), Type::Int)
                    .with_warn(VariableDeclWarn::from_token(meta, self.iter_index_tok.clone()));
                self.iter_index_global_id = meta.add_var(var);
            }
            // Save loop context state and set it to true
            meta.with_context_fn(Context::set_is_loop_ctx, true, |meta| {
                // Type-check the loop body
                self.block.typecheck(meta)?;
                Ok(())
            })?;
            Ok(())
        })?;

        Ok(())
    }
}

impl IterLoop {
    fn translate_path(&self, meta: &mut TranslateMetadata) -> Option<FragmentKind> {
        if let Some(ExprType::LinesInvocation(value)) = &self.iter_expr.value {
            Some(value.translate_path(meta))
        } else {
            None
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

        // Define index handling if needed
        let (index_init, index_update) = match (self.iter_index.as_ref(), self.iter_index_global_id) {
            (Some(index), Some(global_id)) => {
                let idx_var = get_variable_name(index, Some(global_id));
                (raw_fragment!(", {idx_var}=0"), raw_fragment!(", {idx_var}++"))
            },
            _ => (FragmentKind::Empty, FragmentKind::Empty)
        };

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
    
    fn translate_range_loop(
        &self, 
        range: &Range, 
        meta: &mut TranslateMetadata
    ) -> FragmentKind {
        // Optimization: Try to resolve range bounds at compile time
        if let (Some(from_val), Some(to_val)) = (range.from.get_integer_value(), range.to.get_integer_value()) {
            return self.translate_static_range_loop(range, meta, from_val, to_val);
        }
        let id = self.iter_global_id.expect("No global ID set for loop iterator");
        // Dynamic range
        let from = range.from.translate(meta);
        let from_var = meta.push_ephemeral_variable(VarStmtFragment::new("__range_start", Type::Int, from).with_global_id(id))
            .with_quotes(false).to_frag();

        let to = range.to.translate(meta);
        let to_var = meta.push_ephemeral_variable(VarStmtFragment::new("__range_end", Type::Int, to).with_global_id(id))
            .with_quotes(false).to_frag();

        self.translate_dynamic_range_loop(range, meta, from_var, to_var)
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
        
        let iter_name = RawFragment::from(get_variable_name(&self.iter_name, self.iter_global_id)).to_frag();
        // Define index iterator variable if used
        let (index_init, index_update) = match (self.iter_index.as_ref(), self.iter_index_global_id) {
            (Some(index), Some(global_id)) => {
                let idx_var = get_variable_name(index, Some(global_id));
                (raw_fragment!(", {idx_var}=0"), raw_fragment!(", {idx_var}++"))
            },
            _ => (FragmentKind::Empty, FragmentKind::Empty)
        };

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

impl DocumentationModule for IterLoop {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
