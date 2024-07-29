use heraclitus_compiler::prelude::*;
use crate::modules::command::command::Command;
use crate::modules::types::{parse_type, Typed};
use crate::utils::ParserMetadata;
use super::expr::{Expr, ExprType};
use super::ternop::ternary::Ternary;
use super::parentheses::Parentheses;
use crate::modules::variable::get::VariableGet;
use crate::modules::function::invocation::FunctionInvocation;
use crate::modules::builtin::nameof::Nameof;
use crate::{parse_non_operators, parse_operator};
use super::literal::{
    bool::Bool,
    number::Number,
    text::Text,
    array::Array,
    null::Null,
    status::Status
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
    cast::Cast,
    is::Is
};
use super::unop::{
    not::Not,
    neg::Neg
};

impl Expr {
    /// This function will parse expressions in the precedence order.
    /// All the operators are left associative by default, meaning
    /// that they are parsed from left to right
    pub fn parse_expression(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        self.parse_ternary(meta)
    }

    /// This function will parse at the top of the precedence order
    fn parse_non_operators(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        parse_non_operators!(meta, [
            Not, Neg, Nameof,
            // Literals
            Parentheses, Bool, Number, Text, Array, Null, Status,
            // Function invocation
            FunctionInvocation, Command,
            // Variable access
            VariableGet
        ])
    }

    /// Ternary operator is parsed from right to left as it is right associative
    fn parse_ternary(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_range(meta)?;
        while token(meta, "then").is_ok() {
            let branch = self.parse_ternary(meta)?;
            if let Err(err) = token(meta, "else") {
                return error_pos!(meta, err.unwrap_quiet(), "Expected 'else' after 'then'");
            }
            node = parse_operator!(meta, start_index, Ternary {
                cond: Box::new(node),
                true_expr: Box::new(branch),
                false_expr: Box::new(self.parse_ternary(meta)?)
            });
        }
        Ok(node)
    }

    fn parse_range(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_or(meta)?;
        while token(meta, "..").is_ok() {
            let neq = token(meta, "=").is_err();
            node = parse_operator!(meta, start_index, Range {
                from: Box::new(node),
                to: Box::new(self.parse_or(meta)?),
                neq
            });
        }
        Ok(node)
    }

    fn parse_or(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_and(meta)?;
        while token(meta, "or").is_ok() {
            node = parse_operator!(meta, start_index, Or {
                left: Box::new(node),
                right: Box::new(self.parse_and(meta)?)
            });
        }
        Ok(node)
    }

    fn parse_and(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_equality(meta)?;
        while token(meta, "and").is_ok() {
            node = parse_operator!(meta, start_index, And {
                left: Box::new(node),
                right: Box::new(self.parse_equality(meta)?)
            });
        }
        Ok(node)
    }

    fn parse_equality(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_relation(meta)?;
        loop {
            match meta.get_current_token().map_or_else(String::new, |tok| tok.word).as_str() {
                "==" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Eq {
                        left: Box::new(node),
                        right: Box::new(self.parse_relation(meta)?)
                    });
                },
                "!=" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Neq {
                        left: Box::new(node),
                        right: Box::new(self.parse_relation(meta)?)
                    });
                },
                _ => break
            }
        }
        Ok(node)
    }

    fn parse_relation(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_addition(meta)?;
        loop {
            match meta.get_current_token().map_or_else(String::new, |tok| tok.word).as_str() {
                ">" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Gt {
                        left: Box::new(node),
                        right: Box::new(self.parse_addition(meta)?)
                    });
                },
                ">=" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Ge {
                        left: Box::new(node),
                        right: Box::new(self.parse_addition(meta)?)
                    });
                },
                "<" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Lt {
                        left: Box::new(node),
                        right: Box::new(self.parse_addition(meta)?)
                    });
                },
                "<=" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Le {
                        left: Box::new(node),
                        right: Box::new(self.parse_addition(meta)?)
                    });
                },
                _ => break
            }
        }
        Ok(node)
    }

    fn parse_addition(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_multiplication(meta)?;
        loop {
            match meta.get_current_token().map_or_else(String::new, |tok| tok.word).as_str() {
                "+" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Add {
                        kind: node.get_type(),
                        left: Box::new(node),
                        right: Box::new(self.parse_multiplication(meta)?),
                    });
                },
                "-" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Sub {
                        left: Box::new(node),
                        right: Box::new(self.parse_multiplication(meta)?)
                    });
                },
                _ => break
            }
        }
        Ok(node)
    }

    fn parse_multiplication(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_advanced(meta)?;
        loop {
            match meta.get_current_token().map_or_else(String::new, |tok| tok.word).as_str() {
                "*" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Mul {
                        left: Box::new(node),
                        right: Box::new(self.parse_advanced(meta)?)
                    });
                },
                "/" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Div {
                        left: Box::new(node),
                        right: Box::new(self.parse_advanced(meta)?)
                    });
                },
                "%" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Modulo {
                        left: Box::new(node),
                        right: Box::new(self.parse_advanced(meta)?)
                    });
                },
                _ => break
            }
        }
        Ok(node)
    }

    fn parse_advanced(&self, meta: &mut ParserMetadata) -> Result<Expr, Failure> {
        let start_index = meta.get_index();
        let mut node = self.parse_non_operators(meta)?;
        loop {
            match meta.get_current_token().map_or_else(String::new, |tok| tok.word).as_str() {
                "is" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Is {
                        expr: Box::new(node),
                        kind: parse_type(meta)?
                    });
                },
                "as" => {
                    meta.increment_index();
                    node = parse_operator!(meta, start_index, Cast {
                        expr: Box::new(node),
                        kind: parse_type(meta)?
                    });
                },
                _ => break
            }
        }
        Ok(node)
    }
}
