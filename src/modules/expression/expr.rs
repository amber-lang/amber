use heraclitus_compiler::prelude::*;
use crate::modules::{Typed, Type};
use crate::utils::metadata::ParserMetadata;
use super::literal::{
    bool::Bool,
    number::Number,
    text::Text
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

#[derive(Debug)]
pub enum ExprType {
    Bool(Bool),
    Number(Number),
    Text(Text),
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
    fn statement_types(&self) -> Vec<ExprType> {
        vec![
            // Logical operators
            ExprType::And(And::new()),
            ExprType::Or(Or::new()),
            ExprType::Not(Not::new()),
            // Comparison operators
            ExprType::Gt(Gt::new()),
            ExprType::Ge(Ge::new()),
            ExprType::Lt(Lt::new()),
            ExprType::Le(Le::new()),
            ExprType::Eq(Eq::new()),
            ExprType::Neq(Neq::new()),
            // Arithmetic operators
            ExprType::Add(Add::new()),
            ExprType::Sub(Sub::new()),
            ExprType::Mul(Mul::new()),
            ExprType::Div(Div::new()),
            // Literals
            ExprType::VariableGet(VariableGet::new()),
            ExprType::Parenthesis(Parenthesis::new()),
            ExprType::Bool(Bool::new()),
            ExprType::Number(Number::new()),
            ExprType::Text(Text::new())
        ]
    }
    
    fn parse_statement(&mut self, meta: &mut ParserMetadata, statement: ExprType) -> SyntaxResult {
        match statement {
            // Logic operators
            ExprType::And(ex) => self.get(meta, ex, ExprType::And),
            ExprType::Or(ex) => self.get(meta, ex, ExprType::Or),
            ExprType::Not(ex) => self.get(meta, ex, ExprType::Not),
            // Comparison operators
            ExprType::Gt(ex) => self.get(meta, ex, ExprType::Gt),
            ExprType::Ge(ex) => self.get(meta, ex, ExprType::Ge),
            ExprType::Lt(ex) => self.get(meta, ex, ExprType::Lt),
            ExprType::Le(ex) => self.get(meta, ex, ExprType::Le),
            ExprType::Eq(ex) => self.get(meta, ex, ExprType::Eq),
            ExprType::Neq(ex) => self.get(meta, ex, ExprType::Neq),
            // Arithmetic operators
            ExprType::Add(ex) => self.get(meta, ex, ExprType::Add),
            ExprType::Sub(ex) => self.get(meta, ex, ExprType::Sub),
            ExprType::Mul(ex) => self.get(meta, ex, ExprType::Mul),
            ExprType::Div(ex) => self.get(meta, ex, ExprType::Div),
            // Literals
            ExprType::Parenthesis(ex) => self.get(meta, ex, ExprType::Parenthesis),
            ExprType::Bool(ex) => self.get(meta, ex, ExprType::Bool),
            ExprType::Number(ex) => self.get(meta, ex, ExprType::Number),
            ExprType::Text(ex) => self.get(meta, ex, ExprType::Text),
            // Variables
            ExprType::VariableGet(ex) => self.get(meta, ex, ExprType::VariableGet)
        }
    }

    // Get result out of the provided module and save it in the internal state
    fn get<S>(&mut self, meta: &mut ParserMetadata, mut module: S, cb: impl Fn(S) -> ExprType) -> SyntaxResult
    where
        S: SyntaxModule<ParserMetadata> + Typed
    {
        // Match syntax
        match syntax(meta, &mut module) {
            Ok(()) => {
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
        let statements = self.statement_types();
        for statement in statements {
            match self.parse_statement(meta, statement) {
                Ok(()) => return Ok(()),
                Err(details) => error = Some(details)
            }
        }
        Err(error.unwrap())
    }
}
