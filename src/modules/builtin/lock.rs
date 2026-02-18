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
    syntax_name!("Lock");

    fn new() -> Self {
        Lock { path: None }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let position = meta.get_index();
        token(meta, "lock")?;

        if token(meta, "(").is_ok() {
            if token(meta, ")").is_err() {
                let mut expr = Expr::new();
                syntax(meta, &mut expr)?;
                self.path = Some(expr);
                token(meta, ")")?;
            } else {
                self.path = None;
            }
        } else {
            let tok = meta.get_token_at(position);
            let warning = Message::new_warn_at_token(meta, tok)
                .message("Calling a builtin without parentheses is deprecated");
            meta.add_message(warning);
            let mut expr = Expr::new();
            syntax(meta, &mut expr)?;
            self.path = Some(expr);
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
        let lock_var_expr = RawFragment::new(&lock_var).to_frag();

        let lock_path_expr = self
            .path
            .as_ref()
            .map(|expr| expr.translate(meta))
            .unwrap_or(RawFragment::new("/tmp/${0##*/}.lock").to_frag());

        // Variable assignment: lock_var=path
        meta.stmt_queue.push_back(fragments!(
            lock_var_expr,
            "=\"",
            lock_path_expr.clone(),
            "\"\n"
        ));

        // Atomic lock acquisition using noclobber
        let lock_var_name = format!("${{{}}}", lock_var);
        let lock_var_frag = RawFragment::new(&lock_var_name).to_frag();
        meta.stmt_queue.push_back(fragments!(
            "if ! ( set -o noclobber; echo $$ > \"",
            lock_var_frag.clone(),
            "\" ) 2>/dev/null; then\n    exit 1\nfi\n",
            "touch \"",
            lock_var_frag,
            "\"\n"
        ));

        // Install cleanup trap once (only if __amber_cleanup_files is not defined yet)
        let trap_install_check = RawFragment::new("${__amber_cleanup_files+x}").to_frag();
        meta.stmt_queue.push_back(fragments!(
            "if [ -z \"",
            trap_install_check,
            "\" ]; then trap 'for f in \"${__amber_cleanup_files[@]}\"; do rm -f \"$f\"; done' EXIT INT TERM; fi\n"
        ));

        // Add lock file to cleanup array
        let lock_path_clone = lock_path_expr.clone();
        meta.stmt_queue.push_back(fragments!(
            "if [ -z \"${__amber_cleanup_files+x}\" ]; then __amber_cleanup_files=( ",
            lock_path_clone,
            " ); else __amber_cleanup_files+=( ",
            lock_path_expr,
            " ); fi\n"
        ));

        FragmentKind::Empty
    }
}

impl DocumentationModule for Lock {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
