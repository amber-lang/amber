use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::modules::prelude::FragmentKind;
use crate::modules::types::{Type, Typed};
use crate::modules::typecheck::TypeCheckModule;
use crate::utils::metadata::ParserMetadata;
use crate::translate::module::TranslateModule;
use super::expr::Expr;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Parentheses {
    value: Box<Expr>,
    kind: Type
}

impl Parentheses {
    pub fn analyze_control_flow(&self) -> Option<bool> {
        self.value.analyze_control_flow()
    }

    pub fn extract_facts(&self) -> (HashMap<String, Type>, HashMap<String, Type>) {
        self.value.extract_facts()
    }
}


impl Typed for Parentheses {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Parentheses {
    syntax_name!("Parentheses");

    fn new() -> Self {
        Parentheses {
            value: Box::new(Expr::new()),
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "(")?;
        syntax(meta, &mut *self.value)?;
        token(meta, ")")?;
        Ok(())
    }
}

impl TypeCheckModule for Parentheses {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.value.typecheck(meta)?;
        self.kind = self.value.get_type();
        Ok(())
    }
}

impl TranslateModule for Parentheses {
    fn translate(&self, meta: &mut crate::utils::TranslateMetadata) -> FragmentKind {
        self.value.translate(meta)
    }
}

impl DocumentationModule for Parentheses {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
