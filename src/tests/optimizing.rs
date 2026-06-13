/// Tests for the Amber optimizer (unit + integration/snapshot tests).
use crate::compiler::{AmberCompiler, CompilerOptions};
use crate::modules::prelude::{FragmentRenderable, TranslateModule};
use crate::optimizer::optimize_fragments;
use crate::translate::fragments::fragment::FragmentKind;
use crate::translate::fragments::raw::RawFragment;
use crate::utils::TranslateMetadata;
use insta::assert_snapshot;
use std::fs;
use std::path::Path;
use test_generator::test_resources;

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[test]
fn test_optimize_fragments_empty() {
    let mut frag = FragmentKind::Empty;
    optimize_fragments(&mut frag);
}

#[test]
fn test_optimize_fragments_raw_fragment() {
    let raw = RawFragment::new("test code");
    let mut frag = FragmentKind::Raw(raw);
    optimize_fragments(&mut frag);
}

#[test]
fn test_optimize_fragments_multiple_passes() {
    let mut frag = FragmentKind::Empty;
    optimize_fragments(&mut frag);
    optimize_fragments(&mut frag);
    optimize_fragments(&mut frag);
}

#[test]
fn test_optimize_fragments_block_fragment() {
    let frag = FragmentKind::Block(crate::translate::fragments::block::BlockFragment {
        statements: vec![],
        increase_indent: false,
        needs_noop: false,
        is_conditional: false,
    });
    let mut frag = frag;
    optimize_fragments(&mut frag);
}

// ---------------------------------------------------------------------------
// Integration / snapshot tests
// ---------------------------------------------------------------------------

pub fn translate_and_optimize_amber_code<T: Into<String>>(code: T) -> Option<String> {
    let options = CompilerOptions::default();
    let compiler = AmberCompiler::new(code.into(), None, options);
    let tokens = compiler.tokenize().ok()?;
    let (ast, meta) = compiler.parse(tokens).ok()?;
    let (ast, meta) = compiler.typecheck(ast, meta).ok()?;
    let mut translate_meta = TranslateMetadata::new(meta, &compiler.options);
    let mut translation = ast.translate(&mut translate_meta);
    optimize_fragments(&mut translation);
    Some(translation.to_string(&mut translate_meta))
}

/// Autoload the Amber test files in optimizing
#[test_resources("src/tests/optimizing/*.ab")]
fn test_translation(input: &str) {
    let code =
        fs::read_to_string(input).unwrap_or_else(|_| panic!("Failed to open {input} test file"));
    let output = translate_and_optimize_amber_code(code).expect("Couldn't translate Amber code");
    let filename = Path::new(input)
        .file_name()
        .expect("Provided directory")
        .to_str()
        .expect("Cannot translate to string");
    let filename = format!("{filename}__{}", AmberCompiler::resolve_target_shell(None));
    assert_snapshot!(filename, output);
}
