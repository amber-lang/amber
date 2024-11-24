extern crate chrono;
use crate::docs::module::DocumentationModule;
use crate::modules::block::Block;
use crate::translate::check_all_blocks;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::rules;
use postprocessor::PostProcessor;
use chrono::prelude::*;
use colored::Colorize;
use heraclitus_compiler::prelude::*;
use wildmatch::WildMatchPattern;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::time::Instant;

pub mod postprocessor;

const NO_CODE_PROVIDED: &str = "No code has been provided to the compiler";
const AMBER_DEBUG_PARSER: &str = "AMBER_DEBUG_PARSER";
const AMBER_DEBUG_TIME: &str = "AMBER_DEBUG_TIME";

pub struct CompilerOptions {
    pub no_proc: Vec<String>,
    pub minify: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        let no_proc = vec![String::from("*")];
        Self { no_proc, minify: false }
    }
}

impl CompilerOptions {
    pub fn from_args(no_proc: &[String], minify: bool) -> Self {
        let no_proc = no_proc.to_owned();
        Self { no_proc, minify }
    }
}

pub struct AmberCompiler {
    pub cc: Compiler,
    pub path: Option<String>,
    pub options: CompilerOptions,
}

impl AmberCompiler {
    pub fn new(code: String, path: Option<String>, options: CompilerOptions) -> AmberCompiler {
        let cc = Compiler::new("Amber", rules::get_rules());
        let compiler = AmberCompiler { cc, path, options };
        compiler.load_code(AmberCompiler::comment_shebang(code))
    }

    fn comment_shebang(code: String) -> String {
        if code.starts_with("#!") {
            String::from("// ") + &code
        } else {
            code
        }
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
                    println!(
                        "[{}]\tin\t{}ms\t{pathname}",
                        "Tokenize".cyan(),
                        time.elapsed().as_millis()
                    );
                }
                Ok(tokens)
            }
            Err((err_type, pos)) => {
                let error_message = match err_type {
                    LexerErrorType::Singleline => {
                        format!("Singleline {} not closed", pos.data.as_ref().unwrap())
                    }
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
            println!(
                "[{}]\tin\t{}ms\t{pathname}",
                "Parsed".blue(),
                time.elapsed().as_millis()
            );
        }
        // Return result
        match result {
            Ok(()) => Ok((block, meta)),
            Err(failure) => Err(failure.unwrap_loud()),
        }
    }

    pub fn get_sorted_ast_forest(
        &self,
        block: Block,
        meta: &ParserMetadata,
    ) -> Vec<(String, Block)> {
        let imports_sorted = meta.import_cache.topological_sort();
        let imports_blocks = meta.import_cache.files.iter()
            .map(|file| {
                file.metadata
                    .as_ref()
                    .map(|meta| (file.path.clone(), meta.block.clone()))
            })
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

    pub fn translate(&self, block: Block, meta: ParserMetadata) -> Result<String, Message> {
        let ast_forest = self.get_sorted_ast_forest(block, &meta);
        let mut meta_translate = TranslateMetadata::new(meta, &self.options);
        let time = Instant::now();
        let mut result = vec![];
        for (_path, block) in ast_forest {
            result.push(block.translate(&mut meta_translate));
        }
        if Self::env_flag_set(AMBER_DEBUG_TIME) {
            let pathname = self.path.clone().unwrap_or(String::from("unknown"));
            println!(
                "[{}]\tin\t{}ms\t{pathname}",
                "Translate".magenta(),
                time.elapsed().as_millis()
            );
        }

        let mut result = result.join("\n") + "\n";

        let filters = self.options.no_proc.iter()
            .map(|x| WildMatchPattern::new(x))
            .collect();
        let postprocessors = PostProcessor::filter_default(filters);
        for postprocessor in postprocessors {
            result = match postprocessor.execute(result) {
                Ok(result) => result,
                Err(error) => {
                    let error = format!(
                        "Postprocessor '{}' failed\n{}",
                        postprocessor.name,
                        error.to_string().trim_end(),
                    );
                    return Err(Message::new_err_msg(error));
                },
            };
        }

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let header = include_str!("header.sh")
            .replace("{{ version }}", env!("CARGO_PKG_VERSION"))
            .replace("{{ date }}", now.as_str());
        Ok(format!("{}{}", header, result))
    }

    pub fn document(&self, block: Block, meta: ParserMetadata, output: String) {
        let base_path = meta.get_path()
            .map(PathBuf::from)
            .expect("Input file must exist in docs generation");
        let base_dir = fs::canonicalize(base_path).map(|val| {
            val.parent()
                .expect("Parent dir must exist in docs generation")
                .to_owned()
                .clone()
        });
        let base_dir = base_dir.unwrap_or_else(|err| {
            Message::new_err_msg("Couldn't get the absolute path to the provided input file")
                .comment(err.to_string())
                .show();
            std::process::exit(1);
        });
        let ast_forest = self.get_sorted_ast_forest(block, &meta);
        let mut paths = vec![];
        for (path, block) in ast_forest {
            let dep_path = {
                let dep_path = match fs::canonicalize(PathBuf::from(path)) {
                    Ok(path) => path,
                    Err(_) => continue,
                };

                if !dep_path.starts_with(&base_dir) {
                    continue;
                }

                dep_path
            };
            let document = block.document(&meta);
            // Save to file; replace the base directory if the output
            // path is absolute, otherwise append the output path.
            let dir_path = {
                let file_path = dep_path.strip_prefix(&base_dir).unwrap();
                let file_dir = file_path.parent().unwrap();
                base_dir.join(&output).join(file_dir)
            };
            if let Err(err) = fs::create_dir_all(dir_path.clone()) {
                Message::new_err_msg(format!(
                    "Couldn't create directory `{}`. Do you have sufficient permissions?", dir_path.display()
                ))
                .comment(err.to_string())
                .show();
                std::process::exit(1);
            }
            let filename = dep_path.file_stem().unwrap().to_string_lossy();
            let path = dir_path.join(format!("{filename}.md"));
            let mut file = File::create(path.clone()).unwrap();
            file.write_all(document.as_bytes()).unwrap();
            paths.push(String::from(path.to_string_lossy()));
        }
        let file_text = if paths.len() > 1 { "Files" } else { "File" };
        Message::new_info_msg(format!("{file_text} generated at:\n{}", paths.join("\n")))
            .show();
    }

    pub fn compile(&self) -> Result<(Vec<Message>, String), Message> {
        let tokens = self.tokenize()?;
        let (block, meta) = self.parse(tokens)?;
        let messages = meta.messages.clone();
        let code = self.translate(block, meta)?;
        Ok((messages, code))
    }

    pub fn execute(mut code: String, args: Vec<String>) -> Result<ExitStatus, std::io::Error> {
        if let Some(mut command) = Self::find_bash() {
            if !args.is_empty() {
                let args = args.into_iter()
                    .map(|arg| arg.replace("\"", "\\\""))
                    .map(|arg| format!("\"{arg}\""))
                    .collect::<Vec<String>>();
                code = format!("set -- {}\n{}", args.join(" "), code);
            }
            command.arg("-c").arg(code).spawn()?.wait()
        } else {
            let error = std::io::Error::new(ErrorKind::NotFound, "Failed to find Bash");
            Err(error)
        }
    }

    pub fn generate_docs(&self, output: String, usage: bool) -> Result<(), Message> {
        let tokens = self.tokenize()?;
        let (block, mut meta) = self.parse(tokens)?;
        meta.doc_usage = usage;
        self.document(block, meta, output);
        Ok(())
    }

    #[cfg(test)]
    pub fn test_eval(&mut self) -> Result<String, Message> {
        self.options.no_proc = vec!["*".into()];
        self.compile().map_or_else(Err, |(_, code)| {
            if let Some(mut command) = Self::find_bash() {
                let child = command.arg("-c").arg::<&str>(code.as_ref()).output().unwrap();
                let output = String::from_utf8_lossy(&child.stdout).to_string();
                Ok(output)
            } else {
                let message = Message::new_err_msg("Failed to find Bash");
                Err(message)
            }
        })
    }

    #[cfg(windows)]
    fn find_bash() -> Option<Command> {
        if let Some(paths) = env::var_os("PATH") {
            for path in env::split_paths(&paths) {
                let path = path.join("bash.exe");
                if path.exists() {
                    let command = Command::new(path);
                    return Some(command);
                }
            }
        }
        return None;
    }

    #[cfg(not(windows))]
    fn find_bash() -> Option<Command> {
        if env::var("GITHUB_ACTIONS_BASH_CONTAINER").is_ok() {
            let mut command = Command::new("/usr/bin/docker");
            command.args(["exec", "--workdir", "/root", "--user", "405", "bash", "bash"]);
            return Some(command)
        } else {
            let mut command = Command::new("/usr/bin/env");
            command.arg("bash");
            return Some(command)
        }
    }
}
