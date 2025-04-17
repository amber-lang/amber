use crate::modules::types::Type;
use crate::modules::prelude::*;

use super::get_variable_name;

#[derive(Debug, Clone)]
pub struct VarStmtFragment {
    name: String,
    global_id: Option<usize>,
    index: Option<Box<FragmentKind>>,
    kind: Type,
    is_ref: bool,
    operator: String,
    value: Box<FragmentKind>,
}

impl VarStmtFragment {
    pub fn new(name: &str, kind: Type, value: FragmentKind) -> Self {
        Self {
            name: name.to_string(),
            global_id: None,
            index: None,
            kind,
            is_ref: false,
            operator: "=".to_string(),
            value: Box::new(value),
        }
    }

    pub fn with_global_id(mut self, global_id: usize) -> Self {
        self.global_id = Some(global_id);
        self
    }

    pub fn with_op(mut self, op: &str) -> Self {
        self.operator = op.to_string();
        self
    }

    pub fn with_ref(mut self, is_ref: bool) -> Self {
        self.is_ref = is_ref;
        self
    }

    pub fn with_index(mut self, index: FragmentKind) -> Self {
        self.index = Some(Box::new(index));
        self
    }

    fn render_variable_name(&self) -> String {
        let variable = get_variable_name(&self.name, self.global_id);

        if self.is_ref {
            format!("${{{}}}", variable)
        } else {
            format!("{}", variable)
        }
    }
}

impl FragmentRenderable for VarStmtFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        let stmt = {
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
        };

        if self.is_ref {
            format!("eval \"{stmt}\"")
        } else {
            stmt
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::VarStmt(self)
    }
}
