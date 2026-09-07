use crate::modules::function::core::signature::{FunctionParam, FunctionSignature};
use crate::modules::types::Type;
use crate::utils::{pluralize, ParserMetadata};
use similar_string::find_best_similarity;

/// Convert an index to an ordinal number.
pub fn ordinal_number(index: usize) -> String {
    let index = index + 1;
    let mut result = index.to_string();
    let last_digit = index % 10;
    if last_digit == 1 {
        result.push_str("st");
    } else if last_digit == 2 {
        result.push_str("nd");
    } else if last_digit == 3 {
        result.push_str("rd");
    } else {
        result.push_str("th");
    }
    result
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

/// Suggests the closest known function name if such exsits
pub fn similar_function_hint(meta: &ParserMetadata, name: &str) -> Option<String> {
    let names = Vec::from_iter(meta.get_fun_names());
    find_best_similarity(name, &names).and_then(|(match_name, score)| {
        (score >= 0.75).then(|| format!("Did you mean '{match_name}'?"))
    })
}
