use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Await {
    pids: Expr,
}

impl SyntaxModule<ParserMetadata> for Await {
    syntax_name!("AwaitProcesses");

    fn new() -> Self {
        Await { pids: Expr::new() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "await")?;
        token(meta, "(")?;
        syntax(meta, &mut self.pids)?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Await {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.pids.typecheck(meta)?;
        let pids_type = self.pids.get_type();
        if pids_type != Type::array_of(Type::Int) && pids_type != Type::Int {
            let position = self.pids.get_position();
            return error_pos!(meta, position => {
                message: "Builtin function `await` can only be used with values of type Int or [Int]",
                comment: format!("Given type: {}, expected type: {} or {}", pids_type, Type::Int, Type::array_of(Type::Int))
            });
        }
        Ok(())
    }
}

impl TranslateModule for Await {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("wait ", self.pids.translate(meta))
    }
}

impl DocumentationModule for Await {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
