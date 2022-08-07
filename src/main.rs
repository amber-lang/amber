mod modules;
mod rules;
mod utils;

use heraclitus_compiler::prelude::*;
use modules::block;
use crate::utils::metadata::ParserMetadata;

fn main() {
    let code = vec![
        "'interpolate {'this {1} this'} and {'that'} :)'"
    ].join("\n");
    let rules = rules::get_rules();
    let mut cc = Compiler::new("Amber", rules);
    let mut block = block::Block::new();
    cc.load(code);
    if let Ok(tokens) = cc.tokenize() {
        println!("{:?}", tokens);
        let path = Some(format!("/path/to/file"));
        let mut meta = ParserMetadata::new(tokens, path);
        if let Ok(()) = block.parse_debug(&mut meta) {
            // println!("{block:#?}");
        }
    }
}
