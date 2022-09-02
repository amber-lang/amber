use heraclitus_compiler::prelude::*;
use crate::modules::variable::variable_name_extensions;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::block::Block;

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub args: Vec<String>,
    pub body: Block
}

impl SyntaxModule<ParserMetadata> for FunctionDeclaration {
    syntax_name!("Function Declaration");

    fn new() -> Self {
        FunctionDeclaration {
            name: String::new(),
            args: vec![],
            body: Block::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "fun")?;
        self.name = variable(meta, variable_name_extensions())?;
        token(meta, "(")?;
        while let Some(tok) = meta.get_current_token() {
            if tok.word == ")" {
                break;
            }
            self.args.push(variable(meta, variable_name_extensions())?);
            match token(meta, ")") {
                Ok(_) => break,
                Err(_) => token(meta, ",")?
            };
        }
        token(meta, "{")?;
        syntax(meta, &mut self.body)?;
        token(meta, "}")?;
        Ok(())
    }
}

impl TranslateModule for FunctionDeclaration {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // Increase indentation level
        meta.increase_indent();
        // Parse the function body
        let mut result = vec![];
        result.push(format!("function {} {{", self.name));
        result.push(meta.gen_indent() + &self.body.translate(meta));
        result.push("}".to_string());
        // Decrease the indentation
        meta.decrease_indent();
        // Return the translation
        result.join("\n")
    }
}