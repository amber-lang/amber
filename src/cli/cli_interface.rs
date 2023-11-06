use heraclitus_compiler::prelude::*;
use colored::Colorize;
use crate::compiler::AmberCompiler;
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
            ext: ".ab".to_string()
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
                    let code = format!("import * from \"std\"\n{code}");
                    match AmberCompiler::new(code, None).compile() {
                        Ok((messages, code)) => {
                            messages.iter().for_each(|m| m.show());
                            (!messages.is_empty()).then(|| self.render_dash());
                            AmberCompiler::execute(code, self.flags.get_args())
                        },
                        Err(err) => {
                            err.show();
                            std::process::exit(1);
                        }
                    }
                },
                None => {
                    Message::new_err_msg("No value passed after -e flag")
                        .comment("Write code to be evaluated after the -e flag")
                        .show();
                    std::process::exit(1);
                }
            }
        }
        // Parse input file
        else if self.args.len() >= 2 {
            let input = self.args[1].clone();
            match self.read_file(input.clone()) {
                Ok(code) => {
                    match AmberCompiler::new(code, Some(input)).compile() {
                        Ok((messages, code)) => {
                            messages.iter().for_each(|m| m.show());
                            // Save to the output file
                            if self.args.len() >= 3 {
                                Self::save_to_file(self.args[2].clone(), code)
                            }
                            // Execute the code
                            else {
                                (!messages.is_empty()).then(|| self.render_dash());
                                AmberCompiler::execute(code, self.flags.get_args());
                            }
                        },
                        Err(err) => {
                            err.show();
                            std::process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    Message::new_err_msg(err.to_string()).show();
                    std::process::exit(1);
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
    fn set_file_permission(_file: &fs::File, _output: String) {
        // We don't need to set permission on Windows
    }

    #[cfg(not(target_os = "windows"))]
    fn set_file_permission(file: &std::fs::File, path: String) {
        use std::os::unix::prelude::PermissionsExt;
        let mut perm = fs::metadata(path).unwrap().permissions();
        perm.set_mode(0o755);
        file.set_permissions(perm).unwrap();
    }

    #[inline]
    fn save_to_file(output_path: String, code: String) {
        match fs::File::create(output_path.clone()) {
            Ok(mut file) => {
                write!(file, "{}", code).unwrap();
                Self::set_file_permission(&file, output_path);
                
            },
            Err(err) => {
                Message::new_err_msg(err.to_string()).show();
                std::process::exit(1);
            }
        }
    }

    #[inline]
    #[allow(unused_must_use)]
    fn render_dash(&self) {
        let str = "%.s─".dimmed();
        Command::new("bash")
            .arg("-c")
            .arg(format!("printf {str} $(seq 1 $(tput cols))"))
            .spawn().unwrap().wait();
        println!();
    }

    #[inline]
    fn read_file(&self, path: impl AsRef<str>) -> io::Result<String> {
        fs::read_to_string(path.as_ref())
    }
}