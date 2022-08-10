use heraclitus_compiler::prelude::*;
use crate::modules::{Typed, Type};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use super::literal::{
    bool::Bool,
    number::Number,
    text::Text,
    command::Command
};
use super::binop::{
    add::Add,
    sub::Sub,
    mul::Mul,
    div::Div,
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
    not::Not
};
use super::parenthesis::Parenthesis;
use crate::modules::variable::get::VariableGet;
use crate::handle_types;

#[derive(Debug)]
pub enum ExprType {
    Bool(Bool),
    Number(Number),
    Text(Text),
    Command(Command),
    Parenthesis(Parenthesis),
    VariableGet(VariableGet),
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Div(Div),
    And(And),
    Or(Or),
    Gt(Gt),
    Ge(Ge),
    Lt(Lt),
    Le(Le),
    Eq(Eq),
    Neq(Neq),
    Not(Not)
}

#[derive(Debug)]
pub struct Expr {
    value: Option<ExprType>,
    kind: Type
}

impl Typed for Expr {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl Expr {
    handle_types!(ExprType, [
        // Logical operators
        And, Or, Not,
        // Comparison operators
        Gt, Ge, Lt, Le, Eq, Neq,
        // Arithmetic operators
        Add, Sub, Mul, Div,
        // Literals
        VariableGet, Parenthesis,
        Command, Bool, Number, Text
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
            kind: Type::Void
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let mut error = None;
        let statements = self.get_modules();
        for statement in statements {
            match self.parse_match(meta, statement) {
                Ok(()) => return Ok(()),
                Err(details) => error = Some(details)
            }
        }
        Err(error.unwrap())
    }
}

impl TranslateModule for Expr {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        self.translate_match(meta, &self.value.as_ref().unwrap())
    }
}