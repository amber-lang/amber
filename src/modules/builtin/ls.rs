use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::ParserMetadata;
use crate::{fragments, raw_fragment};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Ls {
    value: Box<Option<Expr>>,
    all: Box<Option<Expr>>,
    recursive: Box<Option<Expr>>,
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
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
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
                } else {
                    *self.recursive = None;
                }
            } else {
                *self.all = None;
            }
        } else {
            *self.value = None;
            *self.all = None;
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
        Ok(())
    }
}

impl TypeCheckModule for Ls {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
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
    }
}

impl TranslateModule for Ls {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let handler = self.failure_handler.translate(meta);
        let path_fragment = match &*self.value {
            Some(path_expr) => path_expr.translate(meta),
            None => FragmentKind::Raw(RawFragment::new(".")),
        };

        let all_id = meta.gen_value_id();
        let all_frag = if let Some(all_expr) = &*self.all {
            let all_translate = all_expr.translate(meta);
            let all_var_stmt = VarStmtFragment::new("__ls", Type::Bool, FragmentKind::Empty).with_global_id(all_id);
            let all_expr = meta.push_ephemeral_variable(all_var_stmt);
            meta.stmt_queue.extend([
                fragments!(
                    "(( ",
                    all_translate,
                    " )) && ",
                    raw_fragment!("{}=\"-A\" || {}=\"\"", all_expr.get_name(), all_expr.get_name())
                )
            ]);
            all_expr.to_frag()
        } else {
            let all_var_stmt = VarStmtFragment::new("__ls", Type::Bool, fragments!("")).with_global_id(all_id);
            meta.push_ephemeral_variable(all_var_stmt).to_frag()
        };

        let recursive_id = meta.gen_value_id();
        let recursive_frag = if let Some(recursive_expr) = &*self.recursive {
            let recursive_translate = recursive_expr.translate(meta);
            let recursive_var_stmt = VarStmtFragment::new("__ls", Type::Bool, FragmentKind::Empty).with_global_id(recursive_id);
            let recursive_expr = meta.push_ephemeral_variable(recursive_var_stmt);
            meta.stmt_queue.extend([
                fragments!(
                    "(( ",
                    recursive_translate,
                    " )) && ",
                    raw_fragment!("{}=\"-R\" || {}=\"\"", recursive_expr.get_name(), recursive_expr.get_name())
                )
            ]);
            recursive_expr.to_frag()
        } else {
            let recursive_var_stmt = VarStmtFragment::new("__ls", Type::Bool, fragments!("")).with_global_id(recursive_id);
            meta.push_ephemeral_variable(recursive_var_stmt).to_frag()
        };

        let id = meta.gen_value_id();
        let var_stmt =
            VarStmtFragment::new("__ls", Type::array_of(Type::Text), FragmentKind::Empty)
                .with_global_id(id);
        let var_expr = meta.push_ephemeral_variable(var_stmt);
        meta.stmt_queue.extend([
            fragments!(
                raw_fragment!("read -rd '' -a {} < <(", var_expr.get_name()),
                "ls -1 ",
                all_frag.with_quotes(false),
                " ",
                recursive_frag.with_quotes(false),
                " ",
                path_fragment
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
