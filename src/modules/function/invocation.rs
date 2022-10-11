use heraclitus_compiler::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::expression::expr::Expr;

use super::invocation_utils::*;

#[derive(Debug, Clone)]
pub struct FunctionInvocation {
    name: String,
    args: Vec<Expr>,
    kind: Type,
    variant_id: usize,
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
            variant_id: 0,
            id: 0
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Get the function name
        let tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        // Get the arguments
        token(meta, "(")?;
        self.id = handle_function_reference(meta, tok.clone(), &self.name)?;
        loop {
            if token(meta, ")").is_ok() {
                break
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
        (self.kind, self.variant_id) = handle_function_parameters(meta, &self.name, &types, tok)?;
        Ok(())
    }
}

impl TranslateModule for FunctionInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = format!("__{}_v{}", self.id, self.variant_id);
        let args = self.args.iter().map(|arg| arg.translate(meta)).collect::<Vec<String>>().join(" ");
        format!("$({name} {args})")
    }
}