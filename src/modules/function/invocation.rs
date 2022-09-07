use heraclitus_compiler::prelude::*;
use crate::modules::Type;
use crate::modules::variable::variable_name_extensions;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::expression::expr::Expr;
use crate::modules::Typed;

use super::{handle_function_reference, handle_function_parameters};

#[derive(Debug, Clone)]
pub struct FunctionInvocation {
    name: String,
    args: Vec<Expr>,
    kind: Type,
    id: usize
}

impl Typed for FunctionInvocation {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for FunctionInvocation {
    syntax_name!("Function Invocation");

    fn new() -> Self {
        FunctionInvocation {
            name: String::new(),
            args: vec![],
            kind: Type::Null,
            id: 0
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Get the function name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        // Get the arguments
        token(meta, "(")?;
        handle_function_reference(meta, tok, &self.name);
        while let Some(tok) = meta.get_current_token() {
            if tok.word == ")" {
                break;
            }
            let mut expr = Expr::new();
            syntax(meta, &mut expr)?;
            self.args.push(expr);
            match token(meta, ")") {
                Ok(_) => break,
                Err(_) => token(meta, ",")?
            };
        }
        let types = self.args.iter().map(|e| e.get_type()).collect::<Vec<Type>>();
        (self.kind, self.id) = handle_function_parameters(meta, &self.name, &types);
        Ok(())
    }
}

impl TranslateModule for FunctionInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = if self.id != 0 { format!("__{}_{}", self.id, self.name) } else { self.name.clone() };
        let args = self.args.iter().map(|arg| arg.translate(meta)).collect::<Vec<String>>().join(" ");
        format!("{name} {args}")
    }
}