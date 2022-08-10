mod modules;
mod rules;
mod utils;
mod translate;

use heraclitus_compiler::prelude::*;
use modules::block;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;

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
        println!("{:?}", tokens);
        let path = Some(format!("/path/to/file"));
        let mut meta = ParserMetadata::new(tokens, path);
        if let Ok(()) = block.parse_debug(&mut meta) {
            let mut meta = TranslateMetadata::new();
            let translation = block.translate(&mut meta);
            println!("{translation}");
            // println!("{block:#?}");
        }
    }
}
