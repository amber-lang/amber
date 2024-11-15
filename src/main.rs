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
use compiler::file_source::{FileMeta, FileSource};
use heraclitus_compiler::prelude::*;
use std::error::Error;
use std::fs;
use std::io::{prelude::*, stdin};
use std::path::{Path, PathBuf};
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
    /// (OUTPUT is dir instead, default: `docs/` if missing it will generate the folder)
    #[arg(long)]
    docs: bool,
  
    /// Disable a postprocessor
    /// Available postprocessors: shfmt, bshchk
    /// To select multiple, pass this argument multiple times with different values.
    /// This argument also supports a wilcard match, like "*" or "s*mt"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,

    /// Minify the resulting code
    #[arg(long)]
    minify: bool,

    #[arg(long, short)]
    no_cache: bool
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
    let mut file_source = FileSource::File;

    let input = if let Some(input) = cli.input.clone() {
        String::from(input.to_string_lossy().trim())
    } else {
        return Ok(());
    };

    let code = if input == "-" {
        file_source = FileSource::Stream;
        let mut buf = String::new();
        match stdin().read_to_string(&mut buf) {
            Ok(_) => buf,
            Err(err) => handle_err(err),
        }
    } else {
        match fs::read_to_string(&input) {
            Ok(code) => code,
            Err(err) => handle_err(err),
        }
    };

    let compiler = AmberCompiler::new(
        code,
        Some(input),
        cli.clone(),
        FileMeta {
            is_import: false,
            source: file_source
        }
    );
    let (messages, code) = match compiler.compile() {
        Ok(result) => result,
        Err(err) => {
            err.show();
            std::process::exit(1);
        }
    };
    messages.iter().for_each(|m| m.show());
    // Save to the output file
    let output = if let Some(output) = cli.output {
        String::from(output.to_string_lossy())
    } else {
        // Execute the code
        (!messages.is_empty()).then(render_dash);
        let exit_status = AmberCompiler::execute(code, &[])?;
        std::process::exit(exit_status.code().unwrap_or(1));
    };

    if output == "--silent" {
        return Ok(());
    }

    if output == "-" {
        print!("{code}");
        return Ok(());
    }

    match fs::File::create(&output) {
        Ok(mut file) => {
            write!(file, "{}", code).unwrap();
            set_file_permission(&file, output);
        }
        Err(err) => {
            Message::new_err_msg(err.to_string()).show();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn handle_eval(code: String, cli: Cli) -> Result<(), Box<dyn Error>> {
    let compiler = AmberCompiler::new(
        code,
        None,
        cli,
        FileMeta {
            is_import: false,
            source: FileSource::Stream
        }
    );
    match compiler.compile() {
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
    let input = if let Some(ref input) = cli.input {
        let path = Path::new(input);
        if !path.exists() {
            Message::new_err_msg(format!(
                "Amber file doesn't exist: `{}`.", input.to_string_lossy()
            ))
            .show();
            std::process::exit(1);
        }
        String::from(input.to_string_lossy())
    } else {
        Message::new_err_msg(
            "You need to provide a path to an entry file to generate the documentation",
        )
        .show();
        std::process::exit(1);
    };

    let output = {
        let out = cli.output.clone().unwrap_or_else(|| PathBuf::from("docs"));
        String::from(out.to_string_lossy())
    };

    let code: String = match fs::read_to_string(&input) {
        Ok(code) => code,
        Err(err) => {
            Message::new_err_msg(err.to_string()).show();
            std::process::exit(1);
        }
    };

    let compiler = AmberCompiler::new(
        code,
        Some(input),
        cli,
        FileMeta {
            is_import: false,
            source: FileSource::File
        }
    );
    match compiler.generate_docs(output) {
        Ok(_) => Ok(()),
        Err(err) => {
            err.show();
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
fn set_file_permission(_file: &fs::File, _output: String) {
    // We don't need to set permission on Windows
}

#[cfg(not(windows))]
fn set_file_permission(file: &fs::File, path: String) {
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
