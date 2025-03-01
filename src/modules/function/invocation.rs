use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::fragments;
use crate::modules::prelude::*;
use itertools::izip;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failed::Failed;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
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
    line: usize,
    col: usize,
    failed: Failed,
    modifier: CommandModifier,
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
            line: 0,
            col: 0,
            failed: Failed::new(),
            modifier: CommandModifier::new().parse_expr(),
            is_failable: false
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            // Get the function name
            let tok = meta.get_current_token();
            if let Some(ref tok) = tok {
                (self.line, self.col) = tok.pos;
            }
            self.name = variable(meta, variable_name_extensions())?;
            // Get the arguments
            token(meta, "(")?;
            self.id = handle_function_reference(meta, tok.clone(), &self.name)?;
            loop {
                if token(meta, ")").is_ok() {
                    break
                }
                let mut arg = Expr::new();
                syntax(meta, &mut arg)?;
                self.args.push(arg);
                match token(meta, ")") {
                    Ok(_) => break,
                    Err(_) => token(meta, ",")?,
                };
            }
            let function_unit = meta.get_fun_declaration(&self.name).unwrap().clone();
            let expected_arg_count = function_unit.arg_refs.len();
            let actual_arg_count = self.args.len();
            let optional_count = function_unit.arg_optionals.len();

            // Case when function call is missing arguments
            if actual_arg_count < expected_arg_count {
                // Check if we can compensate with optional arguments stored in fun_unit
                if actual_arg_count >= expected_arg_count - optional_count {
                    let missing = expected_arg_count - actual_arg_count;
                    let provided_optional = optional_count - missing;
                    for exp in function_unit.arg_optionals.iter().skip(provided_optional){
                        self.args.push(exp.clone());
                    }
                }
            }

            let types = self.args.iter().map(|e| e.get_type()).collect::<Vec<Type>>();
            let var_names = self.args.iter().map(|e| e.is_var()).collect::<Vec<bool>>();
            self.refs.clone_from(&function_unit.arg_refs);
            (self.kind, self.variant_id) = handle_function_parameters(meta, self.id, function_unit.clone(), &types, &var_names, tok.clone())?;

            self.is_failable = function_unit.is_failable;
            if self.is_failable {
                match syntax(meta, &mut self.failed) {
                    Ok(_) => if let Type::Failable(t) = &self.kind {
                        self.kind = *t.clone();
                    },
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

            Ok(())
        })
    }
}

impl FunctionInvocation {
    fn get_variable(&self, meta: &TranslateMetadata, name: &str) -> String {
        let quote = meta.gen_quote();
        if matches!(self.kind, Type::Array(_)) {
            format!("{quote}${{{name}[@]}}{quote}")
        } else if matches!(self.kind, Type::Text) {
            format!("{quote}${{{name}}}{quote}")
        } else {
            format!("{quote}${name}{quote}")
        }
    }
}

impl TranslateModule for FunctionInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> TranslationFragment {
        let name = fragments!(raw: "{}__{}_v{}", self.name, self.id, self.variant_id);
        let mut is_silent = self.modifier.is_silent || meta.silenced;
        swap(&mut is_silent, &mut meta.silenced);
        let silent = meta.gen_silent().to_frag();

        let args = izip!(self.args.iter(), self.refs.iter()).map(| (arg, is_ref) | match arg.translate(meta) {
            TranslationFragment::Var(var) if *is_ref => var.set_render_type(VarRenderType::BashName).to_frag(),
            TranslationFragment::Var(var) if var.kind.is_array() => fragments!(var.set_render_type(VarRenderType::BashName).to_frag().unquote(), "[@]"),
            _ if *is_ref => panic!("Reference value accepts only variables"),
            var @ _ => var
        }).collect::<Vec<TranslationFragment>>();
        let args = ListFragment::new(args, " ").to_frag();
        meta.stmt_queue.push_back(fragments!(name, " ", args, silent));

        let invocation_return = &format!("__ret_{}{}_v{}", self.name, self.id, self.variant_id);
        let invocation_instance = &format!("__ret_{}{}_v{}__{}_{}", self.name, self.id, self.variant_id, self.line, self.col);
        let parsed_invocation_return = self.get_variable(meta, invocation_return);
        swap(&mut is_silent, &mut meta.silenced);
        if self.is_failable {
            let failed = self.failed.translate(meta);
            meta.stmt_queue.push_back(failed);
        }
        let variable = meta.push_stmt_variable_lazy(invocation_instance, None, self.kind.clone(), fragments!(raw: "{}", parsed_invocation_return));
        variable.to_frag()
    }
}

impl DocumentationModule for FunctionInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
