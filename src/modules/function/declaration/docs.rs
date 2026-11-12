//! Documentation rendering for function declarations

use super::FunctionDeclaration;
use crate::utils::ParserMetadata;

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
    ) -> String {
        let mut result = String::new();
        let mut parentheses = 0;
        let mut before = "";
        for token in meta.context.expr.iter().skip(doc_index) {
            let cur_word = token.word.as_str();
            if !result.is_empty() {
                result.push_str(&self.get_space(parentheses, before, cur_word))
            }
            before = cur_word;
            match cur_word {
                "(" => parentheses += 1,
                ")" => parentheses -= 1,
                "{" if parentheses == 0 => break,
                _ => {}
            }
            result.push_str(cur_word);
        }
        result
    }

    /// Adds the `import` usage line if `--usage` flag is present
    pub(super) fn insert_usage_import_statement(
        &self,
        meta: &ParserMetadata,
        mut comment_text: String,
    ) -> String {
        // Only stdlib sources have a known `std/<module>` import path
        let lib_name = meta.context.path.stdlib_module_name();
        if let (true, Some(lib_name)) = (meta.doc_usage, lib_name) {
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
