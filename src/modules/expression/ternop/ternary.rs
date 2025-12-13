use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::fragments;
use crate::modules::expression::binop::get_binop_position_info;
use crate::modules::types::{Type, Typed};
use crate::modules::expression::expr::Expr;
use super::TernOp;

#[derive(Debug, Clone)]
pub struct Ternary {
    cond: Box<Expr>,
    true_expr: Box<Expr>,
    false_expr: Box<Expr>,
    kind: Type
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
        *self.true_expr = middle;
    }

    fn set_right(&mut self, right: Expr) {
        *self.false_expr = right;
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
            true_expr: Box::new(Expr::new()),
            false_expr: Box::new(Expr::new()),
            kind: Type::Null
        }
    }

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Ternary {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.cond.typecheck(meta)?;
        if self.cond.get_type() != Type::Bool {
            let msg = self.cond.get_error_message(meta)
                .message("Expected expression that evaluates to 'Bool' in ternary condition");
            return Err(Failure::Loud(msg));
        }

        self.true_expr.typecheck(meta)?;
        self.false_expr.typecheck(meta)?;
        
        let true_type = self.true_expr.get_type();
        let false_type = self.false_expr.get_type();
        
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
             
            if let Some(kind) = resolved_type {
                self.kind = kind;
                return Ok(());
            }

            let pos = get_binop_position_info(meta, &self.true_expr, &self.false_expr);
            let msg = Message::new_err_at_position(meta, pos)
                .message("Ternary operation can only evaluate to value of one type.")
                .comment(format!("Provided branches of type '{}' and '{}'.",
                    true_type,
                    false_type));
            return Err(Failure::Loud(msg));
        }
        Ok(())
    }
}

impl TranslateModule for Ternary {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let is_array = self.true_expr.get_type().is_array();
        let cond = self.cond.translate(meta);
        let true_expr = self.true_expr.translate(meta);
        let false_expr = self.false_expr.translate(meta);
        let expr = fragments!("if [ ", cond, " != 0 ]; then echo ", true_expr, "; else echo ", false_expr, "; fi");
        if is_array {
            let id = meta.gen_value_id();
            let value = SubprocessFragment::new(expr).with_quotes(false).to_frag();
            let var_stmt = VarStmtFragment::new("ternary", self.true_expr.get_type(), value).with_global_id(id);
            meta.push_ephemeral_variable(var_stmt).to_frag()
        } else {
            SubprocessFragment::new(expr).to_frag()
        }
    }
}

impl DocumentationModule for Ternary {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
