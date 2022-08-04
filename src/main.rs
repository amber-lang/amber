mod modules;
mod rules;
mod parser;

use heraclitus_compiler::prelude::*;
use modules::block;
use parser::ParserMetadata;

fn main() {
    let code = "not true";
    let rules = rules::get_rules();
    let mut cc = Compiler::new("Amber", rules);
    let mut block = block::Block::new();
    cc.load(code);
    if let Ok(tokens) = cc.tokenize() {
        println!("{tokens:?}");
        let path = Some(format!("/path/to/file"));
        let mut meta = ParserMetadata::new(tokens, path);
        if let Ok(()) = block.parse_debug(&mut meta) {
            // println!("{block:#?}");
        }
    }
}
