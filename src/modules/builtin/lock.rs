use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Lock {
    path: Option<Expr>,
}

impl SyntaxModule<ParserMetadata> for Lock {
    syntax_name!("LockFile");

    fn new() -> Self {
        Lock { path: None }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let position = meta.get_index();
        token(meta, "lock")?;

        if token(meta, "(").is_ok() {
            // Check if there's an argument or if it's just lock()
            if token(meta, ")").is_err() {
                let mut expr = Expr::new();
                syntax(meta, &mut expr)?;
                self.path = Some(expr);
                token(meta, ")")?;
            }
        } else {
            let tok = meta.get_token_at(position);
            let warning = Message::new_warn_at_token(meta, tok)
                .message("Calling a builtin without parentheses is deprecated");
            meta.add_message(warning);
        }
        Ok(())
    }
}

impl TypeCheckModule for Lock {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Some(ref mut expr) = self.path {
            expr.typecheck(meta)?;
            
            let path_type = expr.get_type();
            if path_type != Type::Text {
                let position = expr.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `lock` can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", path_type, Type::Text)
                });
            }
        }
        Ok(())
    }
}

impl TranslateModule for Lock {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let lock_var = format!("__lock_file_{}", meta.gen_value_id());

        let lock_var_frag = RawFragment::new(&lock_var).to_frag();
        let eq_frag = RawFragment::new("=").to_frag();
        
        let lock_path_expr = self
            .path
            .as_ref()
            .map(|expr| expr.translate(meta))
            .unwrap_or(RawFragment::new("/tmp/${0##*/}.lock").to_frag());

        let if_check = RawFragment::new("if [ -f \"${").to_frag();
        let var_ref = RawFragment::new(&lock_var).to_frag();
        let close_var = RawFragment::new("}\" ]; then\n").to_frag();

        let exit_frag = RawFragment::new("    exit 1\n").to_frag();
        let fi_frag = RawFragment::new("fi\n").to_frag();

        let touch_code = RawFragment::new("touch \"${").to_frag();
        let touch_var = RawFragment::new(&lock_var).to_frag();
        let touch_close = RawFragment::new("}\"\n").to_frag();

        // TODO fix, don't use the path parameter
        let trap_cleanup = RawFragment::new("trap 'rm -f \"/tmp/${0##*/}.lock\"' EXIT\n").to_frag();

        fragments!(
            lock_var_frag,
            eq_frag,
            lock_path_expr,
            "\n",
            if_check,
            var_ref,
            close_var,
            exit_frag,
            fi_frag,
            touch_code,
            touch_var,
            touch_close,
            trap_cleanup
        )
    }
}

impl DocumentationModule for Lock {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
