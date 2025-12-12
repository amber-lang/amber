use heraclitus_compiler::prelude::*;
use crate::modules::prelude::*;
use crate::utils::pluralize;
use super::expression::expr::Expr;
use super::types::{Type, Typed};
use crate::utils::pretty_join;

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
    if !rhs_match || !rhs_type.is_allowed_in(var_type) {
        let message = rhs.get_error_message(meta);
        let msg = format!("Cannot perform {operator} on value of type '{var_type}' and value of type '{rhs_type}'");
        let pretty_types = pretty_join(allowed_types, "and");
        let sentence = pluralize(allowed_types.len(), "Allowed type is", "Allowed types are");
        let comment = format!("{sentence} {pretty_types}.");
        Err(Failure::Loud(message.message(msg).comment(comment)))
    } else {
        Ok(var_type.clone())
    }
}
