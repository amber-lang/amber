use heraclitus_compiler::prelude::*;

use crate::modules::prelude::*;
use crate::modules::types::Typed;
use crate::modules::expression::expr::Expr;
use super::{variable_name_extensions, handle_identifier_name};
use crate::utils::context::{VariableDecl, VariableDeclWarn};
use crate::utils::metadata::ParserMetadata;

#[derive(Debug, Clone)]
pub struct VariableInit {
    name: String,
    expr: Box<Expr>,
    global_id: Option<usize>,
    is_fun_ctx: bool,
    is_const: bool,
    tok: Option<Token>,
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
        if let Err(err) = token(meta, "=") {
            return error_pos!(meta, err.unwrap_quiet(), format!("Expected '=' after variable name '{}'", self.name))
        }
        syntax(meta, &mut *self.expr)?;
        self.is_fun_ctx = meta.context.is_fun_ctx;
        Ok(())
    }
}

impl TypeCheckModule for VariableInit {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;
        handle_identifier_name(meta, &self.name, self.tok.clone())?;
        let var = VariableDecl::new(self.name.clone(), self.expr.get_type())
            .with_warn(VariableDeclWarn::from_token(meta, self.tok.clone())
                .warn_when_unmodified(!self.is_const && !meta.is_global_scope())
                .warn_when_unused(!meta.is_global_scope()))
            .with_const(self.is_const);
        self.global_id = meta.add_var(var);
        Ok(())
    }
}

impl TranslateModule for VariableInit {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let expr = self.expr.translate(meta);
        VarStmtFragment::new(&self.name, self.expr.get_type(), expr)
            .with_global_id(self.global_id)
            .with_local(self.is_fun_ctx)
            .to_frag()
    }
}

impl DocumentationModule for VariableInit {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
