use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::utils::*;

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
                    let mut code_block_column_position: Option<usize> = None;
                    while let Some(token) = meta.get_current_token() {
                        let is_token_underneath = token.pos.0 == col + 1;
                        let last_char = self.value.chars().last().unwrap_or('\n');
                        // If the token is a newline, we add a newline to the comment
                        if token.word.starts_with('\n') {
                            self.value.push('\n');
                            meta.increment_index();
                            continue;
                        }
                        if token.word.starts_with("///") && is_token_underneath {
                            // Update the column of the last comment
                            col = token.pos.0;
                            meta.increment_index();

                            let line = &token.word[3..];
                            let trimmed_line = &line.trim();

                            if trimmed_line.is_empty() {
                                // If the comment signifies a paragraph break, we add two newlines
                                if last_char != '\n' {
                                    self.value.push_str("\n\n");
                                }
                            } else {
                                if last_char != '\n' { self.value.push(' '); }
                                if trimmed_line.starts_with("```") {
                                    if code_block_column_position.is_some() {
                                        code_block_column_position = None;
                                    } else {
                                        code_block_column_position = line.find("```");
                                    }
                                    self.value.push_str(trimmed_line);
                                }
                                else if let Some(code_line_start_index) = code_block_column_position {
                                    // Add code lines relative to the starting code fence's column position. 
                                    let start_index = code_line_start_index.min(line.len());
                                    self.value.push_str(line[start_index..].trim_end());
                                }
                                else {
                                    self.value.push_str(trimmed_line);
                                }
                            }
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

impl TypeCheckModule for CommentDoc {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TranslateModule for CommentDoc {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        let comments = self.value.trim().lines()
            .map(|comment| CommentFragment::new(comment).to_frag())
            .collect::<Vec<_>>();
        BlockFragment::new(comments, false).to_frag()
    }
}

impl DocumentationModule for CommentDoc {
    fn document(&self, _meta: &ParserMetadata) -> String {
        self.value.trim_end().to_string() + "\n"
    }
}
