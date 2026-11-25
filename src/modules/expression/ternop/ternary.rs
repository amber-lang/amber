use super::TernOp;
use crate::fragments;
use crate::modules::expression::binop::get_binop_position_info;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::BoolAnalysis;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Ternary {
    cond: Box<Expr>,
    true_expr: Option<Box<Expr>>,
    false_expr: Option<Box<Expr>>,
    cfa: BoolAnalysis,
    kind: Type,
}

impl Typed for Ternary {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl TernOp for Ternary {
    fn set_left(&mut self, left: Expr) {
        *self.cond = left;
    }

    fn set_middle(&mut self, middle: Expr) {
        self.true_expr = Some(Box::new(middle));
    }

    fn set_right(&mut self, right: Expr) {
        self.false_expr = Some(Box::new(right));
    }

    fn parse_operator_left(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "then")?;
        Ok(())
    }

    fn parse_operator_right(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "else")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Ternary {
    syntax_name!("Ternary Expression");

    fn new() -> Self {
        Ternary {
            cond: Box::new(Expr::new()),
            true_expr: Some(Box::new(Expr::new())),
            false_expr: Some(Box::new(Expr::new())),
            cfa: BoolAnalysis::default(),
            kind: Type::Null,
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Ternary {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.cond.typecheck(meta)?;
        if !matches!(
            self.cond.get_type(),
            Type::Bool | Type::Text | Type::Array(_)
        ) {
            let msg = self
                .cond
                .get_error_message(meta)
                .message("Expected expression that evaluates to 'Bool' in ternary condition");
            return Err(Failure::Loud(msg));
        }

        // Handle static cases
        self.cfa = self.cond.analyze_control_flow();
        match (
            self.cfa.known_value,
            self.cfa.has_side_effects,
        ) {
            (Some(true), false) => {
                let (facts, _) = self.cond.extract_facts();
                let true_expr = self.true_expr.as_mut().unwrap();
                meta.with_narrowed_scope(facts, |meta| true_expr.typecheck(meta))?;
                self.kind = true_expr.get_type();
                self.false_expr = None;
                return Ok(());
            }
            (Some(false), false) => {
                let (_, facts) = self.cond.extract_facts();
                let false_expr = self.false_expr.as_mut().unwrap();
                meta.with_narrowed_scope(facts, |meta| false_expr.typecheck(meta))?;
                self.kind = false_expr.get_type();
                self.true_expr = None;
                return Ok(());
            }
            _ => {}
        }

        let (true_facts, false_facts) = self.cond.extract_facts();
        let true_expr = self.true_expr.as_mut().unwrap();
        let false_expr = self.false_expr.as_mut().unwrap();
        meta.with_narrowed_scope(true_facts, |meta| true_expr.typecheck(meta))?;
        meta.with_narrowed_scope(false_facts, |meta| false_expr.typecheck(meta))?;

        // Unification Logic for dynamic case
        let true_type = true_expr.get_type();
        let false_type = false_expr.get_type();

        if true_type == false_type {
            self.kind = true_type;
        } else {
            // Handle Array type inference
            let mut resolved_type = None;
            if let (Type::Array(t), Type::Array(f)) = (&true_type, &false_type) {
                if **t == Type::Generic && **f != Type::Generic {
                    resolved_type = Some(false_type.clone());
                } else if **t != Type::Generic && **f == Type::Generic {
                    resolved_type = Some(true_type.clone());
                }
            }
            // Check if after inference types match
            if let Some(kind) = resolved_type {
                self.kind = kind;
                return Ok(());
            }

            if let (Some(true_expr), Some(false_expr)) = (&self.true_expr, &self.false_expr) {
                let pos = get_binop_position_info(meta, true_expr, false_expr);
                let msg = Message::new_err_at_position(meta, pos)
                    .message("Ternary operation can only evaluate to value of one type.")
                    .comment(format!(
                        "Provided branches of type '{}' and '{}'.",
                        true_type, false_type
                    ));
                return Err(Failure::Loud(msg));
            }
        }
        Ok(())
    }
}

impl TranslateModule for Ternary {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let true_expr = || self.true_expr.as_ref().expect("Ternary true branch missing");
        let false_expr = || self.false_expr.as_ref().expect("Ternary false branch missing");

        match (self.cfa.known_value, self.cfa.has_side_effects) {
            (Some(true), false) => true_expr().translate(meta),
            (Some(false), false) => false_expr().translate(meta),
            _ => {
                let (true_expr, false_expr) = (true_expr(), false_expr());
                let true_type = true_expr.get_type();
                let cond = self.cond.translate(meta).with_condition(true);
                let id = meta.gen_value_id();
                let is_local = meta.fun_meta.is_some();

                let mut translate_branch = |expr: &Expr| {
                    let (mut stmts, value) = meta.capture_stmt_queue(|meta| expr.translate(meta));
                    let var_stmt = VarStmtFragment::new("ternary", true_type.clone(), value)
                        .with_global_id(id)
                        .with_local(is_local)
                        .with_optimization_when_unused(false);
                    stmts.push(var_stmt.to_frag());
                    BlockFragment::new(stmts, true).with_needs_noop(true).to_frag()
                };
                let true_branch = translate_branch(true_expr);
                let false_branch = translate_branch(false_expr);
                let if_stmt = BlockFragment::new(
                    vec![
                        fragments!("if ", cond, "; then"),
                        true_branch,
                        fragments!("else"),
                        false_branch,
                        fragments!("fi"),
                    ],
                    false,
                );
                meta.stmt_queue.push_back(if_stmt.to_frag());
                let var_stmt = VarStmtFragment::new("ternary", true_type, FragmentKind::Empty)
                    .with_global_id(id);
                VarExprFragment::from_stmt(&var_stmt).to_frag()
            }
        }
    }
}

crate::impl_documentation_noop!(Ternary);

crate::impl_reachable_break!(Ternary, cond, true_expr, false_expr);
