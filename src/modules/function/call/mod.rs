pub mod check;

use self::check::{resolve_function, resolve_variant, validate_ref_arguments};
use super::core::signature::{FunctionDeclId, FunctionVariantId};
use crate::fragments;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::parse_comma_separated;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::translate::fragments::function_call::FunctionCallFragment;
use crate::translate::fragments::{return_variable_name, returned_value_variable_name};
use heraclitus_compiler::prelude::*;
use itertools::izip;

#[derive(Debug, Clone)]
pub struct FunctionCall {
    name: String,
    name_tok: Option<Token>,
    args: Vec<Expr>,
    refs: Vec<bool>,
    return_type: Type,
    variant_id: FunctionVariantId,
    decl_id: FunctionDeclId,
    line: usize,
    col: usize,
    failure_handler: FailureHandler,
    modifier: CommandModifier,
    is_failable: bool,
}

impl Typed for FunctionCall {
    fn get_type(&self) -> Type {
        self.return_type.clone()
    }
}

fn is_var(expr: &Expr) -> bool {
    match &expr.value {
        Some(ExprType::VariableGet(_)) => true,
        _ => false,
    }
}

impl SyntaxModule<ParserMetadata> for FunctionCall {
    syntax_name!("Function Call");

    fn new() -> Self {
        FunctionCall {
            name: String::new(),
            name_tok: None,
            args: vec![],
            refs: vec![],
            return_type: Type::Null,
            variant_id: FunctionVariantId::new(0),
            decl_id: FunctionDeclId::new(0),
            line: 0,
            col: 0,
            failure_handler: FailureHandler::new(),
            modifier: CommandModifier::new_expr(),
            is_failable: false,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_this, meta| {
            // Get the function name and store token for error reporting
            let tok = meta.get_current_token();
            if let Some(ref tok) = tok {
                (self.line, self.col) = tok.pos;
            }
            self.name = variable(meta, variable_name_extensions())?;
            self.name_tok = tok.clone();
            self.failure_handler.set_function_name(self.name.clone());

            // Parse arguments syntax
            token(meta, "(")?;
            self.args = parse_comma_separated(meta, ")", |meta| {
                let mut arg = Expr::new();
                syntax(meta, &mut arg)?;
                Ok(arg)
            })?;

            // Store position for later error reporting
            self.failure_handler
                .set_position(PositionInfo::from_between_tokens(
                    meta,
                    tok.clone(),
                    meta.get_current_token(),
                ));

            // Parse the failed block if exists
            if let Err(Failure::Loud(msg)) = syntax(meta, &mut self.failure_handler) {
                return Err(Failure::Loud(msg));
            }

            Ok(())
        })
    }
}

impl TypeCheckModule for FunctionCall {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |modifier, meta| {
            // Type-check all arguments first
            for arg in &mut self.args {
                arg.typecheck(meta)?;
            }

            // Look up the function declaration (this requires typecheck phase context)
            let signature = resolve_function(meta, self.name_tok.clone(), &self.name)?;
            self.decl_id = signature.id;

            let expected_arg_count = signature.total_arity();
            let actual_arg_count = self.args.len();
            let optional_count = signature.optional_arity();

            // Handle missing arguments by filling with optional defaults
            if actual_arg_count < expected_arg_count {
                // Check if we can compensate with optional arguments stored in the signature
                if actual_arg_count >= expected_arg_count - optional_count {
                    let defaults: Vec<_> = signature
                        .defaults_after(actual_arg_count)
                        .cloned()
                        .collect();
                    self.args.extend(defaults);
                }
            }

            // Check for type inference on reference arguments
            for (arg, param) in izip!(&mut self.args, &signature.params) {
                if param.is_ref {
                    // Handle array type inference
                    if let (Type::Array(inner), Type::Array(expected_inner)) = (arg.get_type(), &param.kind) {
                        if *inner == Type::Generic && **expected_inner != Type::Generic {
                            if let Some(ExprType::VariableGet(var)) = &arg.value {
                                meta.update_var_type(&var.name, Type::array_of(*expected_inner.clone()));
                                arg.kind = Type::array_of(*expected_inner.clone());
                            }
                        }
                    }
                }
            }

            // Validate arguments and get function variant
            let types = self.args.iter().map(Expr::get_type).collect::<Vec<Type>>();
            let arg_is_variable = self.args.iter().map(is_var).collect::<Vec<bool>>();
            self.refs = signature.params.iter().map(|param| param.is_ref).collect();
            validate_ref_arguments(meta, &signature, &arg_is_variable, self.name_tok.clone())?;
            let resolved = resolve_variant(meta, signature.clone(), &types, self.name_tok.clone())?;
            self.return_type = resolved.return_type;
            self.variant_id = resolved.variant_id;

            // Mark variables passed as reference as modified and used
            for (arg, is_ref) in izip!(self.args.iter(), self.refs.iter()) {
                if *is_ref {
                    if let Some(ExprType::VariableGet(var)) = &arg.value {
                        meta.mark_var_modified(&var.name);
                    }
                }
            }

            // Handle failable function logic
            if modifier.is_trust && self.failure_handler.is_question_mark {
                return error!(meta, self.name_tok.clone() => {
                    message: "The '?' operator cannot be used with the 'trust' modifier because 'trust' ignores failure while '?' propagates it",
                    comment: "You should use either 'trust' or '?' but not both"
                });
            }

            self.is_failable = signature.is_failable;
            if self.is_failable {
                if !self.failure_handler.is_parsed && !meta.context.is_trust_ctx {
                    return error!(meta, self.name_tok.clone() => {
                        message: format!("Function '{}' can potentially fail but is left unhandled.", self.name),
                        comment: "You can use '?' to propagate failure, 'failed' block to handle failure, 'succeeded' block to handle success, 'exited' block to handle both"
                    });
                }
                self.failure_handler.typecheck(meta)?;
            } else if self.failure_handler.is_parsed && !meta.context.is_trust_ctx {
                let message = Message::new_warn_at_token(meta, self.name_tok.clone())
                    .message(format!("Function '{}' cannot fail", self.name))
                    .comment("You can remove the failure handler block or '?' at the end");
                meta.add_message(message);
            }

            Ok(())
        })
    }
}

impl TranslateModule for FunctionCall {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        meta.with_silenced(self.modifier.is_silent || meta.silenced, |meta| {
            let silent = meta.gen_silent().to_frag();
            let suppress = meta.gen_suppress().to_frag();
            let args = self.args.iter().map(|arg| arg.translate(meta)).collect();
            let call = FunctionCallFragment::new(
                meta,
                &self.name,
                self.decl_id,
                self.variant_id,
                args,
                &self.refs,
                meta.target.shell,
            );
            let call_stmt = fragments!(call.to_frag(), silent, suppress);
            meta.stmt_queue.push_back(call_stmt);
        });
        // If there is a failure handler parsed
        if self.is_failable && self.failure_handler.is_parsed {
            let handler = self.failure_handler.translate(meta);
            meta.stmt_queue.push_back(handler);
        }
        // Expose return value as variable
        if self.return_type != Type::Null {
            // Store the return value in a separate variable so that
            // the value persists when this function is called
            // twice or more in this expression,
            let return_var_name = return_variable_name(&self.name, self.decl_id, self.variant_id);
            let return_var_expr =
                VarExprFragment::new(&return_var_name, self.return_type.clone()).to_frag();
            // Name of the callsite variable
            let callsite_var_name = returned_value_variable_name(
                &self.name,
                self.decl_id,
                self.variant_id,
                self.line,
                self.col,
            );
            let callsite_var_stmt = VarStmtFragment::new(
                &callsite_var_name,
                self.return_type.clone(),
                return_var_expr,
            );
            meta.push_ephemeral_variable(callsite_var_stmt).to_frag()
        } else {
            fragments!("''")
        }
    }
}

crate::impl_documentation_noop!(FunctionCall);
