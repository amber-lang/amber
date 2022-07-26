mod modules;
mod rules;

use heraclitus_compiler::prelude::*;
use modules::block;

fn main() {
    let code = "1 + 2";
    let rules = rules::get_rules();
    let mut cc = Compiler::new("Amber", rules);
    let mut block = block::Block::new();
    cc.load(code);
    if let Ok(tokens) = cc.tokenize() {
        println!("{tokens:?}");
        let path = Some(format!("/path/to/file"));
        let mut meta = DefaultMetadata::new(tokens, path);
        if let Ok(()) = block.parse(&mut meta) {
            println!("{block:#?}");
        }
    }
}
