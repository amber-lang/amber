use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use super::expression::expr::Expr;
use super::types::{Type, Typed};

pub mod add;
pub mod sub;
pub mod mul;
pub mod div;
pub mod modulo;

pub fn shorthand_typecheck_allowed_types(
    meta: &mut ParserMetadata,
    operator: &str,
    var_type: &Type,
    rhs: &Expr,
    allowed_types: &[Type],
) -> Result<Type, Failure> {
    let rhs_type = rhs.get_type();
    let rhs_match = allowed_types.iter().any(|types| rhs_type.is_allowed_in(types));
    if !rhs_match || rhs_type != *var_type {
        let message = rhs.get_error_message(meta);
        let msg = format!("Cannot {operator} value of type '{var_type}' with value of type '{rhs_type}'");
        let pretty_types = Type::pretty_disjunction(allowed_types);
        let comment = format!("You can only {operator} values of type {pretty_types} together.");
        Err(Failure::Loud(message.message(msg).comment(comment)))
    } else {
        Ok(rhs_type)
    }
}
