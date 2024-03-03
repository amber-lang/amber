mod modules;
mod rules;
mod utils;
mod translate;
mod docs;
mod compiler;
pub mod cli;
use cli::cli_interface::CLI;

#[cfg(test)]
pub mod tests;

fn main() {
    let mut cli = CLI::new();
    cli.run();
}
