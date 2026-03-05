use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::{fragments, raw_fragment};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Cp {
    source: Box<Expr>,
    destination: Box<Expr>,
    force: Box<Option<Expr>>,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
}

impl SyntaxModule<ParserMetadata> for Cp {
    syntax_name!("CopyFiles");

    fn new() -> Self {
        Cp {
            source: Box::new(Expr::new()),
            destination: Box::new(Expr::new()),
            force: Box::new(None),
            failure_handler: FailureHandler::new(),
            modifier: CommandModifier::new_expr(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_, meta| {
            token(meta, "cp")?;
            token(meta, "(")?;
            syntax(meta, &mut *self.source)?;
            token(meta, ",")?;
            syntax(meta, &mut *self.destination)?;
            if token(meta, ",").is_ok() {
                let mut force_expr = Expr::new();
                syntax(meta, &mut force_expr)?;
                *self.force = Some(force_expr);
            }
            token(meta, ")")?;

            // Handle optional failure handler (failed/succeeded/exited blocks)
            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `cp` command can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
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

impl TypeCheckModule for Cp {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
            self.source.typecheck(meta)?;
            let source_type = self.source.get_type();
            if source_type != Type::Text {
                let position = self.source.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `cp` can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", source_type, Type::Text)
                });
            }

            self.destination.typecheck(meta)?;
            let dest_type = self.destination.get_type();
            if dest_type != Type::Text {
                let position = self.destination.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `cp` can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", dest_type, Type::Text)
                });
            }

            if let Some(force_expr) = &mut *self.force {
                force_expr.typecheck(meta)?;
                if force_expr.get_type() != Type::Bool {
                    let position = force_expr.get_position();
                    return error_pos!(meta, position => {
                        message: "Builtin function `cp` can only be used with 3rd argument of type Bool",
                        comment: format!("Given type: {}, expected type: {}", force_expr.get_type(), Type::Bool)
                    });
                }
            }

            self.failure_handler.typecheck(meta)?;
            Ok(())
        })
    }
}

impl TranslateModule for Cp {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let source = self.source.translate(meta);
        let destination = self.destination.translate(meta);
        let handler = self.failure_handler.translate(meta);
        let sudo_prefix = meta.with_sudoed(self.modifier.is_sudo || meta.sudoed, |meta| {
            meta.gen_sudo_prefix().to_frag()
        });
        let silent = meta.with_silenced(self.modifier.is_silent || meta.silenced, |meta| {
            meta.gen_silent().to_frag()
        });
        let suppress = meta.with_suppress(self.modifier.is_suppress || meta.suppress, |meta| {
            meta.gen_suppress().to_frag()
        });

        let force_id = meta.gen_value_id();
        let force_frag = if let Some(force_expr) = &*self.force {
            let force_translate = force_expr.translate(meta);
            let force_var_stmt = VarStmtFragment::new("__cp", Type::Bool, FragmentKind::Empty)
                .with_global_id(force_id);
            let force_expr = meta.push_ephemeral_variable(force_var_stmt);
            meta.stmt_queue.extend([fragments!(
                "(( ",
                force_translate,
                " )) && ",
                raw_fragment!(
                    "{}=\"-f\" || {}=\"\"",
                    force_expr.get_name(),
                    force_expr.get_name()
                )
            )]);
            force_expr.to_frag()
        } else {
            let recursive_var_stmt = VarStmtFragment::new("__cp", Type::Bool, raw_fragment!(""))
                .with_global_id(force_id);
            meta.push_ephemeral_variable(recursive_var_stmt).to_frag()
        };

        BlockFragment::new(
            vec![
                fragments!(
                    sudo_prefix,
                    "cp -r ",
                    force_frag.with_quotes(false),
                    " ",
                    source,
                    " ",
                    destination,
                    suppress,
                    silent
                ),
                handler,
            ],
            false,
        )
        .to_frag()
    }
}

impl DocumentationModule for Cp {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
