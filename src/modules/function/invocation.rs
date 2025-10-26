use std::mem::swap;

use heraclitus_compiler::prelude::*;
use crate::{fragments, raw_fragment};
use crate::modules::prelude::*;
use itertools::izip;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failed::Failed;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::modules::expression::expr::{Expr, ExprType};
use super::invocation_utils::*;

#[derive(Debug, Clone)]
pub struct FunctionInvocation {
    name: String,
    name_tok: Option<Token>,
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

fn is_ref(expr: &Expr) -> bool {
    match &expr.value {
        Some(ExprType::VariableGet(var)) => !var.is_variable_modified(),
        _ => false,
    }
}

impl SyntaxModule<ParserMetadata> for FunctionInvocation {
    syntax_name!("Function Invocation");

    fn new() -> Self {
        FunctionInvocation {
            name: String::new(),
            name_tok: None,
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
            // Get the function name and store token for error reporting
            let tok = meta.get_current_token();
            if let Some(ref tok) = tok {
                (self.line, self.col) = tok.pos;
            }
            self.name = variable(meta, variable_name_extensions())?;
            self.name_tok = tok.clone();
            self.failed.set_function_name(self.name.clone());

            // Parse arguments syntax
            token(meta, "(")?;
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

            // Store position for later error reporting
            self.failed.set_position(PositionInfo::from_between_tokens(meta, tok.clone(), meta.get_current_token()));

            // Try to parse the failed block if present (optional in parse phase)
            syntax(meta, &mut self.failed).ok();

            Ok(())
        })
    }
}

impl TypeCheckModule for FunctionInvocation {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Type-check all arguments first
        for arg in &mut self.args {
            arg.typecheck(meta)?;
        }

        // Look up the function declaration (this requires typecheck phase context)
        self.id = handle_function_reference(meta, self.name_tok.clone(), &self.name)?;

        let function_unit = meta.get_fun_declaration(&self.name).unwrap().clone();
        let expected_arg_count = function_unit.args.len();
        let actual_arg_count = self.args.len();
        let optional_count = function_unit.args.iter().filter(|arg| arg.optional.is_some()).count();

        // Handle missing arguments by filling with optional defaults
        if actual_arg_count < expected_arg_count {
            // Check if we can compensate with optional arguments stored in fun_unit
            if actual_arg_count >= expected_arg_count - optional_count {
                let missing = expected_arg_count - actual_arg_count;
                let provided_optional = optional_count - missing;
                let optionals: Vec<_> = function_unit.args.iter().filter_map(|arg| arg.optional.as_ref()).collect();
                for exp in optionals.iter().skip(provided_optional){
                    self.args.push((*exp).clone());
                }
            }
        }

        // Validate arguments and get function variant
        let types = self.args.iter().map(Expr::get_type).collect::<Vec<Type>>();
        let var_refs = self.args.iter().map(is_ref).collect::<Vec<bool>>();
        self.refs = function_unit.args.iter().map(|arg| arg.is_ref).collect();
        (self.kind, self.variant_id) = handle_function_parameters(meta, self.id, function_unit.clone(), &types, &var_refs, self.name_tok.clone())?;

        // Handle failable function logic
        self.is_failable = function_unit.is_failable;
        if self.is_failable {
            if !self.failed.is_parsed {
                return error!(meta, self.name_tok.clone() => {
                    message: "This function can fail. Please handle the failure",
                    comment: "You can use '?' in the end to propagate the failure"
                });
            }
            self.failed.typecheck(meta)?;
        } else {
            if self.failed.is_parsed {
                let message = Message::new_warn_at_token(meta, self.name_tok.clone())
                    .message("This function cannot fail")
                    .comment("You can remove the 'failed' block or '?' at the end");
                meta.add_message(message);
            }
        }

        Ok(())
    }
}

impl TranslateModule for FunctionInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let name = raw_fragment!("{}__{}_v{}", self.name, self.id, self.variant_id);
        let mut is_silent = self.modifier.is_silent || meta.silenced;
        swap(&mut is_silent, &mut meta.silenced);
        let silent = meta.gen_silent().to_frag();

        let args = izip!(self.args.iter(), self.refs.iter()).map(| (arg, is_ref) | match arg.translate(meta) {
            FragmentKind::VarExpr(var) if *is_ref => var.with_render_type(VarRenderType::BashRef).to_frag(),
            FragmentKind::VarExpr(var) if var.kind.is_array() => fragments!(var.with_render_type(VarRenderType::BashRef).to_frag().with_quotes(false), "[@]"),
            _ if *is_ref => panic!("Reference value accepts only variables"),
            var => var
        }).collect::<Vec<FragmentKind>>();
        let args = ListFragment::new(args).with_spaces().to_frag();
        meta.stmt_queue.push_back(fragments!(name, " ", args, silent));
        swap(&mut is_silent, &mut meta.silenced);
        if self.is_failable {
            let failed = self.failed.translate(meta);
            meta.stmt_queue.push_back(failed);
        }
        if self.kind != Type::Null {
            let invocation_return = format!("__ret_{}{}_v{}", self.name, self.id, self.variant_id);
            let invocation_instance = format!("__ret_{}{}_v{}__{}_{}", self.name, self.id, self.variant_id, self.line, self.col);
            let parsed_invocation_return = VarExprFragment::new(&invocation_return, self.kind.clone()).to_frag();
            let var_stmt = VarStmtFragment::new(&invocation_instance, self.kind.clone(), parsed_invocation_return);
            meta.push_ephemeral_variable(var_stmt).to_frag()
        } else {
            fragments!("''")
        }
    }
}

impl DocumentationModule for FunctionInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
