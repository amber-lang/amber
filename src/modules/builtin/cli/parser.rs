use crate::docs::module::DocumentationModule;
use crate::modules::builtin::cli::param::ParamImpl;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::utils::payload::Payload;
use heraclitus_compiler::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct ParserImpl {
    about: String,
    params: Vec<Rc<RefCell<ParamImpl>>>,
}

impl ParserImpl {
    pub fn find_parser(meta: &ParserMetadata, parser: &Expr) -> Option<Rc<RefCell<ParserImpl>>> {
        if let Some(var) = meta.get_var_from_expr(parser) {
            if let Some(Payload::Parser(parser)) = &var.payload {
                return Some(Rc::clone(parser));
            }
        }
        None
    }

    pub fn add_param(&mut self, param: Rc<RefCell<ParamImpl>>) {
        self.params.push(param);
    }
}

#[derive(Debug, Clone)]
pub struct ParserCli {
    parser: Option<Rc<RefCell<ParserImpl>>>,
}

impl ParserCli {
    pub fn get_payload(&self) -> Option<Payload> {
        if let Some(parser) = &self.parser {
            Some(Payload::Parser(Rc::clone(parser)))
        } else {
            None
        }
    }
}

impl Typed for ParserCli {
    fn get_type(&self) -> Type {
        Type::Null
    }
}

impl SyntaxModule<ParserMetadata> for ParserCli {
    syntax_name!("Parser Invocation");

    fn new() -> Self {
        Self { parser: None }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "parser")?;
        let mut about = Expr::new();
        token(meta, "(")?;
        let about_tok = meta.get_current_token();
        syntax(meta, &mut about)?;
        token(meta, ")")?;
        let about = match about.get_literal_text() {
            Some(about) => about,
            None => return error!(meta, about_tok, "Expected literal string"),
        };
        let parser = ParserImpl { about, params: Vec::new() };
        self.parser = Some(Rc::new(RefCell::new(parser)));
        Ok(())
    }
}

impl TranslateModule for ParserCli {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        String::new()
    }
}

impl DocumentationModule for ParserCli {
    fn document(&self, _meta: &ParserMetadata) -> String {
        String::new()
    }
}
