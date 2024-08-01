use super::binop::{
    add::Add, and::And, div::Div, eq::Eq, ge::Ge, gt::Gt, le::Le, lt::Lt, modulo::Modulo, mul::Mul,
    neq::Neq, or::Or, range::Range, sub::Sub,
};
use super::literal::{
    array::Array, bool::Bool, null::Null, number::Number, status::Status, text::Text,
};
use super::parentheses::Parentheses;
use super::ternop::ternary::Ternary;
use super::typeop::{cast::Cast, is::Is};
use super::unop::{neg::Neg, not::Not};
use crate::docs::module::DocumentationModule;
use crate::modules::builtin::nameof::Nameof;
use crate::modules::command::cmd::Command;
use crate::modules::expression::binop::BinOp;
use crate::modules::expression::ternop::TernOp;
use crate::modules::expression::typeop::TypeOp;
use crate::modules::expression::unop::UnOp;
use crate::modules::function::invocation::FunctionInvocation;
use crate::modules::types::parse_type;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::get::VariableGet;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::{document_expression, parse_expr, parse_expr_group, translate_expression};
use heraclitus_compiler::prelude::*;

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
    Is(Is),
}

#[derive(Debug, Clone, Default)]
pub struct Expr {
    pub value: Option<ExprType>,
    pub kind: Type,
    /// Positions of the tokens enclosing the expression
    pub pos: (usize, usize),
}

impl Typed for Expr {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl Expr {
    pub fn get_error_message(&self, meta: &mut ParserMetadata) -> Message {
        let begin = meta.get_token_at(self.pos.0);
        let end = meta.get_token_at(self.pos.1);
        let pos = PositionInfo::from_between_tokens(meta, begin, end);
        Message::new_err_at_position(meta, pos)
    }

    pub fn is_var(&self) -> bool {
        matches!(self.value, Some(ExprType::VariableGet(_)))
    }

    // Get the variable name if the expression is a variable access
    pub fn get_var_translated_name(&self) -> Option<String> {
        match &self.value {
            Some(ExprType::VariableGet(var)) => Some(var.get_translated_name()),
            _ => None,
        }
    }
}

impl SyntaxModule<ParserMetadata> for Expr {
    syntax_name!("Expr");

    fn new() -> Self {
        Expr {
            value: None,
            kind: Type::Null,
            pos: (0, 0),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let result = parse_expr!(meta, [
            ternary @ TernOp => [ Ternary ],
            range @ BinOp => [ Range ],
            or @ BinOp => [ Or ],
            and @ BinOp => [ And ],
            equality @ BinOp => [ Eq, Neq ],
            relation @ BinOp => [ Gt, Ge, Lt, Le ],
            addition @ BinOp => [ Add, Sub ],
            multiplication @ BinOp => [ Mul, Div, Modulo ],
            types @ TypeOp => [ Is, Cast ],
            unops @ UnOp => [ Neg, Not ],
            literals @ Literal => [
                // Literals
                Parentheses, Bool, Number, Text,
                Array, Null, Nameof, Status,
                // Function invocation
                FunctionInvocation, Command,
                // Variable access
                VariableGet
            ]
        ]);
        *self = result;
        Ok(())
    }
}

impl TranslateModule for Expr {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        translate_expression!(
            meta,
            self.value.as_ref().unwrap(),
            [
                // Ternary conditional
                Ternary,
                // Logical operators
                And,
                Or,
                // Comparison operators
                Gt,
                Ge,
                Lt,
                Le,
                Eq,
                Neq,
                // Arithmetic operators
                Add,
                Sub,
                Mul,
                Div,
                Modulo,
                // Binary operators
                Range,
                Cast,
                Is,
                // Unary operators
                Not,
                Neg,
                Nameof,
                // Literals
                Parentheses,
                Bool,
                Number,
                Text,
                Array,
                Null,
                Status,
                // Function invocation
                FunctionInvocation,
                Command,
                // Variable access
                VariableGet
            ]
        )
    }
}

impl DocumentationModule for Expr {
    fn document(&self, meta: &ParserMetadata) -> String {
        document_expression!(
            meta,
            self.value.as_ref().unwrap(),
            [
                // Ternary conditional
                Ternary,
                // Logical operators
                And,
                Or,
                // Comparison operators
                Gt,
                Ge,
                Lt,
                Le,
                Eq,
                Neq,
                // Arithmetic operators
                Add,
                Sub,
                Mul,
                Div,
                Modulo,
                // Binary operators
                Range,
                Cast,
                Is,
                // Unary operators
                Not,
                Neg,
                Nameof,
                // Literals
                Parentheses,
                Bool,
                Number,
                Text,
                Array,
                Null,
                Status,
                // Function invocation
                FunctionInvocation,
                Command,
                // Variable access
                VariableGet
            ]
        )
    }
}
