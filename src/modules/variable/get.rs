use crate::modules::{SymbolName, prelude::*};
use crate::modules::typecheck::TypeCheckModule;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::{handle_variable_reference, variable_name_extensions};
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct VariableGet {
    pub name: SymbolName,
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

impl SyntaxModule<ParserMetadata> for VariableGet {
    syntax_name!("Variable Access");

    fn new() -> Self {
        VariableGet {
            name: SymbolName::default(),
            kind: Type::Null,
            global_id: None,
            is_ref: false,
            tok: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.tok = meta.get_current_token();
        self.name.used = variable(meta, variable_name_extensions())?;
        Ok(())
    }
}

impl TypeCheckModule for VariableGet {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let variable = handle_variable_reference(meta, &self.tok, &self.name.used)?;
        self.global_id = variable.global_id;
        self.name.declared.clone_from(&variable.name);
        self.is_ref = variable.is_ref;
        self.kind = variable.kind.clone();
        Ok(())
    }
}

impl TranslateModule for VariableGet {
    fn translate(&self, _meta: &mut TranslateMetadata) -> FragmentKind {
        VarExprFragment::new(&self.name.declared, self.get_type())
            .with_global_id(self.global_id)
            .with_ref(self.is_ref)
            .to_frag()
    }
}

crate::impl_documentation_noop!(VariableGet);
