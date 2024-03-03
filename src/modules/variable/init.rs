use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::types::{Typed, Type};
use crate::modules::expression::expr::Expr;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use super::{variable_name_extensions, handle_identifier_name};

#[derive(Debug, Clone)]
pub struct VariableInit {
    name: String,
    expr: Box<Expr>,
    global_id: Option<usize>,
    is_fun_ctx: bool
}

impl VariableInit {
    fn handle_add_variable(&mut self, meta: &mut ParserMetadata, name: &str, kind: Type, tok: Option<Token>) -> SyntaxResult {
        handle_identifier_name(meta, name, tok)?;
        self.global_id = meta.add_var(name, kind);
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for VariableInit {
    syntax_name!("Variable Initialize");

    fn new() -> Self {
        VariableInit {
            name: String::new(),
            expr: Box::new(Expr::new()),
            global_id: None,
            is_fun_ctx: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "let")?;
        // Get the variable name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        context!({
            token(meta, "=")?;
            syntax(meta, &mut *self.expr)?;
            // Add a variable to the memory
            self.handle_add_variable(meta, &self.name.clone(), self.expr.get_type(), tok)?;
            self.is_fun_ctx = meta.context.is_fun_ctx;
            Ok(())
        }, |position| {
            error_pos!(meta, position, format!("Expected '=' after variable name '{}'", self.name))
        })
    }
}

impl TranslateModule for VariableInit {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = self.name.clone();
        let mut  expr = self.expr.translate(meta);
        if let Type::Array(_) = self.expr.get_type() {
            expr = format!("({expr})");
        }
        let local = if self.is_fun_ctx { "local " } else { "" };
        match self.global_id {
            Some(id) => format!("__{id}_{name}={expr}"),
            None => format!("{local}{name}={expr}")
        }
    }
}

impl DocumentationModule for VariableInit {
    fn document(&self) -> String {
        "".to_string()
    }
}
