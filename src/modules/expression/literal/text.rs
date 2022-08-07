use heraclitus_compiler::prelude::*;
use crate::{utils::metadata::ParserMetadata, modules::{Type, Typed}};
use crate::modules::expression::expr::Expr;

#[derive(Debug)]
pub struct Text {
    strings: Vec<String>,
    interps: Vec<Expr>
}

impl Text {
    fn closure_full_string(word: &String) -> bool {
        word.starts_with('\'') && word.ends_with('\'') && word.len() > 1
    }

    fn parse_text(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Handle full string
        if let Ok(word) = token_by(meta, Text::closure_full_string) {
            let stripped = word.chars().take(word.len() - 1).skip(1).collect::<String>();
            self.strings.push(stripped);
            Ok(())
        }
        else {
            let mut is_interp = false;
            // Initialize string
            self.strings.push(token_by(meta, |word| word.starts_with('\''))?);
            // Factor rest of the interpolation
            while let Some(token) = meta.get_current_token() {
                // Track interpolations
                match token.word.as_str() {
                    "{" => is_interp = true,
                    "}" => is_interp = false,
                    // Manage inserting strings and intrpolations
                    _ => if is_interp {
                        let mut expr = Expr::new();
                        syntax(meta, &mut expr)?;
                        self.interps.push(expr);
                        // TODO: [H50] In the next release of Heraclitus
                        // Change this line to `meta.offset_index(-1)`
                        meta.set_index(meta.get_index() - 1);
                    }
                    else {
                        self.strings.push(token.word.clone());
                        if token.word.ends_with('\'') {
                            meta.increment_index();
                            return Ok(())
                        }
                    }
                }
                meta.increment_index();
            }
            Err(ErrorDetails::from_metadata(meta))
        }
    }
}

impl Typed for Text {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Text {
    syntax_name!("Text");

    fn new() -> Self {
        Text {
            strings: vec![],
            interps: vec![]
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.parse_text(meta)?;
        Ok(())
    }
}