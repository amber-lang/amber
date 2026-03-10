use crate::eval_context;
use crate::modules::prelude::*;
use crate::modules::types::Type;
use crate::utils::ShellType;

use super::get_variable_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarStmtFragment {
    pub name: String,
    pub global_id: Option<usize>,
    pub index: Option<Box<FragmentKind>>,
    pub kind: Type,
    // This variable is made only for storing a value used by a single expression
    pub is_ephemeral: bool,
    pub is_ref: bool,
    pub is_local: bool,
    // a "workaround" to properly generate array declaration by copying the origin array
    pub is_array_ref: bool,
    pub is_declared: bool,
    // Determines if the variable can be removed when not used
    pub optimize_unused: bool,
    pub operator: String,
    pub value: Box<FragmentKind>,
}

// Represents Bash variable operation such as `=`, `+=`, etc.

impl Default for VarStmtFragment {
    fn default() -> Self {
        VarStmtFragment {
            name: String::new(),
            global_id: None,
            index: None,
            kind: Type::Generic,
            is_ephemeral: false,
            is_ref: false,
            is_local: false,
            is_array_ref: false,
            is_declared: true,
            optimize_unused: true,
            operator: "=".to_string(),
            value: Box::new(FragmentKind::Empty),
        }
    }
}

impl VarStmtFragment {
    pub fn new(name: &str, kind: Type, value: FragmentKind) -> Self {
        VarStmtFragment {
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

    pub fn with_declared(mut self, is_declared: bool) -> Self {
        self.is_declared = is_declared;
        self
    }

    pub fn with_ephemeral(mut self, is_ephemeral: bool) -> Self {
        self.is_ephemeral = is_ephemeral;
        self
    }

    pub fn with_local(mut self, is_local: bool) -> Self {
        self.is_local = is_local;
        self
    }

    pub fn with_array_ref(mut self, is_array_ref: bool) -> Self {
        self.is_array_ref = is_array_ref;
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

    pub fn render_variable_name(&self, meta: &mut TranslateMetadata) -> String {
        let variable = self.get_name();

        if matches!(meta.target.shell, ShellType::Zsh) && self.is_ref && self.is_declared {
            format!("${{{variable}}}")
        } else {
            variable.to_string()
        }
    }

    fn render_variable_statement(self, meta: &mut TranslateMetadata) -> String {
        let var_name = self.render_variable_name(meta);
        let is_running_command = self.value.is_running_command();
        let mut assignment_parts = vec![];
        let value = self.value.to_string(meta);
        assignment_parts.push(var_name.clone());
        assignment_parts.extend(
            self.index
                .map(|index| format!("[{}]", index.to_string(meta))),
        );
        assignment_parts.push(self.operator);
        // it only adds () to the value IF the variable has already been declared, because otherwise namerefs fail with () syntax
        if self.kind.is_array() && !self.is_array_ref {
            assignment_parts.push(format!("({})", value.clone()));
        } else {
            assignment_parts.push(value.clone());
        }
        let assignment = assignment_parts.join("");
        match meta.target.shell {
            ShellType::Bash => {
                // `local` command consumes exit code of command that it is assigned to.
                // To preserve the exit code of the assignment we split the local declaration into two parts.
                if self.is_local {
                    if is_running_command {
                        format!("local {var_name}\n{}{assignment}", meta.gen_indent())
                    } else if self.is_ref {
                        format!("local -n {assignment}")
                    } else {
                        format!("local {assignment}")
                    }
                } else {
                    assignment
                }
            }
            ShellType::Zsh => {
                if self.is_local {
                    if is_running_command {
                        format!("local {var_name}\n{}{assignment}", meta.gen_indent())
                    } else {
                        format!("local {assignment}")
                    }
                } else {
                    assignment
                }
            }
            ShellType::Ksh => {
                if self.is_local && is_running_command {
                    format!("typeset {var_name}\n{}{assignment}", meta.gen_indent())
                } else if self.is_local {
                    // In ksh93, `arr=()` becomes a literal `()` value, and `typeset -a arr`
                    // does not clear an existing value. `typeset -a arr=()` is required to
                    // create a real empty array while preserving overwrite semantics.
                    if self.kind.is_array() && value.is_empty() {
                        format!("typeset -a {assignment}")
                    } else if self.is_ref {
                        format!("typeset -n {assignment}")
                    } else if self.kind.is_array() {
                        // ksh function-local arrays are required for recursive array operations to keep
                        // each frame isolated once the argument has been rebound through a nameref.
                        format!("typeset {assignment}")
                    } else {
                        format!("typeset {assignment}")
                    }
                } else if self.kind.is_array() && value.is_empty() {
                    format!("typeset -a {assignment}")
                } else {
                    assignment
                }
            }
        }
    }
}

impl FragmentRenderable for VarStmtFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        match meta.target.shell {
            // if array is a direct reference and is already declared, use eval to modify directly
            ShellType::Zsh => {
                if self.is_ref && self.is_declared {
                    let stmt =
                        eval_context!(meta, self.is_ref, { self.render_variable_statement(meta) });
                    format!("eval \"{stmt}\"")
                } else {
                    self.render_variable_statement(meta)
                }
            }
            ShellType::Ksh => {
                if self.kind.is_array() && !self.is_declared {
                    let stmt =
                        eval_context!(meta, self.is_ref, { self.render_variable_statement(meta) });
                    format!("eval \"{stmt}\"")
                } else {
                    self.render_variable_statement(meta)
                }
            }
            ShellType::Bash => self.render_variable_statement(meta),
        }
    }

    fn to_frag(self) -> FragmentKind {
        FragmentKind::VarStmt(self)
    }
}
