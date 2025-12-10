use heraclitus_compiler::prelude::*;
use crate::modules::block::Block;
use crate::modules::prelude::*;
use crate::utils::metadata::ParserMetadata;

#[derive(Debug, Clone)]
pub struct Test {
    pub block: Block,
    pub token: Option<Token>,
    pub is_skipped: bool,
    pub name: String,
}

impl SyntaxModule<ParserMetadata> for Test {
    syntax_name!("Test");

    fn new() -> Self {
        Self {
            block: Block::new().with_no_indent(),
            token: None,
            is_skipped: false,
            name: String::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.token = meta.get_current_token();
        token(meta, "test")?;

        // Parse optional test name
        if let Some(token) = meta.get_current_token() {
            if token.word != "{" {
                if token.word.starts_with('"') {
                    self.name = token.word.trim_matches('"').to_string();
                    meta.set_index(meta.get_index() + 1);
                } else {
                    return error!(meta, Some(token.clone()), "Test name must be a string literal");
                }
            }
        }

        // Check for duplicate test names
        if meta.test_names.contains(&self.name) {
            let message = if self.name.is_empty() {
                "Multiple unnamed tests are not allowed in the same file".to_string()
            } else {
                format!("Test with name '{}' already exists", self.name)
            };
            return error!(meta, self.token.clone(), message);
        }
        meta.test_names.push(self.name.clone());

        // If this test is included in other file, skip it
        if !meta.context.trace.is_empty() {
            self.is_skipped = true;
        }
        context!({
            meta.context.is_main_ctx = true;
            meta.context.is_test_ctx = true;
            // Parse the block
            syntax(meta, &mut self.block)?;
            meta.context.is_main_ctx = false;
            meta.context.is_test_ctx = false;
            Ok(())
        }, |pos| {
            error_pos!(meta, pos, "Undefined syntax in test block")
        })
    }
}

impl TypeCheckModule for Test {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Test cannot be parsed inside of a block
        if !meta.is_global_scope() {
            return error!(meta, self.token.clone(), "Test must be in the global scope")
        }

        // Typecheck the test block content
        meta.with_push_scope(true, |meta| {
            meta.context.is_main_ctx = true;
            meta.context.is_test_ctx = true;
            meta.context.is_trust_ctx = true;
            // Typecheck the block
            self.block.typecheck(meta)?;
            meta.context.is_main_ctx = false;
            meta.context.is_test_ctx = false;
            meta.context.is_trust_ctx = false;
            Ok(())
        })
    }
}

impl TranslateModule for Test {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.is_skipped || !meta.test_mode {
            return FragmentKind::Empty;
        }

        if let Some(target_name) = &meta.test_name {
            if &self.name != target_name {
                return FragmentKind::Empty;
            }
        }

        self.block.translate(meta)
    }
}

impl DocumentationModule for Test {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
