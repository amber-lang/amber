mod modules;
mod rules;
mod parser;

use heraclitus_compiler::prelude::*;
use modules::block;
use parser::ParserMetadata;

fn main() {
    let code = "true and 12 * (1 - 2 or false) + 3 / 12 - 1";
    let rules = rules::get_rules();
    let mut cc = Compiler::new("Amber", rules);
    let mut block = block::Block::new();
    cc.load(code);
    if let Ok(tokens) = cc.tokenize() {
        println!("{tokens:?}");
        let path = Some(format!("/path/to/file"));
        let mut meta = ParserMetadata::new(tokens, path);
        if let Ok(()) = block.parse(&mut meta) {
            println!("{block:#?}");
        }
    }
}
