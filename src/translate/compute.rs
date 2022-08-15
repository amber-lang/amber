use crate::utils::TranslateMetadata;

pub enum ArithType {
    BcSed
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
        ArithType::BcSed => {
            let (left, right) = (left.unwrap_or(format!("")), right.unwrap_or(format!("")));
            // Removes trailing zeros from the expression
            let sed_regex = "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//";
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
            format!("$(echo {left} '{op}' {right} | bc -l | sed '{sed_regex}')")
        }
    }
}