pub mod access;
pub mod binop;
pub mod expr;
pub mod interpolated_region;
pub mod literal;
pub mod macros;
pub mod parentheses;
pub mod ternop;
pub mod typeop;
pub mod unop;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoolAnalysis {
    pub known_value: Option<bool>,
    /// Whether the known value depends on the selected target shell
    pub depends_on_target: bool,
    /// Whether evaluating this expression may contain side effects
    /// Function calls, commands need to be preserved as they can mutate the environment
    pub has_side_effects: bool,
}

impl BoolAnalysis {
  pub fn with_side_effects() -> Self {
    BoolAnalysis {
      known_value: None,
      depends_on_target: false,
      has_side_effects: true
    }
  }
}
