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
    options: Box<Option<Expr>>,
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
            options: Box::new(None),
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
                let mut options_expr = Expr::new();
                syntax(meta, &mut options_expr)?;
                *self.options = Some(options_expr);
            } else {
                *self.options = None;
            }
        } else {
            *self.value = None;
            *self.options = None;
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
        if let Some(options) = &mut *self.options {
            options.typecheck(meta)?;
            let options_type = options.get_type();
            if options_type != Type::array_of(Type::Text) {
                let position = options.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `ls` can only be used with 2nd argument of type [Text]",
                    comment: format!("Given type: {}, expected type: {}", options_type, Type::array_of(Type::Text))
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
        let id = meta.gen_value_id();
        let var_stmt =
            VarStmtFragment::new("__array", Type::array_of(Type::Text), FragmentKind::Empty)
                .with_global_id(id);
        let var_expr = meta.push_ephemeral_variable(var_stmt);
        let options_frag = match &*self.options {
            Some(options_expr) => fragments!(
                raw_fragment!("read -rd '' -a {} < <(ls ", var_expr.get_name()),
                options_expr.translate(meta),
                " "
            ),
            None => raw_fragment!("read -rd '' -a {} < <(ls -1 ", var_expr.get_name()),
        };
        meta.stmt_queue.extend([
            fragments!(
                options_frag.with_quotes(false),
                path_fragment
            ),
            BlockFragment::new(vec![handler], true).to_frag(),
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
