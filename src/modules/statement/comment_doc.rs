use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use crate::docs::module::DocumentationModule;
use crate::utils::metadata::ParserMetadata;
use crate::translate::module::TranslateModule;

#[derive(Debug, Clone)]
pub struct CommentDoc {
    pub value: String
}

impl SyntaxModule<ParserMetadata> for CommentDoc {
    syntax_name!("Comment Doc");

    fn new() -> Self {
        CommentDoc {
            value: String::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        match meta.get_current_token() {
            Some(token) => {
                let mut col = token.pos.0;
                if token.word.starts_with("///") {
                    self.value = token.word[3..].trim().to_string();
                    meta.increment_index();
                    while let Some(token) = meta.get_current_token() {
                        let is_token_underneeth = token.pos.0 == col + 1;
                        let last_char = self.value.chars().last().unwrap_or('\n');
                        // If the token is a newline, we add a newline to the comment
                        if token.word.starts_with('\n') {
                            self.value.push('\n');
                            meta.increment_index();
                            continue;
                        }
                        if token.word.starts_with("///") && is_token_underneeth {
                            // Update the column of the last comment
                            col = token.pos.0;
                            meta.increment_index();
                            // If the comment signifies a paragrah break, we add two newlines
                            if token.word[3..].trim().is_empty() {
                                if last_char == '\n' {
                                    continue;
                                }
                                self.value.push_str("\n\n");
                                continue;
                            }
                            let delimiter = if last_char == '\n' { "" } else { " " };
                            self.value.push_str(&format!("{}{}", delimiter, token.word[3..].trim()));
                        } else {
                            break;
                        }
                    }
                    Ok(())
                } else {
                    Err(Failure::Quiet(PositionInfo::from_token(meta, meta.get_current_token())))
                }
            }
            None => Err(Failure::Quiet(PositionInfo::from_token(meta, meta.get_current_token())))
        }
    }
}

impl TranslateModule for CommentDoc {
    fn translate(&self, _meta: &mut crate::utils::TranslateMetadata) -> String {
        format!("# {}", self.value.trim().split('\n').join("\n# "))
    }
}

impl DocumentationModule for CommentDoc {
    fn document(&self, _meta: &ParserMetadata) -> String {
        self.value.clone() + "\n\n"
    }
}
