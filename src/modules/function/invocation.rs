use heraclitus_compiler::prelude::*;
use itertools::izip;
use crate::modules::condition::failed::Failed;
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
    refs: Vec<bool>,
    kind: Type,
    variant_id: usize,
    id: usize,
    failed: Failed,
    is_failable: bool
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
            refs: vec![],
            kind: Type::Null,
            variant_id: 0,
            id: 0,
            failed: Failed::new(),
            is_failable: false
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
        let function_unit = meta.get_fun_declaration(&self.name).unwrap().clone();
        self.is_failable = function_unit.is_failable;
        if self.is_failable {
            match syntax(meta, &mut self.failed) {
                Ok(_) => {},
                Err(Failure::Quiet(_)) => return error!(meta, tok => {
                    message: "This function can fail. Please handle the failure",
                    comment: "You can use '?' in the end to propagate the failure"
                }),
                Err(err) => return Err(err)
            }
        } else {
            let tok = meta.get_current_token();
            if let Ok(symbol) = token_by(meta, |word| ["?", "failed"].contains(&word.as_str())) {
                let message = Message::new_warn_at_token(meta, tok)
                    .message("This function cannot fail")
                    .comment(format!("You can remove the '{symbol}' in the end"));
                meta.add_message(message);
            }
        }
        let types = self.args.iter().map(|e| e.get_type()).collect::<Vec<Type>>();
        let var_names = self.args.iter().map(|e| e.is_var()).collect::<Vec<bool>>();
        self.refs = function_unit.arg_refs.clone();
        (self.kind, self.variant_id) = handle_function_parameters(meta, self.id, function_unit, &types, &var_names, tok)?;
        Ok(())
    }
}

impl TranslateModule for FunctionInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = format!("{}__{}_v{}", self.name, self.id, self.variant_id);
        let silent = meta.gen_silent();
        let args = izip!(self.args.iter(), self.refs.iter()).map(| (arg, is_ref) | {
            if *is_ref {
                arg.get_var_translated_name().unwrap()
            } else {
                let translation = arg.translate_eval(meta, false);
                // If the argument is an array, we have to get just the "name[@]" part
                (translation.starts_with("\"${") && translation.ends_with("[@]}\""))
                    .then(|| translation.get(3..translation.len() - 2).unwrap().to_string())
                    .unwrap_or_else(|| translation)
            }
        }).collect::<Vec<String>>().join(" ");
        meta.stmt_queue.push_back(format!("{name} {args}{silent}"));
        if self.is_failable {
            let failed = self.failed.translate(meta);
            meta.stmt_queue.push_back(failed);
        }
        if matches!(self.kind, Type::Array(_)) {
            let quote = meta.gen_quote();
            format!("{quote}${{__AMBER_FUN_{}{}_v{}[@]}}{quote}", self.name, self.id, self.variant_id)
        } else {
            format!("${{__AMBER_FUN_{}{}_v{}}}", self.name, self.id, self.variant_id)
        }
    }
}