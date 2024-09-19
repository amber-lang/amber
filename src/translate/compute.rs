use crate::utils::TranslateMetadata;

pub enum ArithType {
    BcSed
}

pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Modulo,
    Neg,
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

pub fn translate_computation(meta: &TranslateMetadata, operation: ArithOp, left: Option<String>, right: Option<String>) -> String {
    match meta.arith_module {
        ArithType::BcSed => {
            let (left, right) = (left.unwrap_or_default(), right.unwrap_or_default());
            let mut scale = "";
            let op = match operation {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "*",
                ArithOp::Div => {
                    scale = "scale=0;";
                    "/"
                },
                ArithOp::Modulo => {
                    scale = "scale=0;";
                    "%"
                },
                ArithOp::Neg => "-",
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
            meta.gen_subprocess(&format!("echo \"{scale}{left}{op}{right}\" | bc -l"))
        }
    }
}

pub fn translate_computation_eval(meta: &mut TranslateMetadata, operation: ArithOp, left: Option<String>, right: Option<String>) -> String {
    let old_eval = meta.eval_ctx;
    meta.eval_ctx = true;
    let result = translate_computation(meta, operation, left, right);
    meta.eval_ctx = old_eval;
    result
}
