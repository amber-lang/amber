use crate::fragments;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;

use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "touch"]
#[kind = "builtin_stmt"]
pub struct Touch {
    value: Expr,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
}

impl SyntaxModule<ParserMetadata> for Touch {
    syntax_name!("TouchFile");

    fn new() -> Self {
        Touch {
            value: Expr::new(),
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_, meta| {
            let position = meta.get_index();
            token(meta, "touch")?;

            if token(meta, "(").is_ok() {
                syntax(meta, &mut self.value)?;
                token(meta, ")")?;
            } else {
                let tok = meta.get_token_at(position);
                let warning = Message::new_warn_at_token(meta, tok)
                    .message("Calling a builtin without parentheses is deprecated");
                meta.add_message(warning);
                syntax(meta, &mut self.value)?;
            }

            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `touch` command can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
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

impl TypeCheckModule for Touch {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
            self.value.typecheck(meta)?;

            let path_type = self.value.get_type();
            if path_type != Type::Text {
                let position = self.value.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `touch` can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", path_type, Type::Text)
                });
            }

            self.failure_handler.typecheck(meta)?;
            Ok(())
        })
    }
}

impl TranslateModule for Touch {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let value = self.value.translate(meta);
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
                fragments!(sudo_prefix, "touch ", value, suppress, silent),
                handler,
            ],
            false,
        )
        .to_frag()
    }
}

impl DocumentationModule for Touch {
    fn document(&self, _meta: &ParserMetadata) -> String {
        String::new()
    }
}
