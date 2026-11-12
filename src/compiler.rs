extern crate chrono;
use crate::docs::module::DocumentationModule;
use crate::get_version;
use crate::modules::block::Block;
use crate::modules::prelude::{BlockFragment, FragmentKind, FragmentRenderable, RawFragment};
use crate::modules::typecheck::TypeCheckModule;
use crate::optimizer::optimize_fragments;
use crate::rules;
use crate::stdlib;
use crate::translate::check_all_blocks;
use crate::translate::module::TranslateModule;
use crate::utils::{pluralize, ParserMetadata, ShellType, TranslateMetadata};
use colored::Colorize;
use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use postprocessor::PostProcessor;
use std::fs;
use std::io::{ErrorKind, Write};
use std::iter::once;
use std::path::PathBuf;
use std::process::{exit, ExitStatus};
use std::time::Instant;
use wildmatch::WildMatchPattern;

pub mod postprocessor;
pub mod shell_resolve;

#[derive(Clone, Debug)]
pub enum SourcePath {
    File { display: String, canonical: PathBuf },
    StdLib(String),
    Stdin,
}

impl PartialEq for SourcePath {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::File { canonical: candidate_path, .. },
                Self::File { canonical: target_path, .. },
            ) => candidate_path == target_path,
            (Self::StdLib(candidate_name), Self::StdLib(target_name)) => {
                candidate_name == target_name
            }
            (Self::Stdin, Self::Stdin) => true,
            _ => false,
        }
    }
}

impl SourcePath {
    pub fn file(display: impl Into<String>) -> std::io::Result<Self> {
        let path = display.into();
        let file_path = fs::canonicalize(&path)?;
        Ok(SourcePath::File { display: path, canonical: file_path })
    }

    pub fn join_relative(&self, relative: &str) -> Option<PathBuf> {
        let base = self.to_path_string()?;
        let mut path = PathBuf::from(base);
        path.pop();
        path.push(relative);
        Some(path)
    }

    pub fn to_path_string(&self) -> Option<&str> {
        match &self {
            Self::File { display, .. } => Some(display.as_str()),
            Self::Stdin => Some("."),
            Self::StdLib(_) => None
        }
    }

    /// Returns the standard library module name or `None` if this is a regular file / stdin
    pub fn stdlib_module_name(&self) -> Option<&str> {
        match self {
            Self::StdLib(name) => Some(name.strip_prefix("std/").unwrap_or(name)),
            Self::File { canonical, .. } => {
                let parent = canonical.parent()?.file_name()?.to_str()?;
                let stem = canonical.file_stem()?.to_str()?;
                (parent == "std" && stdlib::exists(stem)).then_some(stem)
            }
            Self::Stdin => None,
        }
    }

    pub fn to_string(&self) -> &str {
        match &self {
            Self::File { display, .. } => display.as_ref(),
            Self::StdLib(name) => name.as_ref(),
            Self::Stdin => "STDIN",
        }
    }
}


/// Escapes a string for safe use as a shell argument in double quotes.
/// Handles shell-special characters: $ ` " \ !
/// This prevents shell command injection attacks.
pub fn escape_shell_arg(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' | '$' | '`' | '\\' | '!' => format!("\\{}", c),
            _ => c.to_string(),
        })
        .join("")
}

pub struct CompilerOptions {
    pub no_proc: Vec<String>,
    pub target: Option<ShellType>,
    pub minify: bool,
    pub test_mode: bool,
    pub test_name: Option<String>,
    pub debug_parser: bool,
    pub debug_time: bool,
    pub no_optimize: bool,
    pub header_path: Option<String>,
    pub footer_path: Option<String>,
    pub shebang: Option<String>,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        let no_proc = vec![String::from("*")];
        Self {
            no_proc,
            target: None,
            minify: false,
            test_mode: false,
            test_name: None,
            debug_parser: false,
            debug_time: false,
            no_optimize: false,
            header_path: None,
            footer_path: None,
            shebang: None,
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
            target: None,
            minify,
            test_mode,
            test_name,
            debug_parser: false,
            debug_time: false,
            no_optimize: false,
            header_path: None,
            footer_path: None,
            shebang: None,
        }
    }

    pub fn with_env_vars(mut self) -> Self {
        let is_true = |v: String| v == "1" || v == "true";

        if std::env::var("AMBER_DEBUG_PARSER").is_ok_and(is_true) {
            self.debug_parser = true;
        }
        if std::env::var("AMBER_DEBUG_TIME").is_ok_and(is_true) {
            self.debug_time = true;
        }
        if std::env::var("AMBER_NO_OPTIMIZE").is_ok_and(is_true) {
            self.no_optimize = true;
        }
        if let Ok(path) = std::env::var("AMBER_HEADER") {
            self.header_path = Some(path);
        }
        if let Ok(path) = std::env::var("AMBER_FOOTER") {
            self.footer_path = Some(path);
        }
        self
    }

    pub fn with_target(mut self, target: Option<ShellType>) -> Self {
        self.target = target;
        self
    }

    pub fn with_shebang(mut self, shebang: Option<String>) -> Self {
        self.shebang = shebang;
        self
    }
}

pub struct AmberCompiler {
    pub cc: Compiler,
    pub path: SourcePath,
    pub options: CompilerOptions,
}

impl AmberCompiler {
    pub fn new(code: String, path: SourcePath, options: CompilerOptions) -> AmberCompiler {
        let code = code.replace("\r\n", "\n").replace('\r', "\n");
        let cc = Compiler::new("Amber", rules::get_rules(), AmberCompiler::comment_shebang(code));
        AmberCompiler { cc, path, options }
    }

    fn comment_shebang(code: String) -> String {
        if code.starts_with("#!") {
            String::from("// ") + &code
        } else {
            code
        }
    }

    pub fn tokenize(&self) -> Result<Vec<Token>, Message> {
        let time = Instant::now();
        match self.cc.tokenize() {
            Ok(tokens) => {
                if self.options.debug_time {
                    let pathname = self.path.to_string();
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
                let meta = ParserMetadata::new(vec![], self.path.clone(), &self.cc.code);
                Err(Message::new_err_at_position(&meta, pos).message(error_message))
            }
        }
    }

    pub fn parse(&self, tokens: Vec<Token>) -> Result<(Block, ParserMetadata), Message> {
        let mut meta = ParserMetadata::new(tokens, self.path.clone(), &self.cc.code);
        if let Err(Failure::Loud(err)) = check_all_blocks(&meta) {
            return Err(err);
        }
        let mut block = Block::new().with_no_syntax();
        let time = Instant::now();
        // Parse with debug or not
        let result = if self.options.debug_parser {
            block.parse_debug(&mut meta)
        } else {
            block.parse(&mut meta)
        };
        if self.options.debug_time {
            let pathname = self.path.to_string();
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
    ) -> Vec<(SourcePath, Block)> {
        let files = &meta.import_cache.files;
        meta.import_cache
            .topological_sort()
            .into_iter()
            .filter_map(|id| {
                let file = &files[id];
                file.metadata
                    .as_ref()
                    .map(|file_meta| (file.path.clone(), file_meta.block.clone()))
            })
            .chain(once((self.path.clone(), block)))
            .collect()
    }

    fn gen_header(&self, target_shell: ShellType) -> String {
        let mut header_template = if let Some(dynamic) = &self.options.header_path {
            fs::read_to_string(dynamic).unwrap_or_else(|_| {
                let msg = format!("Couldn't read the dynamic header file from path '{dynamic}'");
                Message::new_err_msg(msg).show();
                exit(1);
            })
        } else {
            include_str!("header.sh").trim_end().to_string()
        };

        if self.options.header_path.is_none() {
            let shebang = self
                .options
                .shebang
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| String::from("#!/usr/bin/env {{ shell }}"));
            header_template = format!("{}\n{}\n", shebang, header_template);
        }

        let shell_name = target_shell.family_name();
        header_template
            .replace("{{ version }}", get_version())
            .replace("{{ shell }}", shell_name)
    }

    fn gen_footer(&self) -> String {
        let footer_template = if let Some(dynamic) = &self.options.footer_path {
            fs::read_to_string(dynamic).unwrap_or_else(|_| {
                let msg = format!("Couldn't read the dynamic footer file from path '{dynamic}'");
                Message::new_err_msg(msg).show();
                exit(1);
            })
        } else {
            String::new()
        };

        footer_template.replace("{{ version }}", get_version())
    }

    fn gen_preamble(
        &self,
        sudo_used: bool,
        shellname_used: bool,
        shellversion_used: bool,
        target_shell: &ShellType,
    ) -> FragmentKind {
        let mut preamble = Vec::new();
        match target_shell {
            ShellType::BashModern | ShellType::BashLegacy => (),
            ShellType::Zsh => {
                preamble.push(RawFragment::new(r#"emulate ksh"#).to_frag());
                preamble.push(RawFragment::new(r#"setopt BSD_echo"#).to_frag());
                preamble.push(RawFragment::new(r#"__read_args='-A'"#).to_frag());
            }
            ShellType::Ksh => {
                preamble.push(RawFragment::new(r#"set -m"#).to_frag());
            }
        }
        if sudo_used {
            preamble.push(RawFragment::new(include_str!("preambles/sudo.sh").trim_end()).to_frag());
        }
        if shellname_used {
            preamble.push(
                RawFragment::new(include_str!("preambles/shellname.sh").trim_end())
                    .to_frag(),
            );
        }
        if shellversion_used {
            preamble.push(
                RawFragment::new(include_str!("preambles/shellversion.sh").trim_end())
                    .to_frag(),
            );
        }
        BlockFragment::new(preamble, false).to_frag()
    }

    pub fn translate(&self, block: Block, meta: ParserMetadata) -> Result<String, Message> {
        let ast_forest = self.get_sorted_ast_forest(block, &meta);
        let mut meta_translate = TranslateMetadata::new(meta, &self.options);
        let time = Instant::now();
        let mut body = BlockFragment::new(Vec::new(), false);
        for (_path, block) in ast_forest {
            body.append(block.translate(&mut meta_translate));
        }
        if self.options.debug_time {
            let pathname = self.path.to_string();
            println!(
                "[{}]\tin\t{}ms\t{pathname}",
                "Translate".magenta(),
                time.elapsed().as_millis()
            );
        }

        let mut result = BlockFragment::new(Vec::new(), false);
        // The preamble is generated after the body so that only builtins
        // actually present in the output pull in their preamble variables.
        result.append(self.gen_preamble(
            meta_translate.sudo_used,
            meta_translate.shellname_used,
            meta_translate.shellversion_used,
            &meta_translate.target.shell,
        ));
        result.append(body.to_frag());

        let mut result = result.to_frag();
        if !self.options.no_optimize {
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
            self.gen_header(meta_translate.target.shell),
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
            let SourcePath::File { canonical: dep_path, .. } = &path else {
                continue;
            };
            if !dep_path.starts_with(&base_dir) {
                continue;
            }
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
                let mut file = fs::File::create(path.clone()).unwrap();
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
        meta.with_target_shell(&self.options);

        // Perform type checking on the block
        if let Err(failure) = block.typecheck(&mut meta) {
            return Err(failure.unwrap_loud());
        }

        if self.options.debug_time {
            let pathname = self.path.to_string();
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

    pub fn execute(code: String, args: Vec<String>) -> Result<ExitStatus, std::io::Error> {
        Self::execute_with_target(code, args, None)
    }

    pub fn execute_with_target(
        mut code: String,
        args: Vec<String>,
        target: Option<ShellType>,
    ) -> Result<ExitStatus, std::io::Error> {
        if let Some(mut command) = Self::find_shell(target) {
            if !args.is_empty() {
                let args = args
                    .into_iter()
                    .map(|arg| format!("\"{}\"", escape_shell_arg(&arg)))
                    .collect::<Vec<String>>();
                code = format!("set -- {}\n{}", args.join(" "), code);
            }

            command.arg("-c").arg(code).spawn()?.wait()
        } else {
            let error = std::io::Error::new(ErrorKind::NotFound, "Failed to find shell");
            Err(error)
        }
    }

    pub fn generate_docs(&self, output: Option<String>, usage: bool) -> Result<(), Message> {
        if usage && self.path.stdlib_module_name().is_none() {
            return Err(Message::new_err_msg(format!(
                "The '--usage' flag is only supported for standard library files, but '{}' isn't one",
                self.path.to_string()
            )));
        }
        let tokens = self.tokenize()?;
        let (block, meta) = self.parse(tokens)?;
        let (block, mut meta) = self.typecheck(block, meta)?;
        meta.doc_usage = usage;
        self.document(block, meta, output);
        Ok(())
    }

    #[cfg(test)]
    pub fn test_eval(&mut self) -> Result<(String, ExitStatus), Message> {
        self.options.no_proc = vec!["*".into()];
        self.compile().map_or_else(Err, |(warnings, code)| {
            if let Some(mut command) = Self::find_shell(self.options.target) {
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
                Ok((warning_log + &output + &err_output, child.status))
            } else {
                let message = Message::new_err_msg("Failed to find Bash");
                Err(message)
            }
        })
    }
}
