//! Documentation rendering for function declarations

use super::FunctionDeclaration;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use std::ffi::OsStr;
use std::path::Path;

impl FunctionDeclaration {
    /// Whether a space belongs between `before` and `word` in the rendered signature.
    fn get_space(&self, parentheses: usize, before: &str, word: &str) -> String {
        if parentheses == 0 && word == "("
            || word == ":"
            || word == ")"
            || word == "]"
            || word == ","
            || word == "?"
            || before == "["
            || before == "("
        {
            return String::new();
        }
        " ".to_string()
    }

    /// Renders function signature by replaying the tokens
    /// from `doc_index` up to the opening brace of the body.
    pub(super) fn render_function_signature(
        &self,
        meta: &ParserMetadata,
        doc_index: usize,
    ) -> Result<String, Failure> {
        let mut result = String::new();
        let mut index = doc_index;
        let mut parentheses = 0;
        let mut before = String::new();
        loop {
            let cur_token = meta.context.expr.get(index);
            let cur_word = cur_token.map_or_else(String::new, |v| v.word.clone());
            if !result.is_empty() {
                result.push_str(&self.get_space(parentheses, &before, &cur_word))
            }
            before.clone_from(&cur_word);
            match cur_word.as_str() {
                "(" => parentheses += 1,
                ")" => parentheses -= 1,
                "{" if parentheses == 0 => break,
                "" => {
                    return error!(
                        meta,
                        cur_token.cloned(),
                        "Error when parsing function signature. Please report this issue."
                    );
                }
                _ => {}
            }
            result.push_str(&cur_word);
            index += 1;
        }
        Ok(result)
    }

    /// Adds the `import` usage line if `--usage` flag is present
    pub(super) fn insert_usage_import_statement(
        &self,
        meta: &ParserMetadata,
        mut comment_text: String,
    ) -> String {
        if meta.doc_usage {
            let lib_name = meta
                .context
                .path
                .as_ref()
                .map(Path::new)
                .and_then(Path::file_name)
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".ab"))
                .map(String::from)
                .unwrap_or_default();
            let import_stmt = format!("import {{ {} }} from \"std/{}\"\n", self.name, lib_name);

            let usage_pattern = regex::Regex::new(r"#\s*Usage\s+```ab").unwrap();
            if usage_pattern.is_match(&comment_text) {
                // Insert import statement after "# Usage"
                return usage_pattern
                    .replace(&comment_text, |caps: &regex::Captures| {
                        format!("{}\n{}", &caps[0], import_stmt)
                    })
                    .to_string();
            } else {
                comment_text += "```ab\n";
                comment_text += &import_stmt;
                comment_text += "```";
            }
        }
        comment_text
    }
}
