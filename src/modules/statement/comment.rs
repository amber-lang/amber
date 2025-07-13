use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::translate::fragments::list::ListFragment;

#[derive(Debug, Clone)]
pub struct Comment {
    pub value: String,
    // Store additional comment lines that were merged due to line continuation
    pub additional_lines: Vec<String>,
}

impl SyntaxModule<ParserMetadata> for Comment {
    syntax_name!("Comment");

    fn new() -> Self {
        Comment {
            value: String::new(),
            additional_lines: Vec::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let value = token_by(meta, |word| word.starts_with("//"))?;
        let comment_text = value.get(2..).unwrap_or("");
        
        // Handle the case where line continuation causes multiple lines to be in one token
        let lines: Vec<&str> = comment_text.lines().collect();
        
        // First line goes to the main value
        self.value = lines.first().unwrap_or(&"").trim().to_string();
        
        // Additional lines that start with // are stored separately
        for line in lines.iter().skip(1) {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                let comment_content = trimmed.get(2..).unwrap_or("").trim();
                self.additional_lines.push(comment_content.to_string());
            }
        }
        
        Ok(())
    }
}

impl TranslateModule for Comment {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> FragmentKind {
        if meta.minify {
            FragmentKind::Empty
        } else {
            // If we have additional comment lines, return them all as a list
            if !self.additional_lines.is_empty() {
                let mut fragments = Vec::new();
                
                // Add the main comment
                fragments.push(CommentFragment::new(&self.value).to_frag());
                
                // Add additional comment lines
                for line in &self.additional_lines {
                    fragments.push(CommentFragment::new(line).to_frag());
                }
                
                ListFragment::new(fragments).to_frag()
            } else {
                CommentFragment::new(&self.value).to_frag()
            }
        }
    }
}

impl DocumentationModule for Comment {
    fn document(&self, _meta: &ParserMetadata) -> String {
        self.value.clone() + "\n\n"
    }
}
