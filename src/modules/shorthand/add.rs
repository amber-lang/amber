use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::modules::variable::{handle_variable_reference, prevent_constant_mutation, variable_name_extensions};
use crate::translate::compute::translate_computation_eval;
use crate::translate::{compute::ArithOp, module::TranslateModule};
use crate::modules::types::{Type, Typed};

use super::shorthand_typecheck_allowed_types;

#[derive(Debug, Clone)]
pub struct ShorthandAdd {
    var: String,
    expr: Box<Expr>,
    kind: Type,
    global_id: Option<usize>,
    is_ref: bool,
    tok: Option<Token>,
}

impl SyntaxModule<ParserMetadata> for ShorthandAdd {
    syntax_name!("Shorthand Add");

    fn new() -> Self {
        Self {
            var: String::new(),
            expr: Box::new(Expr::new()),
            kind: Type::Null,
            global_id: None,
            is_ref: false,
            tok: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.tok = meta.get_current_token();
        self.var = variable(meta, variable_name_extensions())?;
        token(meta, "+=")?;
        syntax(meta, &mut *self.expr)?;
        Ok(())
    }
}

impl TypeCheckModule for ShorthandAdd {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;
        
        let variable = handle_variable_reference(meta, &self.tok, &self.var)?;
        prevent_constant_mutation(meta, &self.tok, &self.var, variable.is_const)?;
        meta.mark_var_modified(&self.var);
        self.kind = variable.kind;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        
        let right_type = self.expr.get_type();
        if let (Type::Array(inner_left), Type::Array(inner_right)) = (&self.kind, &right_type) {
            if *inner_left.as_ref() == Type::Generic && *inner_right.as_ref() != Type::Generic {
                meta.update_var_type(&self.var, right_type.clone());
                self.kind = right_type;
                return Ok(());
            }
        }

        shorthand_typecheck_allowed_types(meta, "add", &self.kind, &self.expr, &[
            Type::Num,
            Type::Int,
            Type::Text,
            Type::array_of(Type::Generic),
        ])?;
        Ok(())
    }
}

impl TranslateModule for ShorthandAdd {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let var = VarExprFragment::new(&self.var, self.kind.clone())
            .with_global_id(self.global_id)
            .with_ref(self.is_ref);
        match self.kind {
            Type::Text | Type::Array(_) => {
                let expr = self.expr.translate_eval(meta, self.is_ref);
                VarStmtFragment::new(&self.var, self.kind.clone(), expr)
                    .with_global_id(self.global_id)
                    .with_ref(self.is_ref)
                    .with_operator("+=")
                    .to_frag()
            }
            Type::Int => {
                let expr = self.expr.translate_eval(meta, self.is_ref);
                let expr = ArithmeticFragment::new(var.to_frag(), ArithOp::Add, expr).to_frag();
                VarStmtFragment::new(&self.var, self.kind.clone(), expr)
                    .with_global_id(self.global_id)
                    .with_ref(self.is_ref)
                    .with_operator("=")
                    .to_frag()
            }
            Type::Num => {
                let expr = self.expr.translate_eval(meta, self.is_ref);
                let expr = translate_computation_eval(meta, ArithOp::Add, Some(var.to_frag()), Some(expr), self.is_ref);
                VarStmtFragment::new(&self.var, self.kind.clone(), expr)
                    .with_global_id(self.global_id)
                    .with_ref(self.is_ref)
                    .with_operator("=")
                    .to_frag()
            }
            _ => unreachable!("Unsupported type {} in shorthand addition operation", self.kind)
        }
    }
}

impl DocumentationModule for ShorthandAdd {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
