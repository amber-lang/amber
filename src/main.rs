#![feature(is_some_with)]

mod modules;
mod rules;
mod utils;
mod translate;
mod cli;

fn main() { 
    let mut cli = cli::CLI::new();
    cli.run();
}
