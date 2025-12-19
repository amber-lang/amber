use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::builtin::len::Len;
use crate::modules::command::cmd::Command;
use crate::modules::expression::binop::BinOp;
use crate::modules::prelude::FragmentKind;
use crate::modules::types::{Typed, Type};
use crate::modules::typecheck::TypeCheckModule;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::modules::expression::typeop::TypeOp;
use crate::modules::expression::ternop::TernOp;
use crate::modules::expression::unop::UnOp;
use crate::modules::types::parse_type;
use std::collections::HashMap;
use super::literal::{
    bool::Bool,
    number::Number,
    integer::Integer,
    text::Text,
    array::Array,
    null::Null,
    status::Status,
};
use crate::modules::expression::access::Access;
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
use crate::{
    document_expression,
    parse_expression,
    parse_expression_group,
    typecheck_expression,
    translate_expression
};

#[derive(Debug, Clone)]
pub enum ExprType {
    Bool(Bool),
    Number(Number),
    Integer(Integer),
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
    Access(Access),
}

impl ExprType {
    pub fn analyze_control_flow(&self) -> Option<bool> {
        match self {
            ExprType::Bool(v) => v.analyze_control_flow(),
            ExprType::And(v) => v.analyze_control_flow(),
            ExprType::Or(v) => v.analyze_control_flow(),
            ExprType::Not(v) => v.analyze_control_flow(),
            ExprType::Parentheses(v) => v.analyze_control_flow(),
            ExprType::Is(v) => v.analyze_control_flow(),
            _ => None
        }
    }

    pub fn extract_facts(&self) -> (HashMap<String, Type>, HashMap<String, Type>) {
        match self {
            ExprType::And(v) => v.extract_facts(),
            ExprType::Or(v) => v.extract_facts(),
            ExprType::Not(v) => v.extract_facts(),
            ExprType::Parentheses(v) => v.extract_facts(),
            ExprType::Is(v) => v.extract_facts(),
            _ => (HashMap::new(), HashMap::new())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Expr {
    pub value: Option<ExprType>,
    pub kind: Type,
    pub position: Option<PositionInfo>,
}

impl Typed for Expr {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl Expr {
    pub fn get_integer_value(&self) -> Option<isize> {
        match &self.value {
            Some(ExprType::Number(value)) => value.get_integer_value(),
            Some(ExprType::Integer(value)) => value.value.parse().ok(),
            Some(ExprType::Neg(value)) => value.get_integer_value(),
            _ => None,
        }
    }

    pub fn get_position(&self) -> PositionInfo {
        self.position.clone().expect("Expr position wasn't set in the parsing stage")
    }

    pub fn get_error_message(&self, meta: &mut ParserMetadata) -> Message {
        let pos = self.get_position();
        Message::new_err_at_position(meta, pos)
    }

    pub fn analyze_control_flow(&self) -> Option<bool> {
        self.value.as_ref().and_then(|val| val.analyze_control_flow())
    }

    pub fn extract_facts(&self) -> (std::collections::HashMap<String, Type>, std::collections::HashMap<String, Type>) {
        self.value.as_ref().map(|val| val.extract_facts()).unwrap_or_default()
    }
}

impl SyntaxModule<ParserMetadata> for Expr {
    syntax_name!("Expr");

    fn new() -> Self {
        Expr {
            value: None,
            kind: Type::Null,
            position: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        *self = parse_expression!(meta, [
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
            access @ PostfixOp => [ Access ],
            literals @ Literal => [
                // Literals
                Parentheses, Bool, Number, Integer, Text,
                Array, Null, Status, Nameof,
                // Builtin invocation
                LinesInvocation,
                // Function invocation
                FunctionInvocation, Command,
                // Variable access
                VariableGet
            ]
        ]);
        Ok(())
    }
}

impl TypeCheckModule for Expr {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        typecheck_expression!(self, meta, self.value.as_mut().unwrap(), [
            Add, And, Array, Bool, Cast, Command, Div, Eq, FunctionInvocation,
            Ge, Gt, Integer, Is, Le, Len, LinesInvocation, Lt, Modulo,
            Mul, Nameof, Neg, Neq, Not, Null, Number, Or, Parentheses,
            Range, Status, Sub, Ternary, Text, VariableGet, Access
        ]);
        Ok(())
    }
}

impl TranslateModule for Expr {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        meta.with_expr_ctx(true, |meta| {
            translate_expression!(meta, self.value.as_ref().unwrap(), [
                Add, And, Array, Bool, Cast, Command, Div, Eq, FunctionInvocation,
                Ge, Gt, Integer, Is, Le, Len, LinesInvocation, Lt, Modulo,
                Mul, Nameof, Neg, Neq, Not, Null, Number, Or, Parentheses,
                Range, Status, Sub, Ternary, Text, VariableGet, Access
            ])
        })
    }
}

impl DocumentationModule for Expr {
    fn document(&self, meta: &ParserMetadata) -> String {
        document_expression!(meta, self.value.as_ref().unwrap(), [
            Add, And, Array, Bool, Cast, Command, Div, Eq, FunctionInvocation,
            Ge, Gt, Integer, Is, Le, Len, LinesInvocation, Lt, Modulo,
            Mul, Nameof, Neg, Neq, Not, Null, Number, Or, Parentheses,
            Range, Status, Sub, Ternary, Text, VariableGet, Access
        ])
    }
}
