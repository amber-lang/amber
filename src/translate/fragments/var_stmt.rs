use crate::eval_context;
use crate::modules::types::Type;
use crate::modules::prelude::*;

use super::get_variable_name;

#[derive(Debug, Clone)]
pub struct VarStmtFragment {
    pub name: String,
    pub global_id: Option<usize>,
    pub index: Option<Box<FragmentKind>>,
    pub kind: Type,
    pub is_ref: bool,
    pub optimize_unused: bool,
    pub operator: String,
    pub value: Box<FragmentKind>,
}

// Represents Bash variable operation such as `=`, `+=`, etc.

impl Default for VarStmtFragment {
    fn default() -> Self {
        Self {
            name: String::new(),
            global_id: None,
            index: None,
            kind: Type::Generic,
            is_ref: false,
            optimize_unused: true,
            operator: "=".to_string(),
            value: Box::new(FragmentKind::Empty),
        }
    }
}

impl VarStmtFragment {
    pub fn new(name: &str, kind: Type, value: FragmentKind) -> Self {
        Self {
            name: name.to_string(),
            kind,
            value: Box::new(value),
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

    pub fn with_index<T: Into<Option<FragmentKind>>>(mut self, index: T) -> Self {
        self.index = index.into().map(Box::new);
        self
    }

    pub fn with_operator(mut self, op: &str) -> Self {
        self.operator = op.to_string();
        self
    }

    pub fn with_optimization_when_unused(mut self, optimize: bool) -> Self {
        self.optimize_unused = optimize;
        self
    }

    pub fn get_name(&self) -> String {
        get_variable_name(&self.name, self.global_id)
    }

    pub fn render_variable_name(&self) -> String {
        let variable = self.get_name();

        if self.is_ref {
            format!("${{{}}}", variable)
        } else {
            variable.to_string()
        }
    }

    fn render_variable_statement(self, meta: &mut TranslateMetadata) -> String {
        let mut frags = vec![];
        frags.push(self.render_variable_name());
        frags.extend(self.index.map(|index| format!("[{}]", index.to_string(meta))));
        frags.push(self.operator);
        if self.kind.is_array() {
            frags.push(format!("({})", self.value.to_string(meta)));
        } else {
            frags.push(self.value.to_string(meta));
        }
        frags.join("")
    }
}

impl FragmentRenderable for VarStmtFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        if self.is_ref {
            let stmt = eval_context!(meta, self.is_ref, {
                self.render_variable_statement(meta)
            });
            format!("eval \"{stmt}\"")
        } else {
            self.render_variable_statement(meta)
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::VarStmt(self)
    }
}
