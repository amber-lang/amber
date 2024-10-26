use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::unop::UnOp;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Len {
    value: Box<Expr>,
}

impl Typed for Len {
    fn get_type(&self) -> Type {
        Type::Num
    }
}

impl UnOp for Len {
    fn set_expr(&mut self, expr: Expr) {
        self.value = Box::new(expr);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "len")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Len {
    syntax_name!("Length");

    fn new() -> Self {
        Len {
            value: Box::new(Expr::new()),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if !matches!(self.value.get_type(), Type::Text | Type::Array(_)) {
            let msg = self
                .value
                .get_error_message(meta)
                .message("Length can only be applied to text or array types");
            return Err(Failure::Loud(msg));
        }
        Ok(())
    }
}

impl TranslateModule for Len {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let value = self.value.translate(meta);
        if self.value.get_type() == Type::Text {
            meta.stmt_queue.push_back(format!("__AMBER_LEN={value}"));
            return String::from("\"${#__AMBER_LEN}\"")
        }
        // Case for Array passed as a reference
        if value.starts_with("\"${!") {
                meta.stmt_queue.push_back(format!("__AMBER_LEN=({value})"));
                String::from("\"${#__AMBER_LEN[@]}\"")
        } else {
            format!("\"${{#{}", value.trim_start_matches("\"${"))
                .trim_end()
                .to_string()
        }
    }
}

impl DocumentationModule for Len {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
