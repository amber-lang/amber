mod compiler;
mod docs;
mod modules;
mod rules;
mod stdlib;
mod translate;
mod utils;
mod optimizer;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[cfg(test)]
pub mod tests;

use crate::compiler::{AmberCompiler, CompilerOptions};
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use colored::Colorize;
use heraclitus_compiler::prelude::*;
use std::error::Error;
use std::io::{prelude::*, stdin};
use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

#[derive(Parser, Clone, Debug)]
#[command(version(built_info::GIT_VERSION.unwrap_or(built_info::PKG_VERSION)), arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<CommandKind>,

    /// Input filename ('-' to read from stdin)
    input: Option<PathBuf>,

    /// Arguments passed to Amber script
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,

    /// Disable a postprocessor
    /// Available postprocessors: 'shfmt', 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wilcard match, like "*" or "s*mt"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,
}

#[derive(Subcommand, Clone, Debug)]
enum CommandKind {
    /// Execute Amber code fragment
    Eval(EvalCommand),
    /// Execute Amber script
    Run(RunCommand),
    /// Check Amber script for errors
    Check(CheckCommand),
    /// Compile Amber script to Bash
    Build(BuildCommand),
    /// Generate Amber script documentation
    Docs(DocsCommand),
    /// Generate Bash completion script
    Completion,
}

#[derive(Args, Clone, Debug)]
struct EvalCommand {
    /// Code to evaluate
    code: String,
}

#[derive(Args, Clone, Debug)]
struct RunCommand {
    /// Input filename ('-' to read from stdin)
    input: PathBuf,

    /// Arguments passed to Amber script
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,

    /// Disable a postprocessor
    /// Available postprocessors: 'shfmt', 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wilcard match, like "*" or "s*mt"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,
}

#[derive(Args, Clone, Debug)]
struct CheckCommand {
    /// Input filename ('-' to read from stdin)
    input: PathBuf,

    /// Disable a postprocessor
    /// Available postprocessors: 'shfmt', 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wilcard match, like "*" or "s*mt"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,
}

#[derive(Args, Clone, Debug)]
struct BuildCommand {
    /// Input filename ('-' to read from stdin)
    input: PathBuf,

    /// Output filename ('-' to output to stdout)
    output: Option<PathBuf>,

    /// Disable a postprocessor
    /// Available postprocessors: 'shfmt', 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wilcard match, like "*" or "s*mt"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,

    /// Minify the output file
    #[arg(long)]
    minify: bool,
}

#[derive(Args, Clone, Debug)]
struct DocsCommand {
    /// Input filename ('-' to read from stdin)
    input: PathBuf,

    /// Output directory (relative to input file, default 'docs', '-' to write to stdout)
    output: Option<PathBuf>,

    /// Show standard library usage in documentation
    #[arg(long)]
    usage: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            CommandKind::Eval(command) => {
                handle_eval(command)?;
            }
            CommandKind::Run(command) => {
                let options = CompilerOptions::from_args(&command.no_proc, false);
                let (code, messages) = compile_input(command.input, options);
                execute_output(code, command.args, messages)?;
            }
            CommandKind::Check(command) => {
                let options = CompilerOptions::from_args(&command.no_proc, false);
                compile_input(command.input, options);
            }
            CommandKind::Build(command) => {
                let output = create_output(&command);
                let options = CompilerOptions::from_args(&command.no_proc, command.minify);
                let (code, _) = compile_input(command.input, options);
                write_output(output, code);
            }
            CommandKind::Docs(command) => {
                handle_docs(command)?;
            }
            CommandKind::Completion => {
                handle_completion();
            }
        }
    } else if let Some(input) = cli.input {
        let options = CompilerOptions::from_args(&cli.no_proc, false);
        let (code, messages) = compile_input(input, options);
        execute_output(code, cli.args, messages)?;
    }
    Ok(())
}

fn create_output(command: &BuildCommand) -> PathBuf {
    if let Some(output) = &command.output {
        output.clone()
    } else if command.input.as_os_str() == "-" {
        command.input.clone()
    } else {
        command.input.with_extension("sh")
    }
}

fn compile_input(input: PathBuf, options: CompilerOptions) -> (String, bool) {
    let input = input.to_string_lossy().to_string();
    let amber_code = if input == "-" {
        let mut code = String::new();
        match stdin().read_to_string(&mut code) {
            Ok(_) => code,
            Err(err) => handle_err(err),
        }
    } else {
        match fs::read_to_string(&input) {
            Ok(code) => code,
            Err(err) => handle_err(err),
        }
    };
    let compiler = AmberCompiler::new(amber_code, Some(input), options);
    let (messages, bash_code) = match compiler.compile() {
        Ok(result) => result,
        Err(err) => {
            err.show();
            std::process::exit(1);
        }
    };
    messages.iter().for_each(|m| m.show());
    (bash_code, !messages.is_empty())
}

fn execute_output(code: String, args: Vec<String>, messages: bool) -> Result<(), Box<dyn Error>> {
    if messages {
        render_dash();
    }
    let exit_status = AmberCompiler::execute(code, args)?;
    std::process::exit(exit_status.code().unwrap_or(1));
}

fn write_output(output: PathBuf, code: String) {
    let output = output.to_string_lossy().to_string();
    if output == "-" {
        print!("{code}");
    } else {
        match fs::File::create(&output) {
            Ok(mut file) => {
                write!(file, "{code}").unwrap();
                set_file_permission(&file, output);
            }
            Err(err) => {
                Message::new_err_msg(err.to_string()).show();
                std::process::exit(1);
            }
        }
    }
}

fn handle_eval(command: EvalCommand) -> Result<(), Box<dyn Error>> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(command.code, None, options);
    match compiler.compile() {
        Ok((messages, code)) => {
            messages.iter().for_each(|m| m.show());
            (!messages.is_empty()).then(render_dash);
            let exit_status = AmberCompiler::execute(code, vec![])?;
            std::process::exit(exit_status.code().unwrap_or(1));
        }
        Err(err) => {
            err.show();
            std::process::exit(1);
        }
    }
}

fn handle_docs(command: DocsCommand) -> Result<(), Box<dyn Error>> {
    let input = command.input.to_string_lossy().to_string();
    let code = match fs::read_to_string(&input) {
        Ok(code) => code,
        Err(err) => {
            Message::new_err_msg(err.to_string()).show();
            std::process::exit(1);
        }
    };
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code, Some(input), options);
    let output = command.output.unwrap_or_else(|| PathBuf::from("docs"));
    let output = output.to_string_lossy().to_string();
    let output = if output != "-" { Some(output) } else { None };
    match compiler.generate_docs(output, command.usage) {
        Ok(_) => Ok(()),
        Err(err) => {
            err.show();
            std::process::exit(1);
        }
    }
}

fn handle_completion() {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    clap_complete::generate(Shell::Bash, &mut command, name, &mut io::stdout());
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
