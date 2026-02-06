use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::{validate_index_accessor, variable_name_extensions};
use crate::translate::fragments::var_stmt::VarStmtFragment;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Access {
    pub left: Box<Expr>,
    pub index: Box<Option<Expr>>,
    pub kind: Type,
}

impl Typed for Access {
    fn get_type(&self) -> Type {
        if let Some(index) = self.index.as_ref() {
            match (&index.value, &self.kind) {
                (Some(ExprType::Range(_)), _) => self.kind.clone(),
                (Some(_), Type::Array(item_type)) => *item_type.clone(),
                _ => self.kind.clone(),
            }
        } else {
            self.kind.clone()
        }
    }
}

impl Default for Access {
    fn default() -> Self {
        Access {
            left: Box::new(Expr::new()),
            index: Box::new(None),
            kind: Type::Null,
        }
    }
}

impl Access {
    pub fn new() -> Self {
        Access::default()
    }

    pub fn set_left(&mut self, left: Expr) {
        *self.left = left;
    }

    pub fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Check if it's a destructuring assignment
        if self.is_destruct(meta) {
            return Err(Failure::Quiet(PositionInfo::from_token(
                meta,
                meta.get_current_token(),
            )));
        }
        token(meta, "[")?;
        Ok(())
    }

    fn is_destruct(&self, meta: &mut ParserMetadata) -> bool {
        let index = meta.get_index();
        if token(meta, "[").is_err() {
            meta.set_index(index);
            return false;
        }
        loop {
            if variable(meta, variable_name_extensions()).is_err() {
                meta.set_index(index);
                return false;
            }
            if token(meta, ",").is_err() {
                if token(meta, "]").is_ok() {
                    break;
                } else {
                    meta.set_index(index);
                    return false;
                }
            }
        }
        if token(meta, "=").is_err() {
            meta.set_index(index);
            return false;
        }
        meta.set_index(index);
        true
    }
}

impl SyntaxModule<ParserMetadata> for Access {
    syntax_name!("Access");

    fn new() -> Self {
        Access::new()
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let mut index = Expr::new();
        syntax(meta, &mut index)?;
        token(meta, "]")?;
        *self.index = Some(index);
        Ok(())
    }
}

impl TypeCheckModule for Access {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.left.typecheck(meta)?;
        self.kind = self.left.get_type();

        if let Some(ref mut index_expr) = self.index.as_mut() {
            let pos = self.left.get_position();
            if !self.kind.is_allowed_in(&Type::array_of(Type::Generic)) {
                return error_pos!(
                    meta,
                    pos,
                    format!(
                        "Cannot index a non-array expression of type '{}'",
                        self.kind
                    )
                );
            }

            index_expr.typecheck(meta)?;
            validate_index_accessor(meta, index_expr, true, pos)?;
        }

        Ok(())
    }
}

impl TranslateModule for Access {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let left_frag = self.left.translate(meta);
        match left_frag {
            FragmentKind::VarExpr(mut var) => {
                var.kind = self.get_type();
                var.with_index_by_expr(meta, *self.index.clone()).to_frag()
            }
            _ => {
                let id = meta.gen_value_id();
                let name = format!("access_{id}");
                let stmt = VarStmtFragment::new(&name, self.left.get_type(), left_frag)
                    .with_ephemeral(true);
                meta.stmt_queue.push_back(stmt.clone().to_frag());
                let mut var = VarExprFragment::from_stmt(&stmt);
                var.kind = self.get_type();
                var.with_index_by_expr(meta, *self.index.clone()).to_frag()
            }
        }
    }
}

impl DocumentationModule for Access {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
