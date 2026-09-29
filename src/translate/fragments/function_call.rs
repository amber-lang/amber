use itertools::{izip, Itertools};

use crate::{
    fragments,
    modules::{
        function::core::signature::{FunctionDeclId, FunctionVariantId},
        prelude::*,
    },
    translate::fragments::{
        fragment::{FragmentKind, FragmentRenderable},
        get_function_name,
        var_expr::{VarExprFragment, VarRenderType},
    },
    utils::{ShellType, TranslateMetadata},
};

/// Copies an array expression into a fresh ephemeral variable and returns it.
fn push_ephemeral_array_variable(
    meta: &mut TranslateMetadata,
    var: &VarExprFragment,
    optimize_when_unused: bool,
) -> VarExprFragment {
    let id = meta.gen_value_id();
    let temp_name = format!("{}_{id}", var.get_index_typename());
    let stmt = VarStmtFragment::new(
        &temp_name,
        var.kind.clone(),
        FragmentKind::VarExpr(var.clone()),
    )
    .with_optimization_when_unused(optimize_when_unused);
    meta.push_ephemeral_variable(stmt)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCallFragment {
    pub name: String,
    pub declaration_id: FunctionDeclId,
    pub variant_id: FunctionVariantId,
    pub args: Vec<FragmentKind>,
}

impl FunctionCallFragment {
    pub fn new(
        meta: &mut TranslateMetadata,
        name: impl Into<String>,
        declaration_id: FunctionDeclId,
        variant_id: FunctionVariantId,
        args: Vec<FragmentKind>,
        refs: &[bool],
        shell_type: ShellType,
    ) -> Self {
        FunctionCallFragment {
            name: name.into(),
            declaration_id,
            variant_id,
            args: izip!(args, refs)
                .map(|(frag, is_ref)| Self::prepare_arguments(meta, frag, is_ref, shell_type))
                .collect(),
        }
    }

    /// Creates temporary variables if needed
    /// and adjusts the way this argument should render
    pub fn prepare_arguments(
        meta: &mut TranslateMetadata,
        fragment: FragmentKind,
        is_ref: &bool,
        shell: ShellType,
    ) -> FragmentKind {
        match (is_ref, fragment) {
            // Reference argument - must be a variable, passed as a shell reference
            (true, FragmentKind::VarExpr(var)) => var
                .with_render_type(VarRenderType::BashRef)
                .with_array_ref(matches!(shell, ShellType::Zsh))
                .to_frag(),
            (true, _) => unreachable!("Reference value only accepts variables"),

            // An indexed array expression is copied into a temporary variable
            // This is because arrays always need to be passed by name
            // so that they don't expand into multiple parameters
            (false, FragmentKind::VarExpr(var)) if var.kind.is_array() && var.index.is_some() => {
                let var_expr = push_ephemeral_array_variable(meta, &var, true);
                if matches!(shell, ShellType::Ksh) {
                    var_expr
                        .with_render_type(VarRenderType::NameOf)
                        .to_frag()
                        .with_quotes(false)
                } else {
                    fragments!(
                        var_expr
                            .with_render_type(VarRenderType::NameOf)
                            .with_array_ref(matches!(shell, ShellType::Zsh))
                            .to_frag()
                            .with_quotes(true),
                        "[@]"
                    )
                }
            }

            // A whole array is passed by name. Each shell copies it its own way.
            (false, FragmentKind::VarExpr(var)) if var.kind.is_array() => {
                if matches!(shell, ShellType::Ksh) {
                    if var.is_ref {
                        // In ksh93, forwarding a ref array parameter into a by-copy
                        // array parameter of a `function name {}`-declared function
                        // can lose the caller's contents. Create a temp array
                        // first and pass it.
                        let var_expr = push_ephemeral_array_variable(meta, &var, false);
                        var_expr
                            .with_render_type(VarRenderType::NameOf)
                            .to_frag()
                            .with_quotes(false)
                    } else {
                        var.with_render_type(VarRenderType::BashRef)
                            .to_frag()
                            .with_quotes(false)
                    }
                } else {
                    fragments!(
                        var.with_render_type(VarRenderType::BashRef)
                            .with_array_ref(matches!(shell, ShellType::Zsh))
                            .to_frag()
                            .with_quotes(true),
                        "[@]"
                    )
                }
            }

            // Plain scalar arguments are passed through unchanged.
            (_, fragment) => fragment,
        }
    }
}

impl FragmentRenderable for FunctionCallFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let name = get_function_name(&self.name, self.declaration_id, self.variant_id);
        let args = self
            .args
            .into_iter()
            .map(|arg| arg.to_string(meta))
            .join(" ");

        format!("{name} {args}")
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::FunCall(self)
    }
}
