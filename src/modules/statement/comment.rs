use crate::modules::prelude::*;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Comment {
    pub value: String,
}

impl SyntaxModule<ParserMetadata> for Comment {
    syntax_name!("Comment");

    fn new() -> Self {
        Comment {
            value: String::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let value = token_by(meta, |word| word.starts_with("//"))?;
        self.value = value.get(2..).unwrap_or("").trim().to_string();
        Ok(())
    }
}

impl TypeCheckModule for Comment {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for Comment {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> FragmentKind {
        if meta.minify {
            FragmentKind::Empty
        } else {
            CommentFragment::new(&self.value).to_frag()
        }
    }
}

impl DocumentationModule for Comment {
    fn document(&self, _meta: &ParserMetadata) -> String {
        self.value.clone() + "\n\n"
    }
}
