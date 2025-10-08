use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::types::Typed;
use crate::modules::expression::expr::Expr;
use super::{variable_name_extensions, handle_identifier_name};

#[derive(Debug, Clone)]
pub struct VariableInit {
    name: String,
    expr: Box<Expr>,
    global_id: Option<usize>,
    is_fun_ctx: bool,
    is_const: bool,
    tok: Option<Token>,
}

impl VariableInit {
    fn handle_add_variable(
        &mut self,
        meta: &mut ParserMetadata,
    ) -> SyntaxResult {
        handle_identifier_name(meta, &self.name, self.tok.clone())?;
        self.global_id = meta.add_var(&self.name, self.expr.get_type(), self.is_const);
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
            is_const: false,
            tok: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let keyword = token_by(meta, |word| ["let", "const"].contains(&word.as_str()))?;
        self.is_const = keyword == "const";
        self.tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        context!({
            token(meta, "=")?;
            syntax(meta, &mut *self.expr)?;
            self.is_fun_ctx = meta.context.is_fun_ctx;
            Ok(())
        }, |position| {
            error_pos!(meta, position, format!("Expected '=' after variable name '{}'", self.name))
        })
    }
}

impl TranslateModule for VariableInit {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let expr = self.expr.translate(meta);
        VarStmtFragment::new(&self.name, self.expr.get_type(), expr)
            .with_global_id(self.global_id)
            .to_frag()
    }
}

impl TypeCheckModule for VariableInit {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;
        self.handle_add_variable(meta)?;
        Ok(())
    }
}

impl DocumentationModule for VariableInit {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
