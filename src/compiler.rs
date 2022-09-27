use heraclitus_compiler::prelude::*;
use crate::modules::block;
use crate::translate::check_all_blocks;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::rules;
use std::process::Command;
use std::env;

const NO_CODE_PROVIDED: &str = "No code has been provided to the compiler";

pub struct AmberCompiler {
    pub cc: Compiler,
    pub path: Option<String>,
    pub is_parse_debug: bool,
}

impl AmberCompiler {
    pub fn new(code: String, path: Option<String>) -> AmberCompiler {
        AmberCompiler {
            cc: Compiler::new("Amber", rules::get_rules()),
            path,
            is_parse_debug: false,
        }.load_code(code)
    }

    pub fn load_code(mut self, code: String) -> Self {
        self.cc.load(code);
        self
    }

    pub fn tokenize(&self) -> Result<Vec<Token>, Message> {
        match self.cc.tokenize() {
            Ok(tokens) => Ok(tokens),
            Err((err_type, pos)) => {
                let error_message = match err_type {
                    LexerErrorType::Singleline => {
                        format!("Singleline {} not closed", pos.data.as_ref().unwrap())
                    },
                    LexerErrorType::Unclosed => {
                        format!("Unclosed {}", pos.data.as_ref().unwrap())
                    }
                };
                let code = self.cc.code.as_ref().expect(NO_CODE_PROVIDED).clone();
                let meta = ParserMetadata::new(vec![], self.path.clone(), Some(code));
                Err(Message::new_err_at_position(&meta, pos).message(error_message))
            }
        }
    }

    pub fn parse(&self, tokens: Vec<Token>) -> Result<(block::Block, ParserMetadata), Message> {
        let code = self.cc.code.as_ref().expect(NO_CODE_PROVIDED).clone();
        let mut meta = ParserMetadata::new(tokens, self.path.clone(), Some(code));
        if let Err(Failure::Loud(err)) = check_all_blocks(&mut meta) {
            return Err(err);
        }
        let mut block = block::Block::new();
        // Parse with debug or not
        let result = if let Ok(value) = env::var("AMBER_DEBUG_PARSER") {
            if value == "true" {
                block.parse_debug(&mut meta)
            } else {
                block.parse(&mut meta)
            }
        } else {
            block.parse(&mut meta)
        };
        // Return result
        match result {
            Ok(()) => Ok((block, meta)),
            Err(failure) => Err(failure.unwrap_loud())
        }
    }

    pub fn translate(&self, block: block::Block, meta: ParserMetadata) -> String {
        let mut meta = TranslateMetadata::new(&meta);
        block.translate(&mut meta)
    }

    pub fn compile(&self) -> Result<String, Message> {
        self.tokenize()
            .and_then(|tokens| self.parse(tokens))
            .map(|(block, meta)| self.translate(block, meta))
    }

    pub fn execute(code: String, flags: &[String]) {
        let code = format!("set -- {};\n\n{}", flags.join(" "), code);
        Command::new("/bin/bash").arg("-c").arg(code).spawn().unwrap().wait().unwrap();
    }

    #[allow(dead_code)]
    pub fn test_eval(&self) -> Result<String, Message> {
        self.compile().map_or_else(Err, |code| {
            let child = Command::new("/bin/bash")
                .arg("-c").arg::<&str>(code.as_ref())
                .output().unwrap();
            Ok(String::from_utf8_lossy(&child.stdout).to_string())
        })
    }
}