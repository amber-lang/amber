use crate::docs::module::DocumentationModule;
use crate::modules::builtin::cli::parser::ParserImpl;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::regex;
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::utils::payload::Payload;
use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub enum ParamKind {
    Positional(String),
    Optional(Vec<char>, Vec<String>, bool),
}

impl ParamKind {
    fn from(option: String) -> Option<Self> {
        let regex = regex!(r"^(?:(\w+)|-(\w)|--(\w+))$");
        let mut names = Vec::new();
        let mut shorts = Vec::new();
        let mut longs = Vec::new();
        for token in option.split("|") {
            if let Some(captures) = regex.captures(token) {
                if let Some(name) = captures.get(1) {
                    let name = name.as_str().to_owned();
                    names.push(name);
                } else if let Some(short) = captures.get(2) {
                    let short = short.as_str().chars().next().unwrap();
                    shorts.push(short);
                } else if let Some(long) = captures.get(3) {
                    let long = long.as_str().to_owned();
                    longs.push(long);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        if names.len() == 1 && (shorts.len() + longs.len()) == 0 {
            let name = names.into_iter().next().unwrap();
            Some(ParamKind::Positional(name))
        } else if names.len() == 0 && (shorts.len() + longs.len()) >= 1 {
            Some(ParamKind::Optional(shorts, longs, false))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ParamImpl {
    pub name: String,
    pub kind: ParamKind,
    pub default: Expr,
    pub help: String,
}

impl ParamImpl {
    pub fn new(kind: ParamKind, default: Expr, help: String) -> Rc<RefCell<Self>> {
        let name = String::new();
        let param = ParamImpl { name, kind, default, help };
        Rc::new(RefCell::new(param))
    }

    pub fn help() -> Rc<RefCell<Self>> {
        let kind = ParamKind::Optional(vec![], vec![String::from("help")], true);
        let default = Expr::new();
        let help = String::from("Show help text");
        Self::new(kind, default, help)
    }

    pub fn set_var_name(&mut self, name: &str, id: Option<usize>) {
        self.name = match id {
            Some(id) => format!("__{id}_{name}"),
            None => name.to_string(),
        };
    }

    pub fn describe_optional(shorts: &Vec<char>, longs: &Vec<String>) -> String {
        let shorts = shorts.iter().map(|short| format!("-{short}"));
        let longs = longs.iter().map(|long| format!("--{long}"));
        shorts.chain(longs).join("|")
    }

    pub fn describe_help(&self) -> (String, String) {
        let mut option = match &self.kind {
            ParamKind::Positional(name) => name.to_uppercase(),
            ParamKind::Optional(shorts, longs, _) => Self::describe_optional(shorts, longs),
        };
        if self.default.kind != Type::Null {
            let default = self.default.kind.to_string();
            option = format!("{option}: {default}");
        }
        (option, self.help.clone())
    }

    pub fn invert_default_bool(&self) -> isize {
        let value = self.default.get_integer_value().unwrap_or_default();
        if value == 0 { 1 } else { 0 }
    }
}

#[derive(Debug, Clone)]
pub struct ParamCli {
    param: Option<Rc<RefCell<ParamImpl>>>,
}

impl ParamCli {
    pub fn get_payload(&self) -> Option<Payload> {
        self.param.as_ref()
            .map(|param| Payload::Param(Rc::clone(param)))
    }
}

impl Typed for ParamCli {
    fn get_type(&self) -> Type {
        self.param.as_ref()
            .map(|param| param.borrow().default.kind.clone())
            .unwrap_or(Type::Null)
    }
}

impl SyntaxModule<ParserMetadata> for ParamCli {
    syntax_name!("Param Invocation");

    fn new() -> Self {
        Self { param: None }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "param")?;
        let mut parser = Expr::new();
        let mut option = Expr::new();
        let mut default = Expr::new();
        let mut help = Expr::new();
        token(meta, "(")?;
        let parser_tok = meta.get_current_token();
        syntax(meta, &mut parser)?;
        token(meta, ",")?;
        let option_tok = meta.get_current_token();
        syntax(meta, &mut option)?;
        token(meta, ",")?;
        syntax(meta, &mut default)?;
        token(meta, ",")?;
        let help_tok = meta.get_current_token();
        syntax(meta, &mut help)?;
        token(meta, ")")?;
        let parser = match ParserImpl::find_parser(meta, &parser) {
            Some(parser) => parser,
            None => return error!(meta, parser_tok, "Expected parser object"),
        };
        let option = match option.get_literal_text() {
            Some(option) => option,
            None => return error!(meta, option_tok, "Expected literal string"),
        };
        let kind = match ParamKind::from(option) {
            Some(kind) => kind,
            None => return error!(meta, option_tok, "Expected option string"),
        };
        let help = match help.get_literal_text() {
            Some(help) => help,
            None => return error!(meta, help_tok, "Expected literal string"),
        };
        let param = ParamImpl::new(kind, default, help);
        parser.borrow_mut().add_param(Rc::clone(&param));
        self.param = Some(param);
        Ok(())
    }
}

impl TranslateModule for ParamCli {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        self.param.as_ref()
            .map(|param| param.borrow().default.translate(meta))
            .unwrap_or_default()
    }
}

impl DocumentationModule for ParamCli {
    fn document(&self, _meta: &ParserMetadata) -> String {
        String::new()
    }
}
