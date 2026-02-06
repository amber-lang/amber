use heraclitus_compiler::prelude::*;
use std::collections::HashSet;

use super::{handle_identifier_name, variable_name_extensions};
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::Typed;
use crate::utils::cc_flags::{get_ccflag_by_name, get_ccflag_name, CCFlags};
use crate::utils::context::{VariableDecl, VariableDeclWarn};
use crate::utils::metadata::ParserMetadata;

#[derive(Debug, Clone)]
pub struct VariableInit {
    name: String,
    expr: Box<Expr>,
    global_id: Option<usize>,
    is_fun_ctx: bool,
    is_const: bool,
    is_public: bool,
    tok: Option<Token>,
    flags: HashSet<CCFlags>,
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
            is_public: false,
            tok: None,
            flags: HashSet::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        while let Ok(flag) = token_by(meta, |val| val.starts_with("#[")) {
            self.flags
                .insert(get_ccflag_by_name(&flag[2..flag.len() - 1]));
        }

        if token(meta, "pub").is_ok() {
            self.is_public = true;
        }

        let keyword = token_by(meta, |word| ["let", "const"].contains(&word.as_str()))?;
        self.is_const = keyword == "const";
        self.tok = meta.get_current_token();

        if self.is_public && !self.is_const && !self.flags.contains(&CCFlags::AllowPublicMutable) {
            let flag = get_ccflag_name(CCFlags::AllowPublicMutable);
            return error!(meta, self.tok.clone() => {
                message: "Public variables must be constants",
                comment: format!("Mutable public variables create shared global state.\nUse 'pub const' or add '#[{flag}]' to allow this.")
            });
        }
        self.name = variable(meta, variable_name_extensions())?;
        if let Err(err) = token(meta, "=") {
            return error_pos!(
                meta,
                err.unwrap_quiet(),
                format!("Expected '=' after variable name '{}'", self.name)
            );
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
            .with_warn(
                VariableDeclWarn::from_token(meta, self.tok.clone())
                    .warn_when_unmodified(!self.is_const && !meta.is_global_scope())
                    .warn_when_unused(!meta.is_global_scope()),
            )
            .with_const(self.is_const)
            .with_public(self.is_public);
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
