use crate::modules::prelude::*;
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::{handle_variable_reference, variable_name_extensions};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct VariableGet {
    pub name: String,
    kind: Type,
    global_id: Option<usize>,
    is_ref: bool,
    tok: Option<Token>,
}

impl Typed for VariableGet {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl VariableGet {
    pub fn is_variable_modified(&self) -> bool {
        false
    }
}

impl SyntaxModule<ParserMetadata> for VariableGet {
    syntax_name!("Variable Access");

    fn new() -> Self {
        VariableGet {
            name: String::new(),
            kind: Type::Null,
            global_id: None,
            is_ref: false,
            tok: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.tok = meta.get_current_token();
        self.name = variable(meta, variable_name_extensions())?;
        Ok(())
    }
}

impl TypeCheckModule for VariableGet {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let variable = handle_variable_reference(meta, &self.tok, &self.name)?;
        self.global_id = variable.global_id;
        self.is_ref = variable.is_ref;
        self.kind = variable.kind.clone();
        Ok(())
    }
}

impl TranslateModule for VariableGet {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        VarExprFragment::new(&self.name, self.get_type())
            .with_global_id(self.global_id)
            .with_ref(self.is_ref)
            .to_frag()
    }
}

impl DocumentationModule for VariableGet {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
