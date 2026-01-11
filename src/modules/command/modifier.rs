use amber_meta::ContextManager;
use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::modules::block::Block;

#[derive(Debug, Clone, ContextManager)]
pub struct CommandModifier {
    pub block: Option<Box<Block>>,
    pub trust_position: Option<PositionInfo>,
    pub silent_position: Option<PositionInfo>,
    pub silent_err_position: Option<PositionInfo>,
    pub sudo_position: Option<PositionInfo>,
    #[context]
    pub is_trust: bool,
    pub is_silent: bool,
    pub is_silent_err: bool,
    pub is_sudo: bool,
}

impl CommandModifier {
    pub fn new_expr() -> Self {
        CommandModifier {
            block: None,
            is_trust: false,
            is_silent: false,
            is_silent_err: false,
            is_sudo: false,
            trust_position: None,
            silent_position: None,
            silent_err_position: None,
            sudo_position: None,
        }
    }

    pub fn use_modifiers<F>(
        &mut self, meta: &mut ParserMetadata, context: F
    ) -> SyntaxResult where F: FnOnce(&mut Self, &mut ParserMetadata) -> SyntaxResult {
        // The setter returns the old value
        let old_trust = meta.context.set_is_trust_ctx(self.is_trust || meta.context.is_trust_ctx);
        let result = context(self, meta);
        meta.context.set_is_trust_ctx(old_trust);
        result
    }

    fn parse_modifier_sequence(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        loop {
            match meta.get_current_token() {
                Some(tok) => {
                    match tok.word.as_str() {
                        trust @ ("trust" | "unsafe") => {
                            if trust == "unsafe" {
                                let message = Message::new_warn_at_token(meta, Some(tok.clone()))
                                .message("The keyword `unsafe` has been deprecated in favor of `trust`.")
                                .comment("Learn more about this change: https://docs.amber-lang.com/basic_syntax/commands#command-modifiers");
                                meta.add_message(message);
                            }
                            if self.is_trust {
                                return error!(meta, Some(tok.clone()), "You already declared `trust` modifier before");
                            }
                            self.is_trust = true;
                            self.trust_position = Some(PositionInfo::from_token(meta, Some(tok.clone())));
                            meta.increment_index();
                        },
                        "silent" => {
                            if self.is_silent {
                                return error!(meta, Some(tok.clone()), "You already declared `silent` modifier before");
                            }
                            if self.is_silent_err {
                                return error!(meta, Some(tok.clone()), "You already declared `silent_err` modifier before. You can't use them in conjunction.");
                            }
                            self.is_silent = true;
                            self.silent_position = Some(PositionInfo::from_token(meta, Some(tok.clone())));
                            meta.increment_index();
                        },
                        "silent_err" => {
                            if self.is_silent {
                                return error!(meta, Some(tok.clone()), "You already declared `silent` modifier before. You can't use them in conjunction.");
                            }
                            if self.is_silent_err {
                                return error!(meta, Some(tok.clone()), "You already declared `silent_err` modifier before");
                            }
                            self.is_silent_err = true;
                            self.silent_err_position = Some(PositionInfo::from_token(meta, Some(tok.clone())));
                            meta.increment_index();
                        }
                        "sudo" => {
                            if self.is_sudo {
                                return error!(meta, Some(tok.clone()), "Command modifier 'sudo' has already been declared");
                            }
                            self.is_sudo = true;
                            meta.sudo_used = true;
                            self.sudo_position = Some(PositionInfo::from_token(meta, Some(tok.clone())));
                            meta.increment_index();
                        },
                        _ => break
                    }
                },
                None => return Err(Failure::Quiet(PositionInfo::from_metadata(meta)))
            }
        }
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for CommandModifier {
    syntax_name!("Command Modifier");

    fn new() -> Self {
        CommandModifier {
            block: Some(Box::new(Block::new().with_no_indent())),
            is_trust: false,
            is_silent: false,
            is_silent_err: false,
            is_sudo: false,
            trust_position: None,
            silent_position: None,
            silent_err_position: None,
            sudo_position: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.parse_modifier_sequence(meta)?;
        if let Some(mut block) = self.block.take() {
            return self.use_modifiers(meta, |this, meta| {
                syntax(meta, &mut *block)?;
                this.block = Some(block);
                Ok(())
            })
        }
        Ok(())
    }
}

impl TypeCheckModule for CommandModifier {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Some(mut block) = self.block.take() {
            return self.use_modifiers(meta, |this, meta| {
                block.typecheck(meta)?;
                this.block = Some(block);
                Ok(())
            })
        }
        Ok(())
    }
}

impl TranslateModule for CommandModifier {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if let Some(block) = &self.block {
            meta.silenced = self.is_silent;
            meta.silenced_err = self.is_silent_err;
            meta.sudoed = self.is_sudo;
            let result = block.translate(meta);
            meta.silenced = false;
            meta.silenced_err = false;
            meta.sudoed = false;
            result
        } else {
            FragmentKind::Empty
        }
    }
}

impl DocumentationModule for CommandModifier {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
