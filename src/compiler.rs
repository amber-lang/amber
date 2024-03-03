use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::block::Block;
use crate::translate::check_all_blocks;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::rules;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::env;
use std::time::Instant;
use colored::Colorize;

const NO_CODE_PROVIDED: &str = "No code has been provided to the compiler";
const AMBER_DEBUG_PARSER: &str = "AMBER_DEBUG_PARSER";
const AMBER_DEBUG_TIME: &str = "AMBER_DEBUG_TIME";

pub struct AmberCompiler {
    pub cc: Compiler,
    pub path: Option<String>
}

impl AmberCompiler {
    pub fn new(code: String, path: Option<String>) -> AmberCompiler {
        AmberCompiler {
            cc: Compiler::new("Amber", rules::get_rules()),
            path
        }.load_code(code)
    }

    fn env_flag_set(flag: &str) -> bool {
        if let Ok(value) = env::var(flag) {
            value == "1" || value == "true"
        } else {
            false
        }
    }

    pub fn load_code(mut self, code: String) -> Self {
        self.cc.load(code);
        self
    }

    pub fn tokenize(&self) -> Result<Vec<Token>, Message> {
        let time = Instant::now();
        match self.cc.tokenize() {
            Ok(tokens) => {
                if Self::env_flag_set(AMBER_DEBUG_TIME) {
                    let pathname = self.path.clone().unwrap_or(String::from("unknown"));
                    println!("[{}]\tin\t{}ms\t{pathname}", "Tokenize".cyan(), time.elapsed().as_millis());
                }
                Ok(tokens)
            },
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

    pub fn parse(&self, tokens: Vec<Token>) -> Result<(Block, ParserMetadata), Message> {
        let code = self.cc.code.as_ref().expect(NO_CODE_PROVIDED).clone();
        let mut meta = ParserMetadata::new(tokens, self.path.clone(), Some(code));
        if let Err(Failure::Loud(err)) = check_all_blocks(&meta) {
            return Err(err);
        }
        let mut block = Block::new();
        let time = Instant::now();
        // Parse with debug or not
        let result = if Self::env_flag_set(AMBER_DEBUG_PARSER) {
            block.parse_debug(&mut meta)
        } else {
            block.parse(&mut meta)
        };
        if Self::env_flag_set(AMBER_DEBUG_TIME) {
            let pathname = self.path.clone().unwrap_or(String::from("unknown"));
            println!("[{}]\tin\t{}ms\t{pathname}", "Parsed".blue(), time.elapsed().as_millis());
        }
        // Return result
        match result {
            Ok(()) => Ok((block, meta)),
            Err(failure) => Err(failure.unwrap_loud())
        }
    }

    pub fn get_sorted_ast_forest(&self, block: Block, meta: &ParserMetadata) -> Vec<(String, Block)> {
        let imports_sorted = meta.import_cache.topological_sort();
        let imports_blocks = meta.import_cache.files.iter()
            .map(|file| file.metadata.as_ref().map(|meta| (file.path.clone(), meta.block.clone())))
            .collect::<Vec<Option<(String, Block)>>>();
        let mut result = vec![];
        for index in imports_sorted.iter() {
            if let Some((path, block)) = imports_blocks[*index].clone() {
                result.push((path, block));
            }
        }
        result.push((self.path.clone().unwrap_or(String::from("unknown")), block));
        result
    }

    pub fn translate(&self, block: Block, meta: ParserMetadata) -> String {
        let ast_forest = self.get_sorted_ast_forest(block, &meta);
        let mut meta_translate = TranslateMetadata::new(meta);
        let time = Instant::now();
        let mut result = vec![];
        for (_path, block) in ast_forest {
            result.push(block.translate(&mut meta_translate));
        }
        if Self::env_flag_set(AMBER_DEBUG_TIME) {
            let pathname = self.path.clone().unwrap_or(String::from("unknown"));
            println!("[{}]\tin\t{}ms\t{pathname}", "Translate".magenta(), time.elapsed().as_millis());
        }
        result.join("\n")
    }

    fn get_file_path(&self, path: &str) -> String {
        for (index, char) in path.chars().rev().enumerate() {
            if char == '/' {
                return path.to_string()
            }
            if char == '.' {
                return path[..path.len() - (index + 1)].to_string()
            }
        }
        path.to_string()
    }

    pub fn document(&self, block: Block, meta: ParserMetadata) {
        let ast_forest = self.get_sorted_ast_forest(block, &meta);
        for (path, block) in ast_forest {
            let document = block.document();
            // Save to file
            let path = format!("{}.md", self.get_file_path(path.as_str()));
            let mut file = File::create(path).unwrap();
            file.write_all(document.as_bytes()).unwrap();
        }
    }

    pub fn compile(&self) -> Result<(Vec<Message>, String), Message> {
        self.tokenize()
            .and_then(|tokens| self.parse(tokens))
            .map(|(block, meta)| (meta.messages.clone(), self.translate(block, meta)))
    }

    pub fn execute(code: String, flags: &[String]) {
        let code = format!("set -- {};\n\n{}", flags.join(" "), code);
        Command::new("/bin/bash").arg("-c").arg(code).spawn().unwrap().wait().unwrap();
    }

    pub fn generate_docs(&self) -> Result<(), Message> {
        self.tokenize().and_then(|tokens| self.parse(tokens))
            .map(|(block, meta)| self.document(block, meta))
    }

    #[allow(dead_code)]
    pub fn test_eval(&mut self) -> Result<String, Message> {
        self.compile().map_or_else(Err, |(_, code)| {
            let child = Command::new("/bin/bash")
                .arg("-c").arg::<&str>(code.as_ref())
                .output().unwrap();
            Ok(String::from_utf8_lossy(&child.stdout).to_string())
        })
    }

    pub fn import_std() -> String {
        [
            include_str!("std/main.ab")
        ].join("\n")
    }
}
