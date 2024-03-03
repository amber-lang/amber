use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
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
    neq::Neq
};
use super::unop::{
    not::Not,
    cast::Cast,
    is::Is
};
use super::parenthesis::Parenthesis;
use crate::modules::variable::get::VariableGet;
use crate::modules::command::expr::CommandExpr;
use crate::modules::condition::ternary::Ternary;
use crate::modules::function::invocation::FunctionInvocation;
use crate::modules::builtin::nameof::Nameof;
use crate::handle_types;

#[derive(Debug, Clone)]
pub enum ExprType {
    Bool(Bool),
    Number(Number),
    Text(Text),
    CommandExpr(CommandExpr),
    Parenthesis(Parenthesis),
    VariableGet(VariableGet),
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Div(Div),
    Modulo(Modulo),
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
    Array(Array),
    Range(Range),
    Null(Null),
    Cast(Cast),
    Status(Status),
    Nameof(Nameof),
    Is(Is)
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub value: Option<ExprType>,
    kind: Type
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

    handle_types!(ExprType, [
        // Ternary conditional
        Ternary,
        // Logical operators
        And, Or,
        // Comparison operators
        Gt, Ge, Lt, Le, Eq, Neq,
        // Arithmetic operators
        Add, Sub, Mul, Div, Modulo,
        // Unary operators
        Cast, Not, Nameof, Is,
        // Literals
        Range, Parenthesis, CommandExpr, Bool, Number, Text, Array, Null, Status,
        // Function invocation
        FunctionInvocation,
        // Variable access
        VariableGet
    ]);

    // Get result out of the provided module and save it in the internal state
    fn get<S>(&mut self, meta: &mut ParserMetadata, mut module: S, cb: impl Fn(S) -> ExprType) -> SyntaxResult
    where
        S: SyntaxModule<ParserMetadata> + Typed
    {
        // Match syntax
        match syntax(meta, &mut module) {
            Ok(()) => {
                self.kind = module.get_type();
                self.value = Some(cb(module));
                Ok(())
            }
            Err(details) => Err(details)
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
        let exprs = self.get_modules();
        for expr in exprs {
            match self.parse_match(meta, expr) {
                Ok(()) => return Ok(()),
                Err(failure) => {
                    if let Failure::Loud(err) = failure {
                        return Err(Failure::Loud(err))
                    }
                }
            }
        }
        error!(meta, meta.get_current_token(), "Expected expression")
    }
}

impl TranslateModule for Expr {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        self.translate_match(meta, self.value.as_ref().unwrap())
    }
}

impl DocumentationModule for Expr {
    fn document(&self) -> String {
        self.document_match(self.value.as_ref().unwrap())
    }
}
