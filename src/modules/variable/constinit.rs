use heraclitus_compiler::prelude::*;
use serde::{Deserialize, Serialize};
use crate::docs::module::DocumentationModule;
use crate::modules::types::{Typed, Type};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use super::{variable_name_extensions, handle_identifier_name};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstInit {
    name: String,
    expr: Box<Expr>,
    global_id: Option<usize>,
    is_fun_ctx: bool
}

impl ConstInit {
    fn handle_add_const(&mut self, meta: &mut ParserMetadata, name: &str, kind: Type, tok: Option<Token>) -> SyntaxResult {
        handle_identifier_name(meta, name, tok)?;
        self.global_id = meta.add_var(name, kind, true);
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for ConstInit {
    syntax_name!("Constant Initialize");

    fn new() -> Self {
        ConstInit {
            name: String::new(),
            expr: Box::new(Expr::new()),
            global_id: None,
            is_fun_ctx: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "const")?;
        // Get the variable name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        context!({
            token(meta, "=")?;
            syntax(meta, &mut *self.expr)?;
            // Add a variable to the memory
            self.handle_add_const(meta, &self.name.clone(), self.expr.get_type(), tok)?;
            self.is_fun_ctx = meta.context.is_fun_ctx;
            Ok(())
        }, |position| {
            error_pos!(meta, position, format!("Expected '=' after variable name '{}'", self.name))
        })
    }
}

impl TranslateModule for ConstInit {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = self.name.clone();
        let mut  expr = self.expr.translate(meta);
        if let Type::Array(_) = self.expr.get_type() {
            expr = format!("({expr})");
        }
        match self.global_id {
            Some(id) => format!("declare -r __{id}_{name}={expr}"),
            None => format!("declare -r {name}={expr}")
        }
    }
}

impl DocumentationModule for ConstInit {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
