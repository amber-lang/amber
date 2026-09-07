use itertools::izip;

use crate::{
    modules::{
        function::core::signature::{FunctionDeclId, FunctionVariantId, FunctionVariantParam},
        types::Type,
    },
    translate::fragments::{
        block::BlockFragment,
        fragment::{FragmentKind, FragmentRenderable},
        get_function_name,
        var_expr::VarExprFragment,
        var_stmt::VarStmtFragment,
    },
    utils::{ShellType, TranslateMetadata},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDeclFragment {
    pub name: String,
    pub declaration_id: FunctionDeclId,
    pub variant_id: FunctionVariantId,
    pub prologue: Box<FragmentKind>,
    pub body: Box<FragmentKind>,
}

impl FunctionDeclFragment {
    pub fn new(
        name: impl Into<String>,
        declaration_id: FunctionDeclId,
        variant_id: FunctionVariantId,
        body: FragmentKind,
        params: &[FunctionVariantParam],
        shell_type: ShellType,
    ) -> Self {
        FunctionDeclFragment {
            name: name.into(),
            declaration_id: declaration_id,
            variant_id: variant_id,
            prologue: Box::new(Self::params_to_variables(params, shell_type)),
            body: Box::new(body),
        }
    }

    /// Copy an array argument into a function-local array variable.
    fn bind_array_by_value(
        shell: ShellType,
        result: &mut Vec<FragmentKind>,
        positional: VarExprFragment,
        param: &FunctionVariantParam,
    ) {
        // ksh cannot copy a caller-local array through indirect expansion, so bind
        // the argument name as a nameref first and then copy from that local alias.
        if matches!(shell, ShellType::Ksh) {
            let source_name = format!("{}_source", param.name);
            let source_ref =
                VarStmtFragment::new(&source_name, Type::Generic, positional.to_frag())
                    .with_local(true)
                    .with_ref(true)
                    .with_optimization_when_unused(true);

            let var = VarStmtFragment::new(
                &param.name,
                param.kind.clone(),
                VarExprFragment::new(&source_name, param.kind.clone()).to_frag(),
            )
            .with_global_id(param.global_id)
            .with_local(true)
            .with_optimization_when_unused(true);

            result.push(source_ref.to_frag());
            result.push(var.to_frag());
        } else {
            let val = positional.with_ref(true).with_declared(false);
            let var = VarStmtFragment::new(&param.name, param.kind.clone(), val.to_frag())
                .with_global_id(param.global_id)
                .with_local(true)
                .with_optimization_when_unused(true);

            result.push(var.to_frag());
        }
    }

    /// Bind an array `ref` argument so the function can mutate the caller's array.
    fn bind_array_ref(
        shell: ShellType,
        result: &mut Vec<FragmentKind>,
        positional: VarExprFragment,
        param: &FunctionVariantParam,
    ) {
        let val = positional.with_ref(false);
        let base = VarStmtFragment::new(&param.name, param.kind.clone(), val.to_frag())
            .with_global_id(param.global_id)
            .with_local(true)
            .with_optimization_when_unused(false)
            .with_array_ref(true);

        let var = match shell {
            // zsh binds the alias without a nameref
            ShellType::Zsh => base.with_ref(false).with_declared(false),
            // bash 3.2 has no `local -n`, so the assignment is eval-wrapped instead
            ShellType::BashLegacy => base.with_ref(true).with_declared(false),
            _ => base.with_ref(true),
        };

        result.push(var.to_frag());
    }

    /// Bind a scalar argument, preserving `ref` semantics when requested.
    fn bind_scalar(
        result: &mut Vec<FragmentKind>,
        positional: VarExprFragment,
        param: &FunctionVariantParam,
    ) {
        let val = positional.with_ref(false);
        let var = VarStmtFragment::new(&param.name, param.kind.clone(), val.to_frag())
            .with_global_id(param.global_id)
            .with_local(true)
            .with_optimization_when_unused(!param.is_ref)
            .with_declared(!param.is_ref)
            .with_ref(param.is_ref);

        result.push(var.to_frag());
    }

    /// Expands this parameters into the statements that bind it.
    fn params_to_variables(params: &[FunctionVariantParam], shell: ShellType) -> FragmentKind {
        if params.is_empty() {
            return FragmentKind::Empty;
        }
        let mut result: Vec<FragmentKind> = vec![];
        for (index, param) in izip!(params).enumerate() {
            // Represents the positional parameter in shell script ex. `$1`
            let positional = VarExprFragment::new(&format!("{}", index + 1), Type::Generic);
            match (param.is_ref, &param.kind) {
                (false, Type::Array(_)) => {
                    Self::bind_array_by_value(shell, &mut result, positional, param)
                }
                (true, Type::Array(_)) => {
                    Self::bind_array_ref(shell, &mut result, positional, param)
                }
                _ => Self::bind_scalar(&mut result, positional, param),
            }
        }
        BlockFragment::new(result, true).to_frag()
    }
}

impl FragmentRenderable for FunctionDeclFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let mut result = vec![];
        let name = get_function_name(&self.name, self.declaration_id, self.variant_id);
        // ksh needs the `function` keyword to give the body a local scope
        if matches!(meta.target.shell, ShellType::Ksh) {
            result.push(format!("function {name} {{"));
        } else {
            result.push(format!("{name}() {{"));
        }
        // Push prologue only if it's not empty
        let prologue = self.prologue.to_string(meta);
        if !prologue.is_empty() {
            result.push(prologue);
        }
        result.push(self.body.to_string(meta));
        result.push("}\n".to_string());
        result.join("\n")
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::FunDecl(self)
    }
}
