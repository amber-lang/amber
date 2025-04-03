use crate::modules::prelude::*;
use crate::{modules::{expression::expr::{Expr, ExprType}, prelude::RawFragment, types::Type}, utils::TranslateMetadata};
use super::fragment::{FragmentKind, FragmentRenderable};

/// Represents a variable expression such as `$var` or `${var}`
#[derive(Debug, Clone)]
pub enum VarRenderType {
    Name,
    BashRef,
    BashValue,
}

#[derive(Debug, Clone)]
pub enum VarIndexValue {
    Range(FragmentKind, FragmentKind),
    Index(FragmentKind)
}

#[derive(Debug, Clone)]
pub struct VarFragment {
    pub name: String,
    pub global_id: Option<usize>,
    pub kind: Type,
    pub is_ref: bool,
    pub is_length: bool,
    pub is_quoted: bool,
    pub render_type: VarRenderType,
    pub index: Box<Option<VarIndexValue>>,
}

impl VarFragment {
    pub fn new(name: &str, kind: Type, is_ref: bool, global_id: Option<usize>) -> Self {
        VarFragment {
            name: name.to_string(),
            global_id,
            kind,
            is_ref,
            is_quoted: true,
            is_length: false,
            render_type: VarRenderType::BashValue,
            index: Box::new(None),
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
            self.index = Box::new(Some(index));
        }
        self
    }

    pub fn set_get_length(mut self) -> Self {
        self.is_length = true;
        self
    }

    pub fn set_render_type(mut self, render_type: VarRenderType) -> Self {
        self.render_type = render_type;
        self
    }

    pub fn with_quotes(mut self, is_quoted: bool) -> Self {
        self.is_quoted = is_quoted;
        self
    }

    pub fn get_name(&self) -> String {
        match self.global_id {
            Some(id) => format!("__{id}_{}", self.name.trim_start_matches("__")),
            None => self.name.to_string()
        }
    }

    // Returns the variable name in the bash context Ex. "varname"
    pub fn render_bash_reference(self, meta: &mut TranslateMetadata) -> String {
        let dollar = meta.gen_dollar();
        let name = self.get_name();
        // Dereference variable if it's a reference and is passed by reference
        let result = if self.is_ref { format!("{dollar}{name}") } else { name };

        if !self.is_quoted {
            result
        } else {
            meta.gen_quote().to_string() + &result + meta.gen_quote()
        }
    }

    // Returns the variable value in the bash context Ex. "$varname" or "${varname[@]}"
    pub fn render_bash_value(mut self, meta: &mut TranslateMetadata) -> String {
        let quote = if self.is_quoted { meta.gen_quote() } else { "" };
        let dollar = meta.gen_dollar();
        let name = self.get_name();
        let index = self.index.take();

        let prefix = self.get_variable_prefix(self.is_length);
        let suffix = self.get_variable_suffix(meta, index);

        if self.is_ref {
            return self.render_deref_variable(meta, prefix, &name, &suffix);
        }

        match self.kind {
            Type::Text | Type::Array(_) => {
                format!("{quote}{dollar}{{{prefix}{name}{suffix}}}{quote}")
            }
            _ => {
                format!("{quote}{dollar}{{{prefix}{name}{suffix}}}{quote}")
            }
        }
    }

    // Get variable prefix ${PREFIX:varname:suffix}
    fn get_variable_prefix(&self, is_length: bool) -> &'static str {
        if is_length {
            "#"
        } else {
            ""
        }
    }

    // Get variable suffix ${prefix:varname:SUFFIX}
    fn get_variable_suffix(&self, meta: &mut TranslateMetadata, index: Option<VarIndexValue>) -> String {
        match (&self.kind, index) {
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
        meta.stmt_queue.push_back(RawFragment::new(
            &format!("eval \"local {var_name}={arr_open}\\\"\\${{{eval_value}}}\\\"{arr_close}\"")
        ).to_frag());

        if self.kind.is_array() {
            format!("{quote}{dollar}{{{var_name}[@]}}{quote}")
        } else {
            format!("{quote}{dollar}{{{var_name}}}{quote}")
        }
    }
}

impl FragmentRenderable for VarFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        match self.render_type {
            VarRenderType::Name => self.get_name(),
            VarRenderType::BashRef => self.render_bash_reference(meta),
            VarRenderType::BashValue => self.render_bash_value(meta),
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::Var(self)
    }
}
