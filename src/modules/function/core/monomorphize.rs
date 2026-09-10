//! Amber's functions are monomorphised.
//! Monomorphization is turning declared function into one concrete variant
//! based on argument types provided on the call site. Because function parameters
//! can be generic (or accept a set of types), Amber generates as many function declaration
//! variants as many they were used in the code to preserve the correct type resolution in shell script.

use crate::modules::block::Block;
use crate::modules::function::core::ordinal_number;
use crate::modules::function::core::signature::{
    FunctionParam, FunctionSignature, FunctionVariant, FunctionVariantId,
};
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::Type;
use crate::utils::context::{Context, VariableDecl, VariableDeclWarn};
use crate::utils::{pluralize, ParserMetadata};
use heraclitus_compiler::prelude::*;
use itertools::izip;

/// The outcome of resolving a function call to a concrete function variant with its return type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCall {
    pub return_type: Type,
    pub variant_id: FunctionVariantId,
}

/// Determines if a newly typechecked variant should be persisted in cache or discarded (dry run).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Persistence {
    Cache,
    Discard,
}

impl Persistence {
    pub fn from_meta(meta: &mut ParserMetadata) -> Self {
        if !meta.first_pass_ctx {
            Persistence::Cache
        } else {
            Persistence::Discard
        }
    }
}

pub struct Monomorphizer<'a> {
    meta: &'a mut ParserMetadata,
    signature: FunctionSignature,
    arg_types: Vec<Type>,
    fun_call_tok: Option<Token>,
}

impl<'a> Monomorphizer<'a> {
    pub fn new(
        meta: &'a mut ParserMetadata,
        signature: FunctionSignature,
        arg_types: &[Type],
        fun_call_tok: Option<Token>,
    ) -> Self {
        Monomorphizer {
            meta,
            signature,
            arg_types: arg_types.to_vec(),
            fun_call_tok,
        }
    }

    /// Typecheck the function body against types of provided function arguments.
    /// Once done, can then persist newly typechecked function variant into cache.
    pub fn run(mut self, persistence: Persistence) -> Result<ResolvedCall, Failure> {
        self.check_arity()?;
        self.check_arg_types()?;

        let mut context = self
            .meta
            .fun_cache
            .get_context(self.signature.id)
            .expect("Function was never declared")
            .clone();
        let block = self
            .meta
            .fun_cache
            .get_block(self.signature.id)
            .expect("Function was never declared")
            .clone()
            .with_needs_noop()
            .with_no_syntax();

        // If this funtion is recursively calling itself,
        // we don't want to typecheck it again
        if let Some(variant) = self.get_variant_reserved() {
            return Ok(ResolvedCall {
                return_type: self.signature.returns.clone(),
                variant_id: variant,
            });
        }

        // Reserve this variant so that it doesn't get typechecked
        // again if this function is recursive
        let variant = self.reserve_variant();
        self.add_forward_function_declarations(&mut context);

        let (result, block, global_ids) = self.typecheck_body(&mut context, block);
        self.release_variant();
        result?;

        // Adopt the inferred return type when the declaration left it generic
        let mut returns = self.signature.returns.clone();
        if let Type::Generic = returns {
            returns = context.fun_ret_type.clone().unwrap_or(Type::Null);
        }

        match persistence {
            // Used for a warm-up typecheck mainly checking for general issues such as unused variables.
            Persistence::Discard => Ok(ResolvedCall {
                return_type: returns,
                variant_id: FunctionVariantId::new(0),
            }),
            // Saves the typechecked AST with its metadata such as inferenced types etc.
            Persistence::Cache => {
                let monomorph = FunctionVariant {
                    // The cache assigns the definitive id on insert
                    id: variant,
                    param_types: self.arg_types.clone(),
                    param_global_ids: global_ids,
                    returns: returns.clone(),
                    body: block,
                };
                match self.meta.add_fun_variant(self.signature.id, monomorph) {
                    Some(variant) => Ok(ResolvedCall {
                        return_type: returns,
                        variant_id: variant,
                    }),
                    None => unreachable!("Function was never declared"),
                }
            }
        }
    }

    fn check_arity(&mut self) -> Result<(), Failure> {
        if self.signature.total_arity() == self.arg_types.len() {
            return Ok(());
        }
        let message = arity_mismatch(&self.signature, self.arg_types.len());
        error!(self.meta, self.fun_call_tok.clone(), message)
    }

    /// Check if provided argument types match defined parameter types
    fn check_arg_types(&mut self) -> Result<(), Failure> {
        if !self.signature.is_fully_typed() {
            return Ok(());
        }
        for (index, (param, given)) in
            izip!(self.signature.params.iter(), self.arg_types.iter()).enumerate()
        {
            if !given.is_allowed_in(&param.kind) {
                let message = argument_type_mismatch(&self.signature, index, param, given);
                return error!(self.meta, self.fun_call_tok.clone(), message);
            }
        }
        Ok(())
    }

    /// Returns the already reserved function variant if it's already being typechecked.
    fn get_variant_reserved(&self) -> Option<FunctionVariantId> {
        self.meta
            .fun_variants_in_typecheck
            .get(&(self.signature.id, self.arg_types.clone()))
            .copied()
    }

    /// Claim the next variant id before typechecking, so recursive calls can see it.
    fn reserve_variant(&mut self) -> FunctionVariantId {
        let variant = FunctionVariantId::new(
            self.meta
                .fun_cache
                .get_variants(self.signature.id)
                .map_or(0, Vec::len),
        );
        self.meta
            .fun_variants_in_typecheck
            .insert((self.signature.id, self.arg_types.clone()), variant);
        variant
    }

    // Release function variant from the map holding currently typechecked ones
    fn release_variant(&mut self) {
        self.meta
            .fun_variants_in_typecheck
            .remove(&(self.signature.id, self.arg_types.clone()));
    }

    /// Add all declared functions after the original function was declared,
    /// to bring them into the function context. This allows to call functions
    /// that were defined AFTER the declaration of the called function.
    fn add_forward_function_declarations(&mut self, context: &mut Context) {
        let Some(funcall_global_scope) = self.meta.context.scopes.first() else {
            return;
        };
        let Some(fundecl_global_scope) = context.scopes.first_mut() else {
            return;
        };
        for (name, decl) in &funcall_global_scope.funs {
            if !fundecl_global_scope.funs.contains_key(name) {
                fundecl_global_scope.funs.insert(name.clone(), decl.clone());
            }
        }
    }

    /// Typecheck the body inside the function's own context, with the arguments
    /// bound as local variables of the concrete provided types.
    fn typecheck_body(
        &mut self,
        context: &mut Context,
        mut block: Block,
    ) -> (Result<(), Failure>, Block, Vec<Option<usize>>) {
        // Capture trace, path and the error position
        let caller_trace = self.meta.context.trace.clone();
        let caller_path = self.meta.context.path.clone();
        let fun_call_pos = self
            .fun_call_tok
            .as_ref()
            .map(|tok| PositionInfo::from_token(self.meta, Some(tok.clone())));

        // Warnings should be displayed only if this is a first pass typecheck
        // It's is to avoid duplicate warning messages
        let should_suppress = self.meta.fun_cache.is_first_pass_done(self.signature.id);
        let signature = self.signature.clone();
        let arg_types = self.arg_types.clone();
        let mut global_ids = Vec::with_capacity(signature.params.len());

        let result = self.meta.with_suppress_warnings(should_suppress, |meta| {
            // Swap the contexts to use the function context
            meta.with_context_ref(context, |meta| {
                // Create a sub context for new variables in function
                meta.with_push_scope(true, |meta| {
                    // Add the function itself to the scope to allow recursion
                    meta.context
                        .scopes
                        .last_mut()
                        .unwrap()
                        .add_fun(signature.clone());

                    for (kind, param) in izip!(&arg_types, &signature.params) {
                        let var = VariableDecl::new(param.name.clone(), kind.clone())
                            .with_warn(VariableDeclWarn::from_token(
                                meta,
                                self.fun_call_tok.clone(),
                            ))
                            .with_ref(param.is_ref);
                        global_ids.push(meta.add_var(var));
                    }
                    // Set the expected return type if specified
                    if signature.returns != Type::Generic {
                        meta.context.fun_ret_type = Some(signature.returns.clone());
                    }
                    // Typecheck the function body
                    if let Err(failure) = block.typecheck(meta) {
                        return Err(prepend_call_site_trace(
                            failure,
                            &caller_trace,
                            &caller_path,
                            &meta.context.path,
                            fun_call_pos.as_ref(),
                        ));
                    }
                    Ok(())
                })
            })
        });

        (result, block, global_ids)
    }
}

/// Rewrites an error's trace so it reads from the caller down into the callee,
/// which is what makes cross-file failures point at the right `import`.
fn prepend_call_site_trace(
    failure: Failure,
    caller_trace: &[PositionInfo],
    caller_path: &Option<String>,
    callee_path: &Option<String>,
    call_site: Option<&PositionInfo>,
) -> Failure {
    match failure {
        Failure::Loud(mut msg) => {
            let mut new_trace = Vec::new();
            if caller_path != callee_path {
                new_trace.extend(caller_trace.to_vec());
            }
            if let Some(pos) = call_site {
                new_trace.push(pos.clone());
            }
            new_trace.extend(msg.trace);
            msg.trace = new_trace;
            Failure::Loud(msg)
        }
        other => other,
    }
}

pub fn arity_mismatch(fun: &FunctionSignature, given: usize) -> String {
    let max_args = fun.total_arity();
    let min_args = fun.required_arity();
    let opt_argument = if max_args > min_args {
        format!(" ({max_args} optional)")
    } else {
        String::new()
    };
    // Determine the correct grammar
    let txt_arguments = pluralize(min_args, "argument", "arguments");
    let txt_given = pluralize(given, "was given", "were given");
    format!(
        "Function '{}' expects {min_args} {txt_arguments}{opt_argument}, but {given} {txt_given}",
        fun.name
    )
}

pub fn argument_type_mismatch(
    fun: &FunctionSignature,
    index: usize,
    param: &FunctionParam,
    given: &Type,
) -> String {
    let arg_name = &param.name;
    let arg_type = &param.kind;
    let fun_name = &fun.name;
    let ordinal = ordinal_number(index);
    format!("{ordinal} argument '{arg_name}' of function '{fun_name}' expects type '{arg_type}', but '{given}' was given")
}
