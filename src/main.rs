mod modules;
mod rules;
mod utils;
mod translate;
pub mod cli;
use cli::cli::CLI;

fn main() { 
    let mut cli = CLI::new();
    cli.run();
}
