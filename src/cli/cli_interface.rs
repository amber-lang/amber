use heraclitus_compiler::prelude::*;
use colored::{Colorize, CustomColor};
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
        self.flags.register("docs", true);
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
        // Generate documentation
        else if self.flags.flag_triggered("docs") {
            let input = match self.flags.get_flag("docs").unwrap().value.clone() {
                Some(input) => input,
                None => {
                    Message::new_err_msg("No input file provided")
                        .comment("Write the input file after the docs command")
                        .show();
                    std::process::exit(1);
                }
            };
            match self.read_file(input.clone()) {
                Ok(code) => {
                    match AmberCompiler::new(code, Some(input)).generate_docs() {
                        Ok(()) => {
                            Message::new_info_msg("Documentation generated successfully").show();
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
            println!("{}", " USAGE ".custom_color(CustomColor{r: 255, g: 255, b: 255}).on_color("#000"));
            println!("{}", "For evaluation:".bold());
            {
                println!("  {} [INPUT]", self.exe_name);
                println!("{}", format!("  {} foo{}\n", self.exe_name, self.ext).dimmed());
            }
            {
                println!("  {} -e [EXPR]", self.exe_name);
                println!("{}", format!("  {} -e \"\\$echo Hello World\\$\"\n", self.exe_name).dimmed());
            }
            println!("{}", "For compiling:".bold());
            {
                println!("  {} [INPUT] [OUTPUT]", self.exe_name);
                println!("{}", format!("  {} foo{} bar{}\n", self.exe_name, self.ext, self.ext).dimmed());
            }
            println!("{}", "For documentation generation:".bold());
            {
                println!("  {} docs [INPUT]", self.exe_name);
                println!("{}", format!("  {} docs foo{}\n", self.exe_name, self.ext).dimmed());
            }
            {
                println!("  {} docs [INPUT] [OUTPUT DIR]", self.exe_name);
                println!("{}", format!("  {} docs foo{} docs/", self.exe_name, self.ext).dimmed());
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
