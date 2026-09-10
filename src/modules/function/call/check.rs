use crate::modules::function::core::monomorphize::{Monomorphizer, Persistence, ResolvedCall};
use crate::modules::function::core::ordinal_number;
use crate::modules::function::core::signature::{FunctionParam, FunctionSignature};
use crate::modules::types::Type;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;
use itertools::izip;
use similar_string::find_best_similarity;

/// Suggests the closest known function name if such exsits
pub fn similar_function_hint(meta: &ParserMetadata, name: &str) -> Option<String> {
    let names = Vec::from_iter(meta.get_fun_names());
    find_best_similarity(name, &names).and_then(|(match_name, score)| {
        (score >= 0.75).then(|| format!("Did you mean '{match_name}'?"))
    })
}

pub fn ref_argument_is_not_a_variable(
    fun: &FunctionSignature,
    index: usize,
    param: &FunctionParam,
) -> String {
    let arg_name = &param.name;
    let fun_name = &fun.name;
    let ordinal = ordinal_number(index);
    format!("Cannot pass {ordinal} argument '{arg_name}' as a reference to the function '{fun_name}' because it is not a variable")
}

/// Looks up the function by name, suggesting a close match when it is unknown.
pub fn resolve_function(
    meta: &ParserMetadata,
    tok: Option<Token>,
    name: &str,
) -> Result<FunctionSignature, Failure> {
    match meta.get_fun_declaration(name) {
        Some(fun_decl) => Ok(fun_decl.clone()),
        None => {
            let message = format!("Function '{name}' does not exist");
            // Find other similar function if one exists
            match similar_function_hint(meta, name) {
                Some(comment) => error!(meta, tok, message, comment),
                None => error!(meta, tok, message),
            }
        }
    }
}

/// Validates if each reference parameter receives a variable argument
pub fn validate_ref_arguments(
    meta: &mut ParserMetadata,
    fun: &FunctionSignature,
    is_variable: &[bool],
    tok: Option<Token>,
) -> Result<(), Failure> {
    for (index, (param, is_variable)) in izip!(fun.params.iter(), is_variable.iter()).enumerate() {
        if param.is_ref && !is_variable {
            let message = ref_argument_is_not_a_variable(fun, index, param);
            return error!(meta, tok, message);
        }
    }
    Ok(())
}

/// Returns the variant compiled for `args`, monomorphizing one if this argument
/// combination has not been seen before.
pub fn resolve_variant(
    meta: &mut ParserMetadata,
    fun: FunctionSignature,
    args: &[Type],
    tok: Option<Token>,
) -> Result<ResolvedCall, Failure> {
    let id = fun.id;
    warmup_declared_types_pass(meta, &fun, tok.clone());

    // If the function was previously called with the same arguments, reuse the
    // variant that was already compiled for them.
    let cached = meta
        .fun_cache
        .get_variants(id)
        .and_then(|variants| variants.iter().find(|variant| variant.param_types == args))
        .map(|variant| ResolvedCall {
            return_type: variant.returns.clone(),
            variant_id: variant.id,
        });

    match cached {
        Some(resolved) => Ok(resolved),
        None => {
            let persistence = Persistence::from_meta(meta);
            Monomorphizer::new(meta, fun, args, tok).run(persistence)
        }
    }
}

/// Typechecks the body once against the declared parameter types, before any
/// concrete call is compiled.
fn warmup_declared_types_pass(
    meta: &mut ParserMetadata,
    fun: &FunctionSignature,
    tok: Option<Token>,
) {
    if meta.fun_cache.is_first_pass_done(fun.id) {
        return;
    }
    let declared_types = fun.param_types();
    let _ = meta.with_first_pass_ctx(true, |meta| {
        Monomorphizer::new(meta, fun.clone(), &declared_types, tok.clone())
            .run(Persistence::Discard)
            .map(|_| ())
    });
    meta.fun_cache.set_first_pass_done(fun.id);
}
