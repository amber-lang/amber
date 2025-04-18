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
    pub operator: String,
    pub value: Box<FragmentKind>,
}

impl Default for VarStmtFragment {
    fn default() -> Self {
        Self {
            name: String::new(),
            global_id: None,
            index: None,
            kind: Type::Generic,
            is_ref: false,
            operator: "=".to_string(),
            value: Box::new(FragmentKind::Empty),
        }
    }
}

impl VarStmtFragment {
    fn render_variable_name(&self) -> String {
        let variable = get_variable_name(&self.name, self.global_id);

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
