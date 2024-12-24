use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::builtin::len::Len;
use crate::modules::command::cmd::Command;
use crate::modules::expression::binop::BinOp;
use crate::modules::types::{Typed, Type};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::modules::expression::typeop::TypeOp;
use crate::modules::expression::ternop::TernOp;
use crate::modules::expression::unop::UnOp;
use crate::modules::types::parse_type;
use super::literal::{
    bool::Bool,
    number::Number,
    text::Text,
    array::Array,
    null::Null,
    status::Status,
};
use super::binop::{
    add::Add,
    sub::Sub,
    mul::Mul,
    div::Div,
    modulo::Modulo,
    range::Range,
    and::And,
    or::Or,
    gt::Gt,
    ge::Ge,
    lt::Lt,
    le::Le,
    eq::Eq,
    neq::Neq,
};
use super::unop::{
    not::Not,
    neg::Neg,
};
use super::typeop::{
    cast::Cast,
    is::Is,
};
use super::parentheses::Parentheses;
use crate::modules::variable::get::VariableGet;
use super::ternop::ternary::Ternary;
use crate::modules::function::invocation::FunctionInvocation;
use crate::modules::builtin::lines::LinesInvocation;
use crate::modules::builtin::nameof::Nameof;
use crate::{document_expression, parse_expr, parse_expr_group, translate_expression};
use crate::utils::payload::Payload;

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
    LinesInvocation(LinesInvocation),
    FunctionInvocation(FunctionInvocation),
    Command(Command),
    Array(Array),
    Range(Range),
    Null(Null),
    Cast(Cast),
    Status(Status),
    Nameof(Nameof),
    Len(Len),
    Is(Is),
}

#[derive(Debug, Clone, Default)]
pub struct Expr {
    pub value: Option<ExprType>,
    pub kind: Type,
    /// Positions of the tokens enclosing the expression
    pub pos: (usize, usize)
}

impl Typed for Expr {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl Expr {
    pub fn get_position(&self, meta: &mut ParserMetadata) -> PositionInfo {
        let begin = meta.get_token_at(self.pos.0);
        let end = meta.get_token_at(self.pos.1);
        PositionInfo::from_between_tokens(meta, begin, end)
    }

    pub fn get_error_message(&self, meta: &mut ParserMetadata) -> Message {
        let pos = self.get_position(meta);
        Message::new_err_at_position(meta, pos)
    }

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

    pub fn get_integer_value(&self) -> Option<isize> {
        match &self.value {
            Some(ExprType::Bool(value)) => value.get_integer_value(),
            Some(ExprType::Number(value)) => value.get_integer_value(),
            Some(ExprType::Neg(value)) => value.get_integer_value(),
            _ => None,
        }
    }

    pub fn get_payload(&self) -> Option<Payload> {
        match &self.value {
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
            pos: (0, 0)
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
            unops @ UnOp => [ Neg, Not, Len ],
            literals @ Literal => [
                // Literals
                Parentheses, Bool, Number, Text,
                Array, Null, Status, Nameof,
                // Builtin invocation
                LinesInvocation,
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
        translate_expression!(meta, self.value.as_ref().unwrap(), [
            // Ternary conditional
            Ternary,
            // Logical operators
            And, Or,
            // Comparison operators
            Gt, Ge, Lt, Le, Eq, Neq,
            // Arithmetic operators
            Add, Sub, Mul, Div, Modulo,
            // Binary operators
            Range, Cast, Is,
            // Unary operators
            Not, Neg, Nameof, Len,
            // Literals
            Parentheses, Bool, Number, Text,
            Array, Null, Status,
            // Builtin invocation
            LinesInvocation,
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
            // Binary operators
            Range, Cast, Is,
            // Unary operators
            Not, Neg, Nameof, Len,
            // Literals
            Parentheses, Bool, Number, Text,
            Array, Null, Status,
            // Builtin invocation
            LinesInvocation,
            // Function invocation
            FunctionInvocation, Command,
            // Variable access
            VariableGet
        ])
    }
}
