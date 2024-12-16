use crate::docs::module::DocumentationModule;
use crate::modules::builtin::cli::param::ParamImpl;
use crate::modules::builtin::cli::parser::ParserImpl;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct GetoptCli {
    parser: Option<Rc<RefCell<ParserImpl>>>,
    args: Box<Expr>,
}

impl Typed for GetoptCli {
    fn get_type(&self) -> Type {
        Type::Null
    }
}

impl SyntaxModule<ParserMetadata> for GetoptCli {
    syntax_name!("Getopt Invocation");

    fn new() -> Self {
        let args = Box::new(Expr::new());
        Self { parser: None, args }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "getopt")?;
        let mut parser = Expr::new();
        token(meta, "(")?;
        let parser_tok = meta.get_current_token();
        syntax(meta, &mut parser)?;
        token(meta, ",")?;
        syntax(meta, &mut *self.args)?;
        token(meta, ")")?;
        let parser = match ParserImpl::find_parser(meta, &parser) {
            Some(parser) => parser,
            None => return error!(meta, parser_tok, "Expected parser object"),
        };
        parser.borrow_mut().add_param(ParamImpl::help());
        self.parser = Some(parser);
        Ok(())
    }
}

impl TranslateModule for GetoptCli {
    fn translate(&self, _meta: &mut TranslateMetadata) -> String {
        String::new()
    }
}

impl DocumentationModule for GetoptCli {
    fn document(&self, _meta: &ParserMetadata) -> String {
        String::new()
    }
}
