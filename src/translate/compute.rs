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
            let (left, right) = (left.unwrap_or_default(), right.unwrap_or_default());
            let mut math_lib_flag = true;
            // Removes trailing zeros from the expression
            let sed_regex = "/\\./ s/\\.\\{0,1\\}0\\{1,\\}$//";
            let op = match operation {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "*",
                ArithOp::Div => "/",
                ArithOp::Modulo => {
                    math_lib_flag = false;
                    "%"
                },
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
            let math_lib_flag = if math_lib_flag { "-l" } else { "" };
            meta.externs.push("bc".into());
            meta.externs.push("sed".into());
            meta.gen_subprocess(&format!("echo {left} '{op}' {right} | bc {math_lib_flag} | sed '{sed_regex}'"))
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