use heraclitus_compiler::prelude::*;
use colored::Colorize;
use crate::modules::block;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::rules;
use super::flag_registry::FlagRegistry;
use std::env;
use std::process::Command;
use std::{io, io::prelude::*};
use std::fs;

pub struct CLI {
    args: Vec<String>,
    flags: FlagRegistry,
    name: String,
    exe_name: String,
    version: String,
    ext: String
}

impl Default for CLI {
    fn default() -> Self {
        Self::new()
    }
}

impl CLI {
    pub fn new() -> Self {
        CLI {
            args: vec![],
            flags: FlagRegistry::new(),
            name: "Amber".to_string(),
            exe_name: "amber".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ext: ".amber".to_string()
        }
    }

    pub fn run(&mut self) {
        self.flags.register("-e", true);
        self.flags.register("-h", false);
        self.flags.register("--help", false);
        self.args = self.flags.parse(env::args().collect());
        // Check all flags
        if self.flags.flag_triggered("-e") {
            match self.flags.get_flag("-e").unwrap().value.clone() {
                Some(code) => {
                    let translation = self.compile(code, None);
                    self.execute(translation);
                },
                None => {
                    Logger::new_err_msg("No value passed after -e flag")
                        .attach_comment("You can write code that has to be evaluated after the -e flag")
                        .show().exit();
                }
            }
        }
        // Parse input file
        else if self.args.len() >= 2 {
            let input = self.args[1].clone();
            match self.read_file(input.clone()) {
                Ok(code) => {
                    let code = self.compile(code, Some(input));
                    // Save to the output file
                    if self.args.len() >= 3 {
                        let output = self.args[2].clone();
                        match fs::File::create(output.clone()) {
                            Ok(mut file) => {
                                write!(file, "{}", code).unwrap();
                                self.set_file_permission(&file, output);
                                
                            },
                            Err(err) => {
                                Logger::new_err_msg(err.to_string()).show().exit();
                            }
                        }
                    }
                    // Evaluate
                    else {
                        self.execute(code);
                    }
                }
                Err(err) => {
                    Logger::new_err_msg(err.to_string()).show().exit();
                }
            }
        }
        else {
            println!("{}'s compiler", self.name);
            println!("Version {}\n", self.version);
            println!("USAGE:\t\t\t\tEXAMPLE:");
            println!("{}", "For evaluation:".dimmed());
            {
                let example = format!("{} foo{}", self.exe_name, self.ext).dimmed();
                println!("\t{} [INPUT]\t\t{}", self.exe_name, example);
            }
            {
                let example = format!("{} -e \"\\$echo Hello World\\$\"", self.exe_name).dimmed();
                println!("\t{} -e [EXPR]\t\t{}", self.exe_name, example);
            }
            println!("{}", "For compiling:".dimmed());
            {
                let example = format!("{} foo{} bar{}", self.exe_name, self.ext, self.ext).dimmed();
                println!("\t{} [INPUT] [OUTPUT]\t{}", self.exe_name, example);
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn set_file_permission(&self, _file: &fs::File, _output: String) {
        // We don't need to set permission on Windows
    }

    #[cfg(not(target_os = "windows"))]
    fn set_file_permission(&self, file: &std::fs::File, path: String) {
        use std::os::unix::prelude::PermissionsExt;
        let mut perm = fs::metadata(path).unwrap().permissions();
        perm.set_mode(0o755);
        file.set_permissions(perm).unwrap();
    }

    fn compile(&self, code: String, path: Option<String>) -> String {
        let rules = rules::get_rules();
        let mut cc = Compiler::new("Amber", rules);
        let mut block = block::Block::new();
        cc.load(code.clone());
        if let Ok(tokens) = cc.tokenize() {
            let mut meta = ParserMetadata::new(tokens, path, Some(code));
            if let Ok(()) = block.parse(&mut meta) {
                let mut meta = TranslateMetadata::new(&meta);
                return block.translate(&mut meta);
            }
            return "[parsing err]".to_string()
        }
        "[lexing err]".to_string()
    }

    fn execute(&self, code: String) {
        Command::new("/bin/bash").arg("-c").arg(code).spawn().unwrap().wait().unwrap();
    }

    pub fn test_eval(&self, code: impl AsRef<str>) -> String {
        let translation = self.compile(code.as_ref().to_string(), None);
        let child = Command::new("/bin/bash")
            .arg("-c").arg::<&str>(translation.as_ref())
            .output().unwrap();
        String::from_utf8_lossy(&child.stdout).to_string()
    }

    #[inline]
    fn read_file(&self, path: impl AsRef<str>) -> io::Result<String> {
        fs::read_to_string(path.as_ref())
    }
}