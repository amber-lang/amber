mod modules;
mod rules;
mod utils;

use heraclitus_compiler::prelude::*;
use modules::block;
use crate::utils::metadata::ParserMetadata;

fn main() {
    let code = vec![
        "let age = 12",
        "age = age + 12"
    ].join("\n");
    let rules = rules::get_rules();
    let mut cc = Compiler::new("Amber", rules);
    let mut block = block::Block::new();
    cc.load(code);
    if let Ok(tokens) = cc.tokenize() {
        let path = Some(format!("/path/to/file"));
        let mut meta = ParserMetadata::new(tokens, path);
        if let Ok(()) = block.parse_debug(&mut meta) {
            // println!("{block:#?}");
        }
    }
}
