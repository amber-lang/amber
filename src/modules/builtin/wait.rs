use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::fragments;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Wait {
    pids: Expr,
}

impl SyntaxModule<ParserMetadata> for Wait {
    syntax_name!("WaitForProcesses");

    fn new() -> Self {
        Wait { pids: Expr::new() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "wait")?;
        token(meta, "(")?;
        syntax(meta, &mut self.pids)?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Wait {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.pids.typecheck(meta)?;
        let pids_type = self.pids.get_type();
        if pids_type != Type::array_of(Type::Int) {
            let position = self.pids.get_position();
            return error_pos!(meta, position => {
                message: "Builtin function `wait` can only be used with values of type [Int]",
                comment: format!("Given type: {}, expected type: {}", pids_type, Type::array_of(Type::Int))
            });
        }
        Ok(())
    }
}

impl TranslateModule for Wait {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        fragments!("wait ", self.pids.translate(meta))
    }
}

impl DocumentationModule for Wait {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
