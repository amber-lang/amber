mod compiler;
mod docs;
mod modules;
mod optimizer;
mod rules;
mod stdlib;
mod translate;
mod utils;

mod testing;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[cfg(test)]
pub mod tests;

use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::utils::ShellType;
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use colored::Colorize;
use heraclitus_compiler::prelude::*;
use similar_string::find_best_similarity;
use std::error::Error;
use std::io::{prelude::*, stdin};
use std::path::{Path, PathBuf};
use std::{fs, io};

fn get_version() -> &'static str {
    built_info::GIT_VERSION.unwrap_or(built_info::PKG_VERSION)
}

#[derive(Parser, Clone, Debug)]
#[command(version(get_version()))]
struct Cli {
    /// Input filename ('-' to read from stdin)
    input: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<CommandKind>,

    /// Arguments passed to Amber script
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,

    /// Disable a postprocessor
    /// Available postprocessors: 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wildcard match, like "*" or "b*chk"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,

    /// Code generation target shell
    #[arg(long)]
    target: Option<ShellType>,
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
    /// Run Amber tests
    Test(TestCommand),
}

#[derive(Args, Clone, Debug)]
struct EvalCommand {
    /// Code to evaluate
    code: String,

    /// Code generation target shell
    #[arg(long)]
    target: Option<ShellType>,
}

#[derive(Args, Clone, Debug)]
struct RunCommand {
    /// Input filename ('-' to read from stdin)
    input: PathBuf,

    /// Arguments passed to Amber script
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,

    /// Disable a postprocessor
    /// Available postprocessors: 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wildcard match, like "*" or "b*chk"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,

    /// Code generation target shell
    #[arg(long)]
    target: Option<ShellType>,
}

#[derive(Args, Clone, Debug)]
struct CheckCommand {
    /// Input filename ('-' to read from stdin)
    input: PathBuf,

    /// Disable a postprocessor
    /// Available postprocessors: 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wildcard match, like "*" or "b*chk"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,

    /// Code generation target shell
    #[arg(long)]
    target: Option<ShellType>,
}

#[derive(Args, Clone, Debug)]
struct BuildCommand {
    /// Input filename ('-' to read from stdin)
    input: PathBuf,

    /// Output filename ('-' to output to stdout)
    output: Option<PathBuf>,

    /// Disable a postprocessor
    /// Available postprocessors: 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wildcard match, like "*" or "b*chk"
    #[arg(long, verbatim_doc_comment)]
    no_proc: Vec<String>,

    /// Minify the output file
    #[arg(long)]
    minify: bool,

    /// Code generation target shell
    #[arg(long)]
    target: Option<ShellType>,
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

#[derive(Args, Clone, Debug)]
pub struct TestCommand {
    /// Input filename or directory ('-' to read from stdin)
    #[arg(default_value = ".")]
    pub input: PathBuf,

    /// Arguments passed to Amber script
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,

    /// Disable a postprocessor
    /// Available postprocessors: 'bshchk'
    /// To select multiple, pass multiple times with different values
    /// Argument also supports a wildcard match, like "*" or "b*chk"
    #[arg(long, verbatim_doc_comment)]
    pub no_proc: Vec<String>,

    /// Code generation target shell
    #[arg(long)]
    pub target: Option<ShellType>,
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

#[cfg(windows)]
fn set_file_permission(_file: &fs::File, _output: String) {}

#[cfg(not(windows))]
pub(crate) fn set_file_permission(file: &fs::File, path: String) {
    use std::os::unix::prelude::PermissionsExt;
    let mut perm = fs::metadata(path).unwrap().permissions();
    perm.set_mode(0o755);
    file.set_permissions(perm).unwrap();
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

fn handle_err(err: std::io::Error) -> ! {
    Message::new_err_msg(err.to_string()).show();
    std::process::exit(1);
}

#[inline]
#[allow(unused_must_use)]
pub fn render_dash() {
    let str = "%.s─".dimmed();
    AmberCompiler::execute(format!("printf {str} $(seq 1 $(tput cols))"), vec![]);
    println!();
}

fn execute_output(
    code: String,
    args: Vec<String>,
    messages: bool,
    target: Option<ShellType>,
) -> Result<i32, Box<dyn Error>> {
    if messages {
        render_dash();
    }
    let exit_status = AmberCompiler::execute_with_target(code, args, target)?;
    Ok(exit_status.code().unwrap_or(1))
}

fn resolve_command_target(
    command_target: Option<ShellType>,
    cli_target: Option<ShellType>,
) -> Option<ShellType> {
    command_target.or(cli_target)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn write_output(output: PathBuf, code: String) {
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

#[cfg(test)]
fn handle_eval(command: EvalCommand) -> Result<i32, Box<dyn Error>> {
    handle_eval_with_target(command, None)
}

fn handle_eval_with_target(
    command: EvalCommand,
    cli_target: Option<ShellType>,
) -> Result<i32, Box<dyn Error>> {
    let target = resolve_command_target(command.target, cli_target);
    let options = CompilerOptions::default()
        .with_target(target)
        .with_env_vars();
    let compiler = AmberCompiler::new(command.code, None, options);
    match compiler.compile() {
        Ok((messages, code)) => {
            messages.iter().for_each(|m| m.show());
            (!messages.is_empty()).then(render_dash);
            let exit_status = AmberCompiler::execute_with_target(code, vec![], target)?;
            Ok(exit_status.code().unwrap_or(1))
        }
        Err(err) => {
            err.show();
            Ok(1)
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
    let options = CompilerOptions::default().with_env_vars();
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

pub(crate) fn handle_completion() {
    handle_completion_with_output(&mut io::stdout());
}

pub(crate) fn handle_completion_with_output(output: &mut dyn std::io::Write) {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    clap_complete::generate(Shell::Bash, &mut command, name, output);
}

fn handle_bad_command_name(
    input: &Path,
    no_proc: &[String],
    args: Vec<String>,
    target: Option<ShellType>,
) -> Result<i32, Box<dyn Error>> {
    let input_str = input.to_string_lossy();

    let cli_cmd = Cli::command();
    let subcommands: Vec<&str> = cli_cmd.get_subcommands().map(|s| s.get_name()).collect();

    if !input.exists() && input_str != "-" {
        if input_str.starts_with('-') || input_str == "help" {
            eprintln!("Error: Unknown command or invalid option: {}", input_str);
            Cli::command().print_help().unwrap();
            println!();
            std::process::exit(1);
        }

        if let Some((match_name, score)) = find_best_similarity(&input_str, &subcommands) {
            if score >= 0.75 {
                eprintln!("Error: Unknown command: {}", input_str);
                eprintln!("Did you mean '{}'?", match_name);
                Cli::command().print_help().unwrap();
                println!();
                std::process::exit(1);
            }
        }

        eprintln!("Error: File not found: {}", input_str);
        Cli::command().print_help().unwrap();
        println!();
        std::process::exit(1);
    }

    let options = CompilerOptions::from_args(no_proc, false, false, None)
        .with_target(target)
        .with_env_vars();
    let (code, messages) = compile_input(input.to_path_buf(), options);
    execute_output(code, args, messages, target)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => err.exit(),
    };

    if let Some(ref input) = cli.input {
        std::process::exit(handle_bad_command_name(
            input,
            &cli.no_proc,
            cli.args,
            cli.target,
        )?);
    }

    let Some(command) = cli.command else {
        Cli::command().print_help().unwrap();
        println!();
        std::process::exit(0);
    };

    let exit_code = match command {
        CommandKind::Eval(command) => handle_eval_with_target(command, cli.target)?,
        CommandKind::Run(command) => {
            let target = resolve_command_target(command.target, cli.target);
            let options = CompilerOptions::from_args(&command.no_proc, false, false, None)
                .with_target(target)
                .with_env_vars();
            let (code, messages) = compile_input(command.input, options);
            execute_output(code, command.args, messages, target)?
        }
        CommandKind::Check(command) => {
            let target = resolve_command_target(command.target, cli.target);
            let options = CompilerOptions::from_args(&command.no_proc, false, false, None)
                .with_target(target)
                .with_env_vars();
            compile_input(command.input, options);
            0
        }
        CommandKind::Build(command) => {
            let target = resolve_command_target(command.target, cli.target);
            let output = create_output(&command);
            let options = CompilerOptions::from_args(&command.no_proc, command.minify, false, None)
                .with_target(target)
                .with_env_vars();
            let (code, _) = compile_input(command.input, options);
            write_output(output, code);
            0
        }
        CommandKind::Docs(command) => {
            handle_docs(command)?;
            0
        }
        CommandKind::Completion => {
            handle_completion();
            0
        }
        CommandKind::Test(mut command) => {
            command.target = resolve_command_target(command.target, cli.target);
            testing::handle_test(command)?
        }
    };

    std::process::exit(exit_code);
}
