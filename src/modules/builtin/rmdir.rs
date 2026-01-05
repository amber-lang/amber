use crate::{fragments, raw_fragment};
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::syntax_name;
use crate::modules::condition::failure_handler::FailureHandler;

#[derive(Debug, Clone)]
pub struct RmDir {
    value: Box<Expr>,
    ignore_non_empty: Box<Expr>,
    failure_handler: FailureHandler,
}

impl SyntaxModule<ParserMetadata> for RmDir {
    syntax_name!("RemoveDirectory");

    fn new() -> Self {
        RmDir {
            value: Box::new(Expr::new()),
            ignore_non_empty: Box::new(Expr::new()),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "rmdir")?;
        token(meta, "(")?;
        syntax(meta, &mut *self.value)?;
        token(meta, ",")?;
        syntax(meta, &mut *self.ignore_non_empty)?;
        token(meta, ")")?;

        if let Err(e) = syntax(meta, &mut self.failure_handler) {
            match e {
                Failure::Quiet(pos) => {
                    return error_pos!(meta, pos => {
                        message: "The `rmdir` command can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
                        comment: "You can use '?' to propagate failure, 'failed' block to handle failure, 'succeeded' block to handle success, 'exited' block to handle both, or 'trust' modifier to ignore results"
                    });
                }
                _ => return Err(e),
            }
        }
        Ok(())
    }
}

impl TypeCheckModule for RmDir {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value.typecheck(meta)?;
        if self.value.get_type() != Type::array_of(Type::Text) {
            let position = self.value.get_position();
            return error_pos!(meta, position => {
                message: "Builtin function `rmdir` can only be used with 1st argument of type [Text]",
                comment: format!("Given type: {}, expected type: {}", self.value.get_type(), Type::array_of(Type::Text))
            });
        }
        if self.ignore_non_empty.get_type() != Type::Bool {
            let position = self.ignore_non_empty.get_position();
            return error_pos!(meta, position => {
                message: "Builtin function `rmdir` can only be used with 2nd argument of type Bool",
                comment: format!("Given type: {}, expected type: {}", self.ignore_non_empty.get_type(), Type::Bool)
            });
        }
        self.failure_handler.typecheck(meta)?;
        Ok(())
    }
}

impl TranslateModule for RmDir {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let ignore_non_empty_translate = self.ignore_non_empty.translate(meta);
        let ignore_non_empty_id = meta.gen_value_id();
        let ignore_non_empty_var_stmt = VarStmtFragment::new("__force", Type::Bool, FragmentKind::Empty).with_global_id(ignore_non_empty_id);
        let ignore_non_empty_expr = meta.push_ephemeral_variable(ignore_non_empty_var_stmt);
        meta.stmt_queue.extend([
            fragments!(
                raw_fragment!("read -rd '' -a {} < <([[ ", ignore_non_empty_expr.get_name()),
                ignore_non_empty_translate,
                " == 1 ]] && echo \"--ignore-fail-on-non-empty\")"
            )
        ]);
        let ignore_non_empty_frag = ignore_non_empty_expr.to_frag();

        fragments!(
            "rmdir ",
            ignore_non_empty_frag.with_quotes(false),
            " ",
            self.value.translate(meta)
        )
    }
}

impl DocumentationModule for RmDir {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
