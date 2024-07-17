mod compiler;
mod modules;
mod rules;
mod translate;
mod utils;
mod stdlib;

#[cfg(test)]
pub mod tests;

use crate::compiler::AmberCompiler;
use clap::Parser;
use colored::Colorize;
use heraclitus_compiler::prelude::*;
use std::error::Error;
use std::fs;
use std::io::{prelude::*, stdin};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(version, arg_required_else_help(true))]
struct Cli {
    #[arg(help = "'-' to read from stdin")]
    input: Option<PathBuf>,
    #[arg(help = "'-|--silence' to output to stdout, /dev/null to discard")]
    output: Option<PathBuf>,

    /// Code to evaluate
    #[arg(short, long)]
    eval: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Some(code) = cli.eval {
        let code = format!("import * from \"std\"\n{code}");
        match AmberCompiler::new(code, None).compile() {
            Ok((messages, code)) => {
                messages.iter().for_each(|m| m.show());
                (!messages.is_empty()).then(|| render_dash());
                let exit_status = AmberCompiler::execute(code, &vec![])?;
                std::process::exit(exit_status.code().unwrap_or(1));
            }
            Err(err) => {
                err.show();
                std::process::exit(1);
            }
        }
    } else if let Some(input) = cli.input {
        let input = String::from(input.to_string_lossy());
        let code = {
            if input == "-" {
                let mut buf = String::new();
                match stdin().read_to_string(&mut buf) {
                    Ok(_) => buf,
                    Err(err) => handle_err(err)
                }
            } else {
                match fs::read_to_string(&input) {
                    Ok(code) => code,
                    Err(err) => handle_err(err)
                }
            }
        };
        match AmberCompiler::new(code, Some(input)).compile() {
            Ok((messages, code)) => {
                messages.iter().for_each(|m| m.show());
                // Save to the output file
                if let Some(output) = cli.output {
                    let outs = String::from(output.to_string_lossy());
                    if outs == "--silent" {
                        return Ok(())
                    }
                    if outs == "-" {
                        print!("{code}");
                    }

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
                    let exit_status = AmberCompiler::execute(code, &vec![])?;
                    std::process::exit(exit_status.code().unwrap_or(1));
                }
            }
            Err(err) => {
                err.show();
                std::process::exit(1);
            }
        }
    }
    Ok(())
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

fn handle_err(err: std::io::Error) -> ! {
    Message::new_err_msg(err.to_string()).show();
    std::process::exit(1);
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
