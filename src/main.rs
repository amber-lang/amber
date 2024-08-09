mod compiler;
mod docs;
mod modules;
mod rules;
mod stdlib;
mod translate;
mod utils;

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

#[derive(Parser, Clone, Debug)]
#[command(version, arg_required_else_help(true))]
#[derive(Default)]
pub struct Cli {
    #[arg(help = "'-' to read from stdin")]
    input: Option<PathBuf>,
    #[arg(help = "'-' to output to stdout, '--silent' to discard")]
    output: Option<PathBuf>,

    /// Code to evaluate
    #[arg(short, long)]
    eval: Option<String>,

    /// Generate docs
    /// (OUTPUT is dir instead, default: `docs/`)
    #[arg(long)]
    docs: bool,

    /// Don't format the output file
    #[arg(long)]
    disable_format: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    if cli.docs {
        handle_docs(cli)?;
    } else if let Some(ref code) = cli.eval {
        handle_eval(code.to_string(), cli)?;
    } else {
        handle_compile(cli)?;
    }
    Ok(())
}

fn handle_compile(cli: Cli) -> Result<(), Box<dyn Error>> {
    let valid_input: String;
    if let Some(input) = cli.input.clone() {
        valid_input = String::from(input.to_string_lossy().trim());
    } else {
        return Ok(());
    }

    let code = {
        if valid_input == "-" {
            let mut buf = String::new();
            match stdin().read_to_string(&mut buf) {
                Ok(_) => buf,
                Err(err) => handle_err(err),
            }
        } else {
            match fs::read_to_string(&valid_input) {
                Ok(code) => code,
                Err(err) => handle_err(err),
            }
        }
    };

    let valid_messages;
    let valid_code: String;
    match AmberCompiler::new(code, Some(valid_input), cli.clone()).compile() {
        Ok((messages, code)) => {
            valid_messages = messages;
            valid_code = code;
        }
        Err(err) => {
            err.show();
            std::process::exit(1);
        }
    }
    valid_messages.iter().for_each(|m| m.show());
    // Save to the output file
    let valid_output: String;
    if let Some(output) = cli.output {
        valid_output = String::from(output.to_string_lossy());
    } else {
        // Execute the code
        (!valid_messages.is_empty()).then(render_dash);
        let exit_status = AmberCompiler::execute(valid_code, &[])?;
        std::process::exit(exit_status.code().unwrap_or(1));
    }

    if valid_output == "--silent" {
        return Ok(());
    }

    if valid_output == "-" {
        print!("{valid_code}");
        return Ok(());
    }

    match fs::File::create(&valid_output) {
        Ok(mut file) => {
            write!(file, "{}", valid_code).unwrap();
            set_file_permission(&file, valid_output);
        }
        Err(err) => {
            Message::new_err_msg(err.to_string()).show();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn handle_eval(code: String, cli: Cli) -> Result<(), Box<dyn Error>> {
    match AmberCompiler::new(code, None, cli).compile() {
        Ok((messages, code)) => {
            messages.iter().for_each(|m| m.show());
            (!messages.is_empty()).then(render_dash);
            let exit_status = AmberCompiler::execute(code, &[])?;
            std::process::exit(exit_status.code().unwrap_or(1));
        }
        Err(err) => {
            err.show();
            std::process::exit(1);
        }
    }
}

fn handle_docs(cli: Cli) -> Result<(), Box<dyn Error>> {
    let valid_input: String;
    if let Some(ref input) = cli.input {
        valid_input = String::from(input.to_string_lossy());
    } else {
        Message::new_err_msg(
            "You need to provide a path to an entry file to generate the documentation",
        )
        .show();
        std::process::exit(1);
    }

    let output = {
        let out = cli.output.clone().unwrap_or_else(|| PathBuf::from("docs"));
        String::from(out.to_string_lossy())
    };

    
    let valid_code: String = match fs::read_to_string(&valid_input) {
        Ok(code) => code,
        Err(err) => {
            Message::new_err_msg(err.to_string()).show();
            std::process::exit(1);
        }
    };

    match AmberCompiler::new(valid_code, Some(valid_input), cli).generate_docs(output) {
        Ok(_) => Ok(()),
        Err(err) => {
            err.show();
            std::process::exit(1);
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
