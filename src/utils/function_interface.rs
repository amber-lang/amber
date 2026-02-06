use super::{
    context::{FunctionDecl, FunctionDeclArg},
    function_cache::FunctionInstance,
};
use crate::modules::function::declaration::FunctionDeclarationArgument;
use crate::modules::{block::Block, types::Type};

#[derive(Clone, Debug)]
pub struct FunctionInterface {
    pub id: Option<usize>,
    pub name: String,
    pub args: Vec<FunctionDeclarationArgument>,
    pub returns: Type,
    pub is_public: bool,
    pub is_failable: bool,
}

impl FunctionInterface {
    pub fn into_fun_declaration(self, id: usize) -> FunctionDecl {
        let is_args_typed = self.args.iter().all(|arg| arg.kind != Type::Generic);
        let args = self
            .args
            .into_iter()
            .map(|arg| FunctionDeclArg {
                name: arg.name,
                kind: arg.kind,
                optional: arg.optional,
                is_ref: arg.is_ref,
            })
            .collect();

        FunctionDecl {
            name: self.name,
            args,
            returns: self.returns,
            is_args_typed,
            is_public: self.is_public,
            is_failable: self.is_failable,
            id,
        }
    }

    pub fn into_fun_instance(
        self,
        args_global_ids: Vec<Option<usize>>,
        block: Block,
    ) -> FunctionInstance {
        FunctionInstance {
            variant_id: 0,
            args: self.args.iter().map(|arg| arg.kind.clone()).collect(),
            args_global_ids,
            returns: self.returns,
            block,
        }
    }
}
