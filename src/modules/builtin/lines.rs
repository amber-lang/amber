use crate::fragments;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;

use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::raw_fragment;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "lines"]
#[kind = "builtin_expr"]
pub struct LinesInvocation {
    pub path: Box<Option<Expr>>,
    pub modifier: CommandModifier,
    pub failure_handler: FailureHandler,
}

impl Typed for LinesInvocation {
    fn get_type(&self) -> Type {
        Type::array_of(Type::Text)
    }
}

impl SyntaxModule<ParserMetadata> for LinesInvocation {
    syntax_name!("Lines Invocation");

    fn new() -> Self {
        LinesInvocation {
            path: Box::new(None),
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_, meta| {
            token(meta, "lines")?;
            token(meta, "(")?;
            let mut path = Expr::new();
            syntax(meta, &mut path)?;
            token(meta, ")")?;
            *self.path = Some(path);

            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `lines` command can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
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

impl TypeCheckModule for LinesInvocation {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
            if let Some(path) = &mut *self.path {
                path.typecheck(meta)?;
                if path.get_type() != Type::Text {
                    let msg = format!(
                        "Expected value of type 'Text' but got '{}'",
                        path.get_type()
                    );
                    let pos = path.get_position();
                    return error_pos!(meta, pos, msg);
                }
                self.failure_handler.typecheck(meta)?;
                Ok(())
            } else {
                unreachable!()
            }
        })
    }
}

impl TranslateModule for LinesInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let temp = format!("__AMBER_LINE_{}", meta.gen_value_id());
        let path = (*self.path)
            .as_ref()
            .map(|p| p.translate(meta))
            .expect("Cannot read lines without provided path");
        let indent = TranslateMetadata::single_indent();
        let id = meta.gen_value_id();
        let var_stmt =
            VarStmtFragment::new("__array", Type::array_of(Type::Text), FragmentKind::Empty)
                .with_global_id(id);
        let var_expr = meta.push_ephemeral_variable(var_stmt);
        let handler = self.failure_handler.translate(meta);

        let has_sudo = self.modifier.is_sudo || meta.sudoed;
        let sudo_prefix = meta.with_sudoed(has_sudo, |meta| meta.gen_sudo_prefix().to_frag());

        // Using only suppress to hide stderr, as stdout is passed to arr
        let suppress = meta.with_suppress(
            self.modifier.is_suppress || meta.suppress || self.modifier.is_silent || meta.silenced,
            |meta| meta.gen_suppress().to_frag(),
        );

        let fifo_var = format!("__AMBER_FIFO_{}", meta.gen_value_id());
        let pid_var = format!("__AMBER_PID_{}", meta.gen_value_id());

        // Producer writes into the FIFO in the background so that:
        // - sudo works (process substitution `<(sudo ...)` strips sudo's TTY access)
        // - the producer's exit status is properly captured via `wait`
        let producer = if has_sudo {
            fragments!(
                sudo_prefix,
                " cat ",
                path,
                suppress,
                raw_fragment!(" >\"${fifo_var}\" &")
            )
        } else {
            fragments!("cat ", path, suppress, raw_fragment!(" >\"${fifo_var}\" &"))
        };

        meta.stmt_queue.extend([
            raw_fragment!("{fifo_var}=$(mktemp -u) || exit 1"), // panic if fifo cannot be created
            raw_fragment!("mkfifo \"${fifo_var}\" || exit 1"),
            producer,
            raw_fragment!("{pid_var}=$!"),
            // NOTE: The same read-loop pattern also exists in iter_loop.rs (IterLoop::translate).
            // If you change this, update that one too.
            raw_fragment!("while IFS= read -r {temp} || [ -n \"${temp}\" ]; do"),
            raw_fragment!("{indent}{}+=(\"${}\")", var_expr.get_name(), temp),
            raw_fragment!("done <\"${fifo_var}\""),
            raw_fragment!("wait ${pid_var}"),
            raw_fragment!("rm -f \"${fifo_var}\""),
            handler,
        ]);
        var_expr.to_frag()
    }
}

impl DocumentationModule for LinesInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
