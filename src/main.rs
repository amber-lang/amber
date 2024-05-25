mod compiler;
mod modules;
mod rules;
mod translate;
mod utils;

#[cfg(test)]
pub mod tests;

use crate::compiler::AmberCompiler;
use clap::Parser;
use colored::Colorize;
use heraclitus_compiler::prelude::*;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(version, arg_required_else_help(true))]
struct Cli {
    input: Option<PathBuf>,
    output: Option<PathBuf>,

    /// Code to evaluate
    #[arg(short, long)]
    eval: Option<String>,

    /// Disable Runtime Dependency Checker (not recommended)
    #[arg(long)]
    disable_rdc: bool
}

fn main() {
    let cli = Cli::parse();

    if let Some(code) = cli.eval {
        let code = format!("import * from \"std\"\n{code}");
        match AmberCompiler::new(code, None, cli.disable_rdc).compile() {
            Ok((messages, code)) => {
                messages.iter().for_each(|m| m.show());
                (!messages.is_empty()).then(|| render_dash());
                AmberCompiler::execute(code, &vec![])
            }
            Err(err) => {
                err.show();
                std::process::exit(1);
            }
        }
    } else if let Some(input) = cli.input {
        let input = String::from(input.to_string_lossy());

        match fs::read_to_string(&input) {
            Ok(code) => {
                match AmberCompiler::new(code, Some(input), cli.disable_rdc).compile() {
                    Ok((messages, code)) => {
                        messages.iter().for_each(|m| m.show());
                        // Save to the output file
                        if let Some(output) = cli.output {
                            match fs::File::create(&output) {
                                Ok(mut file) => {
                                    write!(file, "{}", code).unwrap();
                                    set_file_permission(
                                        &file,
                                        String::from(output.to_string_lossy()),
                                    );
                                }
                                Err(err) => {
                                    Message::new_err_msg(err.to_string()).show();
                                    std::process::exit(1);
                                }
                            }
                        }
                        // Execute the code
                        else {
                            (!messages.is_empty()).then(|| render_dash());
                            AmberCompiler::execute(code, &vec![]);
                        }
                    }
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
    } else {
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
#[allow(unused_must_use)]
fn render_dash() {
    let str = "%.s─".dimmed();
    Command::new("bash")
        .arg("-c")
        .arg(format!("printf {str} $(seq 1 $(tput cols))"))
        .spawn()
        .unwrap()
        .wait();
    println!();
}
