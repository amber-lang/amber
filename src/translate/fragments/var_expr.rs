use crate::modules::prelude::*;
use crate::utils::TranslateMetadata;
use crate::modules::types::Type;
use crate::modules::prelude::RawFragment;
use crate::modules::expression::expr::{Expr, ExprType};
use super::fragment::{FragmentKind, FragmentRenderable};
use super::get_variable_name;
use super::var_stmt::VarStmtFragment;

/// Represents a variable expression such as `$var` or `${var}`
#[derive(Debug, Clone)]
pub enum VarRenderType {
    NameOf,
    BashRef,
    BashValue,
}

#[derive(Debug, Clone)]
pub enum VarIndexValue {
    Index(FragmentKind),
    Range(FragmentKind, FragmentKind),
}

#[derive(Debug, Clone)]
pub struct VarExprFragment {
    pub name: String,
    pub global_id: Option<usize>,
    pub kind: Type,
    pub is_ref: bool,
    pub is_length: bool,
    pub is_quoted: bool,
    pub render_type: VarRenderType,
    pub index: Option<Box<VarIndexValue>>,
}

// Represents variable that resolves to a value. Prefixed with `$`.

impl Default for VarExprFragment {
    fn default() -> Self {
        VarExprFragment {
            name: String::new(),
            global_id: None,
            kind: Type::Generic,
            is_ref: false,
            is_length: false,
            is_quoted: true,
            render_type: VarRenderType::BashValue,
            index: None,
        }
    }
}

impl VarExprFragment {
    pub fn new(name: &str, kind: Type) -> Self {
        VarExprFragment {
            name: name.to_string(),
            kind,
            ..Default::default()
        }
    }

    pub fn with_global_id<T: Into<Option<usize>>>(mut self, id: T) -> Self {
        self.global_id = id.into();
        self
    }

    pub fn with_ref(mut self, is_ref: bool) -> Self {
        self.is_ref = is_ref;
        self
    }

    pub fn from_stmt(stmt: &VarStmtFragment) -> Self {
        VarExprFragment {
            name: stmt.name.clone(),
            global_id: stmt.global_id,
            kind: stmt.kind.clone(),
            is_ref: stmt.is_ref,
            ..Default::default()
        }
    }

    pub fn with_index(mut self, meta: &mut TranslateMetadata, index: Option<Expr>) -> Self {
        if let Some(index) = index {
            let index = match index.value {
                Some(ExprType::Range(range)) => {
                    let (offset, length) = range.get_array_index(meta);
                    VarIndexValue::Range(offset, length)
                }
                Some(ExprType::Neg(neg)) => {
                    let index = neg.get_array_index(meta);
                    VarIndexValue::Index(index)
                }
                _ => {
                    let index = index.translate_eval(meta, true);
                    VarIndexValue::Index(index)
                }
            };
            self.index = Some(Box::new(index));
        }
        self
    }

    pub fn with_length_getter(mut self, value: bool) -> Self {
        self.is_length = value;
        self
    }

    pub fn with_render_type(mut self, render_type: VarRenderType) -> Self {
        self.render_type = render_type;
        self
    }

    pub fn with_quotes(mut self, is_quoted: bool) -> Self {
        self.is_quoted = is_quoted;
        self
    }

    pub fn get_name(&self) -> String {
        get_variable_name(&self.name, self.global_id)
    }

    // Returns the variable name in the bash context Ex. "varname"
    pub fn render_bash_reference(self, meta: &mut TranslateMetadata) -> String {
        let dollar = meta.gen_dollar();
        let mut name = self.get_name();
        // Dereference variable if it's a reference and is passed by reference
        if self.is_ref {
            name = format!("{dollar}{name}");
        }

        if self.is_quoted {
            let quote = meta.gen_quote();
            format!("{quote}{name}{quote}")
        } else {
            name
        }
    }

    // Returns the variable value in the bash context Ex. "$varname" or "${varname[@]}"
    pub fn render_bash_value(mut self, meta: &mut TranslateMetadata) -> String {
        let name = self.get_name();
        let index = self.index.take();
        let prefix = self.get_variable_prefix();
        let suffix = self.get_variable_suffix(meta, index);

        if self.is_ref {
            self.render_deref_variable(meta, prefix, &name, &suffix)
        } else {
            let quote = if self.is_quoted { meta.gen_quote() } else { "" };
            let dollar = meta.gen_dollar();
            format!("{quote}{dollar}{{{prefix}{name}{suffix}}}{quote}")
        }
    }

    // Get variable prefix ${PREFIX:varname:suffix}
    fn get_variable_prefix(&self) -> &'static str {
        if self.is_length {
            "#"
        } else {
            ""
        }
    }

    // Get variable suffix ${prefix:varname:SUFFIX}
    fn get_variable_suffix(&self, meta: &mut TranslateMetadata, index: Option<Box<VarIndexValue>>) -> String {
        match (&self.kind, index.map(|var| *var)) {
            (Type::Array(_), Some(VarIndexValue::Range(offset, length))) => {
                let offset = offset.with_quotes(false).to_string(meta);
                let length = length.with_quotes(false).to_string(meta);
                format!("[@]:{offset}:{length}")
            }
            (_, Some(VarIndexValue::Index(index))) => {
                let index = index.with_quotes(false).to_string(meta);
                format!("[{index}]")
            }
            (Type::Array(_), None) => {
                String::from("[@]")
            }
            _ => {
                String::new()
            }
        }
    }

    fn render_deref_variable(self, meta: &mut TranslateMetadata, prefix: &str, name: &str, suffix: &str) -> String {
        let arr_open = if self.kind.is_array() { "(" } else { "" };
        let arr_close = if self.kind.is_array() { ")" } else { "" };
        let quote = if self.is_quoted { meta.gen_quote() } else { "" };
        let dollar = meta.gen_dollar();
        if prefix.is_empty() && suffix.is_empty() {
            return format!("{quote}{dollar}{{!{name}}}{quote}");
        }
        let id = meta.gen_value_id();
        let eval_value = format!("{prefix}${{{name}}}{suffix}");
        // TODO: Check if we can just `{name}_deref` without `__` and/or id.
        let var_name = format!("__{name}_deref_{id}");
        meta.stmt_queue.push_back(RawFragment::from(
            format!("eval \"local {var_name}={arr_open}\\\"\\${{{eval_value}}}\\\"{arr_close}\"")
        ).to_frag());

        if self.kind.is_array() {
            format!("{quote}{dollar}{{{var_name}[@]}}{quote}")
        } else {
            format!("{quote}{dollar}{{{var_name}}}{quote}")
        }
    }
}

impl FragmentRenderable for VarExprFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        match self.render_type {
            VarRenderType::NameOf => self.get_name(),
            VarRenderType::BashRef => self.render_bash_reference(meta),
            VarRenderType::BashValue => self.render_bash_value(meta),
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::VarExpr(self)
    }
}
