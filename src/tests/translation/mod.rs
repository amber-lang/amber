use crate::{compiler::{AmberCompiler, CompilerOptions}, modules::prelude::{FragmentKind, TranslateModule}, utils::TranslateMetadata};

/// Tests that check shell AST that Amber generates from the source code.

pub mod number_binop;
pub mod number_shorthand;

pub fn translate_amber_code<T: Into<String>>(code: T) -> Option<FragmentKind> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.into(), None, options);
    let tokens = compiler.tokenize().ok()?;
    let (ast, meta) = compiler.parse(tokens).ok()?;
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);
    Some(ast.translate(&mut translate_meta))
}
