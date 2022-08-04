use heraclitus_compiler::prelude::*;
use crate::parser::ParserMetadata;
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

#[derive(Debug)]
pub enum ExprType {
    Bool(Bool),
    Number(Number),
    Text(Text),
    Parenthesis(Parenthesis),
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
    value: Option<ExprType>
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
            ExprType::Parenthesis(Parenthesis::new()),
            ExprType::Bool(Bool::new()),
            ExprType::Number(Number::new()),
            ExprType::Text(Text::new())
        ]
    }
    
    fn parse_statement(&mut self, meta: &mut ParserMetadata, statement: ExprType) -> SyntaxResult {
        match statement {
            // Logic operators
            ExprType::And(and) => self.get(meta, and, ExprType::And),
            ExprType::Or(or) => self.get(meta, or, ExprType::Or),
            ExprType::Not(not) => self.get(meta, not, ExprType::Not),
            // Comparison operators
            ExprType::Gt(cmp) => self.get(meta, cmp, ExprType::Gt),
            ExprType::Ge(cmp) => self.get(meta, cmp, ExprType::Ge),
            ExprType::Lt(cmp) => self.get(meta, cmp, ExprType::Lt),
            ExprType::Le(cmp) => self.get(meta, cmp, ExprType::Le),
            ExprType::Eq(cmp) => self.get(meta, cmp, ExprType::Eq),
            ExprType::Neq(cmp) => self.get(meta, cmp, ExprType::Neq),
            // Arithmetic operators
            ExprType::Add(add) => self.get(meta, add, ExprType::Add),
            ExprType::Sub(sub) => self.get(meta, sub, ExprType::Sub),
            ExprType::Mul(mul) => self.get(meta, mul, ExprType::Mul),
            ExprType::Div(div) => self.get(meta, div, ExprType::Div),
            // Literals
            ExprType::Parenthesis(p) => self.get(meta, p, ExprType::Parenthesis),
            ExprType::Bool(bool) => self.get(meta, bool, ExprType::Bool),
            ExprType::Number(num) => self.get(meta, num, ExprType::Number),
            ExprType::Text(txt) => self.get(meta, txt, ExprType::Text)
        }
    }

    // Get result out of the provided module and save it in the internal state
    fn get<S>(&mut self, meta: &mut ParserMetadata, mut module: S, cb: impl Fn(S) -> ExprType) -> SyntaxResult
    where
        S: SyntaxModule<ParserMetadata>
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
            value: None
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
