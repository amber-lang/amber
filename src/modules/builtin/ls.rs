use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::ParserMetadata;
use crate::{fragments, raw_fragment};
use heraclitus_compiler::prelude::*;
use crate::modules::command::modifier::CommandModifier;

#[derive(Debug, Clone)]
pub struct Ls {
    value: Box<Option<Expr>>,
    all: Box<Option<Expr>>,
    recursive: Box<Option<Expr>>,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
}

impl Typed for Ls {
    fn get_type(&self) -> Type {
        Type::array_of(Type::Text)
    }
}

impl SyntaxModule<ParserMetadata> for Ls {
    syntax_name!("ListDirectory");

    fn new() -> Self {
        Ls {
            value: Box::new(None),
            all: Box::new(None),
            recursive: Box::new(None),
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |this, meta| {
            token(meta, "ls")?;
            token(meta, "(")?;
            let mut path = Expr::new();
            if syntax(meta, &mut path).is_ok() {
                *self.value = Some(path);
                if token(meta, ",").is_ok() {
                    let mut all_expr = Expr::new();
                    syntax(meta, &mut all_expr)?;
                    *self.all = Some(all_expr);
                    if token(meta, ",").is_ok() {
                        let mut recursive_expr = Expr::new();
                        syntax(meta, &mut recursive_expr)?;
                        *self.recursive = Some(recursive_expr);
                    }
                }
            }
            token(meta, ")")?;

            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `ls` command can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
                            comment: "You can use '?' to propagate failure, 'failed' block to handle failure, 'succeeded' block to handle success, 'exited' block to handle both, or 'trust' modifier to ignore results"
                        });
                    }
                    _ => return Err(e),
                }
            }

            if let Some(silent_position) = &this.silent_position {
                if this.is_silent {
                    return error_pos!(meta, silent_position.clone() => {
                        message: "Builtin `ls` can't be used with the silent modifier.",
                        comment: "You can use the suppress modifier to suppress stderr output."
                    })
                }
            }
            Ok(())
        })
    }
}

impl TypeCheckModule for Ls {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
            if let Some(path) = &mut *self.value {
                path.typecheck(meta)?;
                let path_type = path.get_type();
                if path_type != Type::Text {
                    let position = path.get_position();
                    return error_pos!(meta,  position => {
                        message: "Builtin function `ls` can only be used with 1st argument of type Text",
                        comment: format!("Given type: {}, expected type: {}", path_type, Type::Text)
                    });
                }
            }
            if let Some(all) = &mut *self.all {
                all.typecheck(meta)?;
                let options_type = all.get_type();
                if options_type != Type::Bool {
                    let position = all.get_position();
                    return error_pos!(meta, position => {
                        message: "Builtin function `ls` can only be used with 2nd argument of type Bool",
                        comment: format!("Given type: {}, expected type: {}", options_type, Type::Bool)
                    });
                }
            }

            if let Some(recursive) = &mut *self.recursive {
                recursive.typecheck(meta)?;
                let recursive_type = recursive.get_type();
                if recursive_type != Type::Bool {
                    let position = recursive.get_position();
                    return error_pos!(meta, position => {
                        message : "Builtin function `ls` can only be used with 3rd argument of type Bool",
                        comment : format!("Given type: {}, expected type: {}", recursive_type, Type::Bool)
                    });
                }
            }

            self.failure_handler.typecheck(meta)?;
            Ok(())
        })
    }
}

impl TranslateModule for Ls {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let id = meta.gen_value_id();
        let handler = self.failure_handler.translate(meta);
        let path_fragment = match &*self.value {
            Some(path_expr) => path_expr.translate(meta),
            None => FragmentKind::Raw(RawFragment::new(".")),
        };

        // Escape backslashes in path for pathname expansion while preserving glob characters.
        let path_var_stmt = VarStmtFragment::new("__ls_path", Type::Text, path_fragment)
            .with_global_id(id);
        let path_expr = meta.push_ephemeral_variable(path_var_stmt);
        // Escape backslashes
        meta.stmt_queue.push_back(raw_fragment!(
            "{}=\"${{{}//\\\\/\\\\\\\\}}\"",
            path_expr.get_name(), path_expr.get_name()
        ));

        // Only create variables for all/recursive when expressions are provided
        let all_frag = if let Some(all_expr) = &*self.all {
            let all_translate = all_expr.translate(meta);
            let all_var_name = format!("__ls_all_{}", id);
            meta.stmt_queue.push_back(fragments!(
                "(( ",
                all_translate,
                " )) && ",
                raw_fragment!("{}=\"-A\" || {}=\"\"", all_var_name, all_var_name)
            ));
            raw_fragment!(" ${{{}}}", all_var_name)
        } else {
            FragmentKind::Empty
        };

        let recursive_frag = if let Some(recursive_expr) = &*self.recursive {
            let recursive_translate = recursive_expr.translate(meta);
            let recursive_var_name = format!("__ls_rec_{}", id);
            meta.stmt_queue.push_back(fragments!(
                "(( ",
                recursive_translate,
                " )) && ",
                raw_fragment!("{}=\"-R\" || {}=\"\"", recursive_var_name, recursive_var_name)
            ));
            raw_fragment!(" ${{{}}}", recursive_var_name)
        } else {
            FragmentKind::Empty
        };

        let suppress = meta.with_suppress(self.modifier.is_suppress || meta.suppress, |meta| {
            meta.gen_suppress().to_frag()
        });
        let sudo_prefix = meta.with_sudoed(self.modifier.is_sudo || meta.sudoed, |meta| {
            meta.gen_sudo_prefix().to_frag()
        });

        let var_stmt = VarStmtFragment::new("__ls", Type::array_of(Type::Text), FragmentKind::Empty)
            .with_global_id(id);
        let var_expr = meta.push_ephemeral_variable(var_stmt);
        meta.stmt_queue.extend([
            fragments!(
                raw_fragment!("IFS=$'\\n' read -rd '' -a {} < <(IFS=$'\\n';", var_expr.get_name()),
                sudo_prefix,
                "ls -1",
                all_frag,
                recursive_frag,
                " ",
                path_expr.to_frag().with_quotes(false),
                suppress
            ),
            handler,
            fragments!(")"),
        ]);
        var_expr.to_frag()
    }
}

impl DocumentationModule for Ls {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
