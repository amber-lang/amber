use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "shellname"]
#[kind = "builtin_expr"]
pub struct Shellname {}

impl Typed for Shellname {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Shellname {
    syntax_name!("Shellname");

    fn new() -> Self {
        Shellname {}
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "shellname")?;
        token(meta, "(")?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Shellname {
    fn typecheck(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl Shellname {
    /// Check if this shellname() call is being compared to a literal string.
    /// Returns the literal value if found, or None if it's a standalone usage.
    pub fn try_fold_comparison(&self, meta: &TranslateMetadata, other: &Expr, operator_eq: bool) -> Option<FragmentKind> {
        // Get the target shell's family name
        let target_family = meta.target.shell.family_name();
        
        // Check if the other side is a literal string (single TextPart::String)
        let literal = match &other.value {
            Some(ExprType::Text(text)) => text.as_simple_string()?,
            _ => return None,
        };
        
        // Compare target family with the literal
        let matches = target_family == literal;
        
        // Return constant "1" for true, "0" for false
        // For != operator, we invert the result
        let result = if operator_eq {
            if matches { "1" } else { "0" }
        } else {
            if matches { "0" } else { "1" }
        };
        
        Some(RawFragment::from(result.to_string()).to_frag())
    }
}

impl TranslateModule for Shellname {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Only set shellname_used flag for standalone usage
        // (not when it's being folded via try_fold_comparison)
        meta.shellname_used = true;
        VarExprFragment::new("EXEC_SHELL", Type::Text).to_frag()
    }
}

crate::impl_documentation_noop!(Shellname);
