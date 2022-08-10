use crate::utils::TranslateMetadata;

pub enum ArithType {
    BasicCalculator
}

pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Neq,
    Not,
    And,
    Or
}

pub fn translate_computation(meta: &mut TranslateMetadata, operation: ArithOp, left: Option<String>, right: Option<String>) -> String {
    match meta.arith_module {
        ArithType::BasicCalculator => {
            let (left, right) = (left.unwrap_or(format!("")), right.unwrap_or(format!("")));
            let op = match operation {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "*",
                ArithOp::Div => "/",
                ArithOp::Gt => ">",
                ArithOp::Ge => ">=",
                ArithOp::Lt => "<",
                ArithOp::Le => "<=",
                ArithOp::Eq => "==",
                ArithOp::Neq => "!=",
                ArithOp::Not => "!",
                ArithOp::And => "&&",
                ArithOp::Or => "||"
            };
            format!("$(echo {left} {op} {right} | bc -l)")
        }
    }
}