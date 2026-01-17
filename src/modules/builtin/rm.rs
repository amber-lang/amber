use crate::{fragments, raw_fragment};
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::syntax_name;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;

#[derive(Debug, Clone)]
pub struct Rm {
    value: Box<Expr>,
    recursive: Box<Option<Expr>>,
    force: Box<Option<Expr>>,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
}

impl SyntaxModule<ParserMetadata> for Rm {
    syntax_name!("Remove");

    fn new() -> Self {
        Rm {
            value: Box::new(Expr::new()),
            recursive: Box::new(None),
            force: Box::new(None),
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_, meta| {
            token(meta, "rm")?;
            token(meta, "(")?;
            syntax(meta, &mut *self.value)?;
            if token(meta, ",").is_ok() {
                let mut recursive_expr = Expr::new();
                syntax(meta, &mut recursive_expr)?;
                *self.recursive = Some(recursive_expr);
            }
            if token(meta, ",").is_ok() {
                let mut force_expr = Expr::new();
                syntax(meta, &mut force_expr)?;
                *self.force = Some(force_expr);
            }
            token(meta, ")")?;

            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `rm` command can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
                            comment: "You can use '?' to propagate failure, 'failed' block to handle failure, 'succeeded' block to handle success, 'exited' block to handle both, or 'trust' modifier to ignore results"
                        });
                    }
                    _ => return Err(e),
                }
            }
            Ok(())
        })
    }
}

impl TypeCheckModule for Rm {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
            self.value.typecheck(meta)?;
            if self.value.get_type() != Type::Text {
                let position = self.value.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `rm` can only be used with 1st argument of type Text",
                    comment: format!("Given type: {}, expected type: {}", self.value.get_type(), Type::Text)
                });
            }
            if let Some(recursive_expr) = &mut *self.recursive {
                recursive_expr.typecheck(meta)?;
                if recursive_expr.get_type() != Type::Bool {
                    let position = recursive_expr.get_position();
                    return error_pos!(meta, position => {
                        message: "Builtin function `rm` can only be used with optional 2nd argument of type Bool",
                        comment: format!("Given type: {}, expected type: {}", recursive_expr.get_type(), Type::Bool)
                    });
                }
            }
            if let Some(force_expr) = &mut *self.force {
                force_expr.typecheck(meta)?;
                if force_expr.get_type() != Type::Bool {
                    let position = force_expr.get_position();
                    return error_pos!(meta, position => {
                    message: "Builtin function `rm` can only be used with 3rd argument of type Bool",
                    comment: format!("Given type: {}, expected type: {}", force_expr.get_type(), Type::Bool)
                });
                }
            }
            self.failure_handler.typecheck(meta)?;
            Ok(())
        })
    }
}

impl TranslateModule for Rm {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let recursive_id = meta.gen_value_id();
        let recursive_frag = if let Some(recursive_expr) = &*self.recursive {
            let recursive_translate = recursive_expr.translate(meta);
            let recursive_var_stmt = VarStmtFragment::new("__rm", Type::Bool, FragmentKind::Empty).with_global_id(recursive_id);
            let recursive_expr = meta.push_ephemeral_variable(recursive_var_stmt);
            meta.stmt_queue.extend([
                fragments!(
                    "(( ",
                    recursive_translate,
                    " )) && ",
                    raw_fragment!("{}=\"-r\" || {}=\"\"", recursive_expr.get_name(), recursive_expr.get_name())
                )
            ]);
            recursive_expr.to_frag()
        } else {
            let recursive_var_stmt = VarStmtFragment::new("__rm", Type::Bool, raw_fragment!("")).with_global_id(recursive_id);
            meta.push_ephemeral_variable(recursive_var_stmt).to_frag()
        };

        let force_id = meta.gen_value_id();
        let force_frag = if let Some(force_expr) = &*self.force {
            let force_translate = force_expr.translate(meta);
            let force_var_stmt = VarStmtFragment::new("__rm", Type::Bool, FragmentKind::Empty).with_global_id(force_id);
            let force_expr = meta.push_ephemeral_variable(force_var_stmt);
            meta.stmt_queue.extend([
                fragments!(
                    "(( ",
                    force_translate,
                    " )) && ",
                    raw_fragment!("{}=\"-f\" || {}=\"\"", force_expr.get_name(), force_expr.get_name())
                )
            ]);
            force_expr.to_frag()
        } else {
            let recursive_var_stmt = VarStmtFragment::new("__rm", Type::Bool, raw_fragment!("")).with_global_id(force_id);
            meta.push_ephemeral_variable(recursive_var_stmt).to_frag()
        };

        let silent = meta.with_silenced(self.modifier.is_silent || meta.silenced, |meta| {
            meta.gen_silent().to_frag()
        });
        let suppress = meta.with_suppress(self.modifier.is_suppress || meta.suppress, |meta| {
            meta.gen_suppress().to_frag()
        });
        let sudo_prefix = meta.with_sudoed(self.modifier.is_sudo || meta.sudoed, |meta| {
            meta.gen_sudo_prefix().to_frag()
        });

        fragments!(
            sudo_prefix,
            "rm ",
            force_frag.with_quotes(false),
            " ",
            recursive_frag.with_quotes(false),
            " ",
            self.value.translate(meta),
            suppress,
            silent
        )
    }
}

impl DocumentationModule for Rm {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
