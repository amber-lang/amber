use crate::fragments;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::utils::{ParserMetadata, TranslateMetadata};
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "await"]
#[kind = "builtin_stmt"]
pub struct Await {
    pids: Expr,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
}

impl SyntaxModule<ParserMetadata> for Await {
    syntax_name!("AwaitProcesses");

    fn new() -> Self {
        Await {
            pids: Expr::new(),
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_, meta| {
            token(meta, "await")?;
            token(meta, "(")?;
            syntax(meta, &mut self.pids)?;
            token(meta, ")")?;

            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `await` command can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
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

impl TypeCheckModule for Await {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
            self.pids.typecheck(meta)?;
            let pids_type = self.pids.get_type();
            if pids_type != Type::array_of(Type::Int) && pids_type != Type::Int {
                let position = self.pids.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `await` can only be used with values of type Int or [Int]",
                    comment: format!("Given type: {}, expected type: {} or {}", pids_type, Type::Int, Type::array_of(Type::Int))
                });
            }
            self.failure_handler.typecheck(meta)?;
            Ok(())
        })
    }
}

impl TranslateModule for Await {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let pids = self.pids.translate(meta);
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
        BlockFragment::new(
            vec![
                fragments!(sudo_prefix, "wait ", pids, suppress, silent),
                handler,
            ],
            false,
        )
        .to_frag()
    }
}

impl DocumentationModule for Await {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
