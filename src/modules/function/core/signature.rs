//! Definitions for the function structures in Parsing and Typecheck

use crate::modules::block::Block;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::raw_fragment;
use heraclitus_compiler::prelude::*;
use std::fmt;

/// Globally unique identifier of a function declaration
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct FunctionDeclId(usize);

impl FunctionDeclId {
    pub fn new(id: usize) -> Self {
        FunctionDeclId(id)
    }
}

impl fmt::Display for FunctionDeclId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier of one monomorphized variant within a single declaration.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct FunctionVariantId(usize);

impl FunctionVariantId {
    pub fn new(id: usize) -> Self {
        FunctionVariantId(id)
    }
}

impl fmt::Display for FunctionVariantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Function declaration parameter.
#[derive(Clone, Debug)]
pub struct FunctionParam {
    pub name: String,
    pub kind: Type,
    /// Default value, present when the parameter was declared as `name = expr`.
    pub default: Option<Expr>,
    pub is_ref: bool,
    pub token: Option<Token>,
}

impl FunctionParam {
    pub fn new(name: String, kind: Type) -> Self {
        FunctionParam {
            name,
            kind,
            default: None,
            is_ref: false,
            token: None,
        }
    }

    pub fn with_default(mut self, default: Option<Expr>) -> Self {
        self.default = default;
        self
    }

    pub fn with_ref(mut self, is_ref: bool) -> Self {
        self.is_ref = is_ref;
        self
    }

    pub fn with_token(mut self, token: Option<Token>) -> Self {
        self.token = token;
        self
    }

    pub fn is_optional(&self) -> bool {
        self.default.is_some()
    }
}

/// Function declaration data without body
#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub id: FunctionDeclId,
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub returns: Type,
    pub is_public: bool,
    pub is_failable: bool,
}

impl FunctionSignature {
    /// Number of parameters that must be supplied at a call site.
    pub fn required_arity(&self) -> usize {
        self.params
            .iter()
            .filter(|param| !param.is_optional())
            .count()
    }

    /// Number of parameters in total, optional ones included.
    pub fn total_arity(&self) -> usize {
        self.params.len()
    }

    /// Number of parameters that carry a default value.
    pub fn optional_arity(&self) -> usize {
        self.params
            .iter()
            .filter(|param| param.is_optional())
            .count()
    }

    /// Whether every parameter carries a concrete type annotation.
    pub fn is_fully_typed(&self) -> bool {
        self.params.iter().all(|param| param.kind != Type::Generic)
    }

    /// Whether the parameter list mixes annotated and generic parameters
    pub fn has_mixed_typing(&self) -> bool {
        let any_generic = self.params.iter().any(|param| param.kind == Type::Generic);
        let any_typed = self.params.iter().any(|param| param.kind != Type::Generic);
        any_generic && any_typed
    }

    pub fn param_types(&self) -> Vec<Type> {
        self.params.iter().map(|param| param.kind.clone()).collect()
    }

    /// Returns default values for arguments that weren't provided
    pub fn defaults_after(&self, provided: usize) -> impl Iterator<Item = &Expr> {
        let missing = self.total_arity().saturating_sub(provided);
        let already_given = self.optional_arity().saturating_sub(missing);
        self.params
            .iter()
            .filter_map(|param| param.default.as_ref())
            .skip(already_given)
    }
}

/// Parameter of a function variant
pub struct FunctionVariantParam {
    pub name: String,
    pub kind: Type,
    pub global_id: Option<usize>,
    pub is_ref: bool,
}

impl FunctionVariantParam {
    pub fn from_tuple((param, global_id, kind): (&FunctionParam, &Option<usize>, &Type)) -> Self {
        FunctionVariantParam {
            name: param.name.clone(),
            kind: kind.clone(),
            global_id: *global_id,
            is_ref: param.is_ref,
        }
    }
}

/// One monomorphized function variant, ready for translation.
#[derive(Clone, Debug)]
pub struct FunctionVariant {
    pub id: FunctionVariantId,
    pub param_types: Vec<Type>,
    pub param_global_ids: Vec<Option<usize>>,
    pub returns: Type,
    pub body: Block,
}

/// Shell fragment representation of function signature
#[derive(Clone, Debug)]
pub struct FunctionFragmentSignature {
    pub name: String,
    pub declaration_id: FunctionDeclId,
    pub variant_id: FunctionVariantId,
    pub return_type: Type,
}

impl FunctionFragmentSignature {
    /// The default value to be returned when function doesn't return anything or fails.
    pub fn default_return(&self) -> FragmentKind {
        if self.return_type.is_array() {
            raw_fragment!("")
        } else {
            raw_fragment!("''")
        }
    }
}
