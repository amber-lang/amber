use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::handle_identifier_name;
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct VariableInit {
    name: String,
    expr: Box<Expr>,
    global_id: Option<usize>,
    is_fun_ctx: bool,
    is_const: bool,
}

impl VariableInit {
    fn handle_add_variable(
        &mut self,
        meta: &mut ParserMetadata,
        tok: Option<Token>,
    ) -> SyntaxResult {
        handle_identifier_name(meta, &self.name, tok)?;
        self.global_id = meta.add_var(
            &self.name,
            self.expr.get_type(),
            self.expr.get_payload(),
            self.is_const,
        );
        if let Some(mut payload) = self.expr.get_payload() {
            payload.set_var_name(&self.name, self.global_id);
        }
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
            is_fun_ctx: false,
            is_const: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let keyword = token_by(meta, |word| ["let", "const"].contains(&word.as_str()))?;
        self.is_const = keyword == "const";
        // Get the variable name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        context!({
            token(meta, "=")?;
            syntax(meta, &mut *self.expr)?;
            // Add a variable to the memory
            self.handle_add_variable(meta, tok)?;
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
        let mut expr = self.expr.translate(meta);
        if let Type::Array(_) = self.expr.get_type() {
            expr = format!("({expr})");
        }
        if let Some(id) = self.global_id {
            format!("__{id}_{name}={expr}")
        } else if self.is_fun_ctx {
            format!("local {name}={expr}")
        } else {
            format!("{name}={expr}")
        }
    }
}

impl DocumentationModule for VariableInit {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
