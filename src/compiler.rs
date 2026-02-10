extern crate chrono;
use crate::docs::module::DocumentationModule;
use crate::get_version;
use crate::modules::block::Block;
use crate::modules::prelude::{BlockFragment, FragmentKind, FragmentRenderable, RawFragment};
use crate::modules::typecheck::TypeCheckModule;
use crate::optimizer::optimize_fragments;
use crate::rules;
use crate::translate::check_all_blocks;
use crate::translate::module::TranslateModule;
use crate::utils::{pluralize, ParserMetadata, TranslateMetadata};
use colored::Colorize;
use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use postprocessor::PostProcessor;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::iter::once;
use std::path::PathBuf;
use std::process::{exit, Command, ExitStatus};
use std::time::Instant;
use wildmatch::WildMatchPattern;

pub mod postprocessor;

const NO_CODE_PROVIDED: &str = "No code has been provided to the compiler";
const AMBER_DEBUG_PARSER: &str = "AMBER_DEBUG_PARSER";
const AMBER_DEBUG_TIME: &str = "AMBER_DEBUG_TIME";
const AMBER_NO_OPTIMIZE: &str = "AMBER_NO_OPTIMIZE";

pub struct CompilerOptions {
    pub no_proc: Vec<String>,
    pub minify: bool,
    pub test_mode: bool,
    pub test_name: Option<String>,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        let no_proc = vec![String::from("*")];
        Self {
            no_proc,
            minify: false,
            test_mode: false,
            test_name: None,
        }
    }
}

impl CompilerOptions {
    pub fn from_args(
        no_proc: &[String],
        minify: bool,
        test_mode: bool,
        test_name: Option<String>,
    ) -> Self {
        let no_proc = no_proc.to_owned();
        Self {
            no_proc,
            minify,
            test_mode,
            test_name,
        }
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
        let mut block = Block::new().with_no_syntax();
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
        let imports_blocks = meta
            .import_cache
            .files
            .iter()
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

    fn gen_header(&self) -> String {
        let header_template = if let Ok(dynamic) = env::var("AMBER_HEADER") {
            fs::read_to_string(&dynamic).unwrap_or_else(|_| {
                let msg = format!("Couldn't read the dynamic header file from path '{dynamic}'");
                Message::new_err_msg(msg).show();
                exit(1);
            })
        } else {
            include_str!("header.sh").trim_end().to_string()
        };

        header_template.replace("{{ version }}", get_version())
    }

    fn gen_footer(&self) -> String {
        let footer_template = if let Ok(dynamic) = env::var("AMBER_FOOTER") {
            fs::read_to_string(&dynamic).unwrap_or_else(|_| {
                let msg = format!("Couldn't read the dynamic footer file from path '{dynamic}'");
                Message::new_err_msg(msg).show();
                exit(1);
            })
        } else {
            String::new()
        };

        footer_template.replace("{{ version }}", get_version())
    }

    fn gen_sudo_preamble(&self) -> FragmentKind {
        let condition = r#"[ "$EUID" -ne 0 ] && { { command -v sudo >/dev/null 2>&1 && __sudo=sudo; } || { command -v doas >/dev/null 2>&1 && __sudo=doas; }; }"#;
        RawFragment::new(condition).to_frag()
    }

    pub fn translate(&self, block: Block, meta: ParserMetadata) -> Result<String, Message> {
        let sudo_used = meta.sudo_used;
        let ast_forest = self.get_sorted_ast_forest(block, &meta);
        let mut meta_translate = TranslateMetadata::new(meta, &self.options);
        let time = Instant::now();
        let mut result = BlockFragment::new(Vec::new(), false);

        if sudo_used {
            result.append(self.gen_sudo_preamble());
        }

        for (_path, block) in ast_forest {
            result.append(block.translate(&mut meta_translate));
        }
        if Self::env_flag_set(AMBER_DEBUG_TIME) {
            let pathname = self.path.clone().unwrap_or(String::from("unknown"));
            println!(
                "[{}]\tin\t{}ms\t{pathname}",
                "Translate".magenta(),
                time.elapsed().as_millis()
            );
        }

        let mut result = result.to_frag();
        if !Self::env_flag_set(AMBER_NO_OPTIMIZE) {
            optimize_fragments(&mut result);
        }

        let mut result = result.to_string(&mut meta_translate);

        let filters = self
            .options
            .no_proc
            .iter()
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
                }
            };
        }

        Ok(format!(
            "{}\n{}\n{}",
            self.gen_header(),
            result,
            self.gen_footer()
        ))
    }

    pub fn document(&self, block: Block, meta: ParserMetadata, output: Option<String>) {
        let base_path = meta
            .get_path()
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
            // Check if an output directory was specified.
            if let Some(output) = &output {
                // Save to file; replace the base directory if the output
                // path is absolute, otherwise append the output path.
                let dir_path = {
                    let file_path = dep_path.strip_prefix(&base_dir).unwrap();
                    let file_dir = file_path.parent().unwrap();
                    base_dir.join(output).join(file_dir)
                };
                if let Err(err) = fs::create_dir_all(dir_path.clone()) {
                    let message = format!(
                        "Couldn't create directory `{}`. Do you have sufficient permissions?",
                        dir_path.display()
                    );
                    Message::new_err_msg(message)
                        .comment(err.to_string())
                        .show();
                    std::process::exit(1);
                }
                let filename = dep_path.file_stem().unwrap().to_string_lossy();
                let path = dir_path.join(format!("{filename}.md"));
                let mut file = File::create(path.clone()).unwrap();
                file.write_all(document.as_bytes()).unwrap();
                paths.push(String::from(path.to_string_lossy()));
            } else {
                // Write to standard output.
                std::io::stdout().write_all(document.as_bytes()).unwrap();
            }
        }
        if !paths.is_empty() {
            let files = pluralize(paths.len(), "File", "Files");
            let message = once(format!("{files} generated at:"))
                .chain(paths)
                .join("\n");
            Message::new_info_msg(message).show();
        }
    }

    pub fn typecheck(
        &self,
        mut block: Block,
        mut meta: ParserMetadata,
    ) -> Result<(Block, ParserMetadata), Message> {
        let time = Instant::now();

        // Perform type checking on the block
        if let Err(failure) = block.typecheck(&mut meta) {
            return Err(failure.unwrap_loud());
        }

        if Self::env_flag_set(AMBER_DEBUG_TIME) {
            let pathname = self.path.clone().unwrap_or(String::from("unknown"));
            println!(
                "[{}]\tin\t{}ms\t{pathname}",
                "Typecheck".green(),
                time.elapsed().as_millis()
            );
        }

        Ok((block, meta))
    }

    pub fn compile(&self) -> Result<(Vec<Message>, String), Message> {
        let tokens = self.tokenize()?;
        let (block, meta) = self.parse(tokens)?;
        let (block, meta) = self.typecheck(block, meta)?;
        let messages = meta.messages.clone();
        let code = self.translate(block, meta)?;
        Ok((messages, code))
    }

    pub fn execute(mut code: String, args: Vec<String>) -> Result<ExitStatus, std::io::Error> {
        if let Some(mut command) = Self::find_bash() {
            if !args.is_empty() {
                let args = args
                    .into_iter()
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

    pub fn generate_docs(&self, output: Option<String>, usage: bool) -> Result<(), Message> {
        let tokens = self.tokenize()?;
        let (block, meta) = self.parse(tokens)?;
        let (block, mut meta) = self.typecheck(block, meta)?;
        meta.doc_usage = usage;
        self.document(block, meta, output);
        Ok(())
    }

    #[cfg(test)]
    pub fn test_eval(&mut self) -> Result<String, Message> {
        self.options.no_proc = vec!["*".into()];
        self.compile().map_or_else(Err, |(warnings, code)| {
            if let Some(mut command) = Self::find_bash() {
                let child = command
                    .arg("-c")
                    .arg::<&str>(code.as_ref())
                    .output()
                    .unwrap();
                let output = String::from_utf8_lossy(&child.stdout).to_string();
                let err_output = String::from_utf8_lossy(&child.stderr).to_string();
                let warning_log = {
                    let warn_map = warnings.iter().map(|warn| {
                        warn.message
                            .clone()
                            .unwrap_or_else(|| "Empty warning".to_string())
                    });
                    let warn_log = warn_map.collect::<Vec<String>>().join("\n");
                    if warn_log.is_empty() {
                        String::new()
                    } else {
                        warn_log + "\n"
                    }
                };
                Ok(warning_log + &output + &err_output)
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

    /// Return bash command. In some situations, mainly for testing purposes, this can return a command, for example, containerized execution which is not bash but behaves like bash.
    #[cfg(not(windows))]
    fn find_bash() -> Option<Command> {
        if env::var("AMBER_TEST_STRATEGY").is_ok_and(|value| value == "docker") {
            let mut command = Command::new("docker");
            let args_string = env::var("AMBER_TEST_ARGS")
                .expect("Please pass docker arguments in AMBER_TEST_ARGS environment variable.");
            let args: Vec<&str> = args_string.split_whitespace().collect();
            command.args(args);
            Some(command)
        } else {
            let mut command = Command::new("/usr/bin/env");
            command.arg("bash");
            Some(command)
        }
    }
}
