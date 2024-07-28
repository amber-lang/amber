use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::command::command::Command;
use crate::modules::types::{Typed, Type};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use super::literal::{
    bool::Bool,
    number::Number,
    text::Text,
    array::Array,
    range::Range,
    null::Null,
    status::Status
};
use super::binop::{
    add::Add,
    sub::Sub,
    mul::Mul,
    div::Div,
    modulo::Modulo,
    and::And,
    or::Or,
    gt::Gt,
    ge::Ge,
    lt::Lt,
    le::Le,
    eq::Eq,
    neq::Neq,
    cast::Cast,
    is::Is
};
use super::unop::{
    not::Not,
    neg::Neg
};
use super::parentheses::Parentheses;
use crate::modules::variable::get::VariableGet;
use super::ternop::ternary::Ternary;
use crate::modules::function::invocation::FunctionInvocation;
use crate::modules::builtin::nameof::Nameof;
use crate::{document_expression, translate_expression};

#[derive(Debug, Clone)]
pub enum ExprType {
    Bool(Bool),
    Number(Number),
    Text(Text),
    Parentheses(Parentheses),
    VariableGet(VariableGet),
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Div(Div),
    Modulo(Modulo),
    Neg(Neg),
    And(And),
    Or(Or),
    Gt(Gt),
    Ge(Ge),
    Lt(Lt),
    Le(Le),
    Eq(Eq),
    Neq(Neq),
    Not(Not),
    Ternary(Ternary),
    FunctionInvocation(FunctionInvocation),
    Command(Command),
    Array(Array),
    Range(Range),
    Null(Null),
    Cast(Cast),
    Status(Status),
    Nameof(Nameof),
    Is(Is)
}

/// This means that the expression was already parsed previously so we don't need to parse it again
pub type AlreadyParsedExpr = Expr;

#[derive(Debug, Clone)]
pub struct Expr {
    pub value: Option<ExprType>,
    pub kind: Type
}

impl Typed for Expr {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl Expr {
    pub fn is_var(&self) -> bool {
        matches!(self.value, Some(ExprType::VariableGet(_)))
    }

    // Get the variable name if the expression is a variable access
    pub fn get_var_translated_name(&self) -> Option<String> {
        match &self.value {
            Some(ExprType::VariableGet(var)) => Some(var.get_translated_name()),
            _ => None
        }
    }
}

impl SyntaxModule<ParserMetadata> for Expr {
    syntax_name!("Expr");

    fn new() -> Self {
        Expr {
            value: None,
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let result = self.parse_expression(meta)?;
        self.value = result.value;
        self.kind = result.kind;
        Ok(())
    }
}

impl TranslateModule for Expr {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        translate_expression!(meta, self.value.as_ref().unwrap(), [
            // Ternary conditional
            Ternary,
            // Logical operators
            And, Or,
            // Comparison operators
            Gt, Ge, Lt, Le, Eq, Neq,
            // Arithmetic operators
            Add, Sub, Mul, Div, Modulo,
            // Unary operators
            Cast, Not, Neg, Nameof, Is,
            // Literals
            Range, Parentheses, Bool, Number, Text, Array, Null, Status,
            // Function invocation
            FunctionInvocation, Command,
            // Variable access
            VariableGet
        ])
    }
}

impl DocumentationModule for Expr {
    fn document(&self, meta: &ParserMetadata) -> String {
        document_expression!(meta, self.value.as_ref().unwrap(), [
            // Ternary conditional
            Ternary,
            // Logical operators
            And, Or,
            // Comparison operators
            Gt, Ge, Lt, Le, Eq, Neq,
            // Arithmetic operators
            Add, Sub, Mul, Div, Modulo,
            // Unary operators
            Cast, Not, Neg, Nameof, Is,
            // Literals
            Range, Parentheses, Bool, Number, Text, Array, Null, Status,
            // Function invocation
            FunctionInvocation, Command,
            // Variable access
            VariableGet
        ])
    }
}
