use crate::fragments;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Mv {
    source: Box<Expr>,
    destination: Box<Expr>,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
}

impl SyntaxModule<ParserMetadata> for Mv {
    syntax_name!("MoveFiles");

    fn new() -> Self {
        Mv {
            source: Box::new(Expr::new()),
            destination: Box::new(Expr::new()),
            failure_handler: FailureHandler::new(),
            modifier: CommandModifier::new_expr(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            let position = meta.get_index();
            token(meta, "mv")?;

            if token(meta, "(").is_ok() {
                syntax(meta, &mut *self.source)?;
                token(meta, ",")?;
                syntax(meta, &mut *self.destination)?;
                token(meta, ")")?;
            } else {
                let tok = meta.get_token_at(position);
                let warning = Message::new_warn_at_token(meta, tok)
                    .message("Calling a builtin without parentheses is deprecated");
                meta.add_message(warning);

                syntax(meta, &mut *self.source)?;
                syntax(meta, &mut *self.destination)?;
            }

            // Handle optional failure handler (failed/succeeded/exited blocks)
            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `mv` command can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
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

impl TypeCheckModule for Mv {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
            self.source.typecheck(meta)?;
            self.destination.typecheck(meta)?;
            self.failure_handler.typecheck(meta)?;

            let source_type = self.source.get_type();
            if source_type != Type::Text {
                let position = self.source.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `mv` can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", source_type, Type::Text)
                });
            }

            let dest_type = self.destination.get_type();
            if dest_type != Type::Text {
                let position = self.destination.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `mv` can only be used with values of type Text",
                    comment: format!("Given type: {}, expected type: {}", dest_type, Type::Text)
                });
            }

            Ok(())
        })
    }
}

impl TranslateModule for Mv {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        meta.with_silenced(self.modifier.is_silent || meta.silenced, |meta| {
            meta.with_sudoed(self.modifier.is_sudo || meta.sudoed, |meta| {
                let source = self.source.translate(meta);
                let destination = self.destination.translate(meta);
                let handler = self.failure_handler.translate(meta);
                let silent = meta.gen_silent().to_frag();
                let sudo_prefix = meta.gen_sudo_prefix().to_frag();
                let translation = fragments!("mv ", source, " ", destination);

                BlockFragment::new(
                    vec![
                        ListFragment::new(vec![sudo_prefix, translation, silent])
                            .with_spaces()
                            .to_frag(),
                        handler,
                    ],
                    false,
                )
                .to_frag()
            })
        })
    }
}

impl DocumentationModule for Mv {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
