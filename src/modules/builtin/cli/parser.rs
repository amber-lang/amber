use crate::docs::module::DocumentationModule;
use crate::modules::builtin::cli::param::{ParamImpl, ParamKind};
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

    pub fn translate(&self, meta: &mut TranslateMetadata, args: &Expr) -> String {
        let mut output = Vec::new();
        let indent = TranslateMetadata::single_indent();
        // Run getopt to parse command line
        let getopt = self.create_getopt(meta, args);
        output.push(format!("getopt=$({getopt}) || exit"));
        output.push(String::from("eval set -- $getopt"));
        output.push(String::from("while true; do"));
        output.push(format!("{indent}case \"$1\" in"));
        // Extract optional parameters
        for param in &self.params {
            let param = param.borrow();
            self.append_optional(&mut output, &indent, args, &param);
        }
        // Stop at "--" or unexpected parameter
        output.push(format!("{indent}--)"));
        output.push(format!("{indent}{indent}shift"));
        output.push(format!("{indent}{indent}break"));
        output.push(format!("{indent}{indent};;"));
        output.push(format!("{indent}*)"));
        output.push(format!("{indent}{indent}exit 1"));
        output.push(format!("{indent}{indent};;"));
        output.push(format!("{indent}esac"));
        output.push(String::from("done"));
        // Skip "$0" in remaining parameters
        output.push(String::from("shift"));
        // Extract positional parameters
        for param in &self.params {
            let param = param.borrow();
            self.append_positional(&mut output, &param);
        }
        meta.stmt_queue.push_back(output.join("\n"));
        String::new()
    }

    fn create_getopt(&self, meta: &mut TranslateMetadata, args: &Expr) -> String {
        let mut all_shorts = Vec::new();
        let mut all_longs = Vec::new();
        for param in &self.params {
            let param = param.borrow();
            if let ParamKind::Optional(shorts, longs, _) = &param.kind {
                let colon = match param.default.kind {
                    Type::Bool | Type::Null => "",
                    _ => ":",
                };
                for short in shorts {
                    all_shorts.push(format!("{short}{colon}"));
                }
                for long in longs {
                    all_longs.push(format!("{long}{colon}"));
                }
            }
        }
        let shorts = all_shorts.join("");
        let longs = all_longs.join(",");
        let args = args.translate(meta);
        format!("getopt --options={shorts} --longoptions={longs} -- {args}")
    }

    fn append_optional(&self, output: &mut Vec<String>, indent: &str, args: &Expr, param: &ParamImpl) {
        if let ParamKind::Optional(shorts, longs, help) = &param.kind {
            let option = ParamImpl::describe_optional(shorts, longs);
            output.push(format!("{indent}{option})"));
            if *help {
                let run_name = Self::create_run_name(args);
                self.append_help(output, indent, run_name);
            } else {
                let name = &param.name;
                match param.default.kind {
                    Type::Null => {
                        output.push(format!("{indent}{indent}{name}=1"));
                        output.push(format!("{indent}{indent}shift"));
                    }
                    Type::Bool => {
                        let value = param.invert_default_bool();
                        output.push(format!("{indent}{indent}{name}={value}"));
                        output.push(format!("{indent}{indent}shift"));
                    }
                    Type::Array(_) => {
                        // Optional array parameters with non-empty default
                        // values will *extend* not *replace* the default
                        // values here.  We could code for this edge case,
                        // but I'm not sure it's worth it.
                        output.push(format!("{indent}{indent}{name}+=(\"$2\")"));
                        output.push(format!("{indent}{indent}shift"));
                        output.push(format!("{indent}{indent}shift"));
                    }
                    _ => {
                        output.push(format!("{indent}{indent}{name}=\"$2\""));
                        output.push(format!("{indent}{indent}shift"));
                        output.push(format!("{indent}{indent}shift"));
                    }
                }
            }
            output.push(format!("{indent}{indent};;"));
        }
    }

    fn append_positional(&self, output: &mut Vec<String>, param: &ParamImpl) {
        if let ParamKind::Positional(_) = &param.kind {
            let name = &param.name;
            if let Type::Array(_) = param.default.kind {
                output.push(format!("[ -n \"$1\" ] && {name}=(\"$@\")"));
                output.push(String::from("set --"));
            } else {
                output.push(format!("[ -n \"$1\" ] && {name}=\"$1\""));
                output.push(String::from("shift"));
            }
        }
    }

    fn create_run_name(args: &Expr) -> String {
        let name = args.get_translated_name().unwrap_or_default();
        format!("$(basename ${{{name}[0]}})")
    }

    fn append_help(&self, output: &mut Vec<String>, indent: &str, run_name: String) {
        let params = self.params.iter()
            .map(|param| param.borrow().describe_help())
            .collect::<Vec<_>>();
        let width = params.iter()
            .map(|(x, _)| x.len())
            .max()
            .unwrap_or_default();
        output.push(format!("{indent}{indent}cat <<EOF"));
        output.push(self.about.clone());
        output.push(format!("Syntax: {run_name} [options]"));
        for (option, help) in params {
            let padding = width.checked_sub(option.len()).unwrap_or_default() + 3;
            let padding = ".".repeat(padding);
            output.push(format!("  {option} {padding} {help}"));
        }
        output.push(String::from("EOF"));
        output.push(format!("{indent}{indent}exit 1"))
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
