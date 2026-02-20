use crate::fragments;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::raw_fragment;
use crate::translate::fragments::var_stmt::VarStmtFragment;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Lock {
    path: Option<Expr>,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
}

impl SyntaxModule<ParserMetadata> for Lock {
    syntax_name!("Lock");

    fn new() -> Self {
        Lock {
            path: None,
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;

        self.modifier.use_modifiers(meta, |_, meta| {
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

            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `lock` builtin can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
                            comment: "You can use '?' to propagate failure, 'failed' block to handle failure, 'succeeded' block to handle success, 'exited' block to handle both, or 'trust' modifier to ignore results"
                        });
                    },
                    _ => return Err(e)
                }
            }
            Ok(())
        })
    }
}

impl TypeCheckModule for Lock {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
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

            self.failure_handler.typecheck(meta)?;
            Ok(())
        })
    }
}

impl TranslateModule for Lock {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let lock_var_id = meta.gen_value_id();
        let lock_path_expr = self
            .path
            .as_ref()
            .map(|expr| expr.translate(meta))
            .unwrap_or(raw_fragment!("/tmp/${{0##*/}}.lock"));

        let lock_var_stmt = VarStmtFragment::new(
            &format!("__lock_file_{}", lock_var_id),
            Type::Text,
            lock_path_expr,
        )
        .with_global_id(lock_var_id);
        let lock_var_expr = meta.push_ephemeral_variable(lock_var_stmt);

        let lock_var_frag = lock_var_expr.with_quotes(false).to_frag();
        let lock_var_frag_first = lock_var_frag.clone();
        let lock_var_frag_second = lock_var_frag.clone();
        let lock_var_frag_third = lock_var_frag.clone();

        let blocker = fragments!(
            "if ! ( set -o noclobber; echo $$ > \"",
            lock_var_frag_first,
            "\" ) 2>/dev/null; then\n    exit 1\nfi\n",
            "touch \"",
            lock_var_frag_second,
            "\"\n"
        );

        let cleanup_array_update = fragments!(
            "if [ -z \"${__amber_cleanup_files+x}\" ]; then __amber_cleanup_files=( \"",
            lock_var_frag_third.clone(),
            "\" ); else __amber_cleanup_files+=( \"",
            lock_var_frag_third,
            "\" ); fi\n"
        );

        BlockFragment::new(
            vec![
                blocker,
                cleanup_array_update,
                self.failure_handler.translate(meta),
            ],
            false,
        )
        .to_frag()
    }
}

impl DocumentationModule for Lock {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
