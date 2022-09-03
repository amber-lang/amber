use heraclitus_compiler::prelude::*;
use crate::modules::Type;
use crate::modules::variable::variable_name_extensions;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::block::Block;

use super::{handle_existing_function, handle_add_function, skip_function_body};

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub returns: Type,
    pub body: Block
}

impl SyntaxModule<ParserMetadata> for FunctionDeclaration {
    syntax_name!("Function Declaration");

    fn new() -> Self {
        FunctionDeclaration {
            name: String::new(),
            args: vec![],
            returns: Type::Generic,
            body: Block::new()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "fun")?;
        // Get the function name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        handle_existing_function(meta, tok.clone());
        // Get the arguments
        token(meta, "(")?;
        while let Some(tok) = meta.get_current_token() {
            if tok.word == ")" {
                break;
            }
            let name = variable(meta, variable_name_extensions())?;
            self.args.push((name, Type::Generic));
            match token(meta, ")") {
                Ok(_) => break,
                Err(_) => token(meta, ",")?
            };
        }
        // Parse the body
        token(meta, "{")?;
        let index_begin = meta.get_index();
        skip_function_body(meta);
        let index_end = meta.get_index();
        token(meta, "}")?;
        // Add the function to the memory
        let body = meta.expr[index_begin..index_end].iter().cloned().collect::<Vec<Token>>();
        handle_add_function(meta, &self.name, &self.args, self.returns.clone(), tok, body);
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