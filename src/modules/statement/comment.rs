use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;

#[derive(Debug, Clone)]
pub struct Comment {
    pub value: String
}

impl SyntaxModule<ParserMetadata> for Comment {
    syntax_name!("Comment");

    fn new() -> Self {
        Comment {
            value: String::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let value = token_by(meta, |word| word.starts_with("//"))?;
        let comment_text = value.get(2..).unwrap_or("");
        
        // Handle the case where line continuation causes multiple lines to be in one token
        // We only want the first line as the comment content, ignoring subsequent lines
        // that were merged due to line continuation
        let first_line = comment_text.lines().next().unwrap_or("").trim();
        self.value = first_line.to_string();
        
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
