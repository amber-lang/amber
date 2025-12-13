use crate::modules::expression::expr::Expr;
use crate::modules::expression::unop::UnOp;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::modules::typecheck::TypeCheckModule;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Len {
    value: Box<Expr>,
}

impl Typed for Len {
    fn get_type(&self) -> Type {
        Type::Int
    }
}

impl UnOp for Len {
    fn set_expr(&mut self, expr: Expr) {
        *self.value = expr;
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

    fn parse(&mut self, _meta: &mut ParserMetadata) -> SyntaxResult {
        Ok(())
    }
}

impl TypeCheckModule for Len {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Typecheck the expression first
        self.value.typecheck(meta)?;
        
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
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let value = self.value.translate(meta);
        let id = meta.gen_value_id();
        let var_stmt = VarStmtFragment::new("__length", self.value.get_type(), value).with_global_id(id);
        meta.push_ephemeral_variable(var_stmt).with_length_getter(true).to_frag()
    }
}

impl DocumentationModule for Len {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
