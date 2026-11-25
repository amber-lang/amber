use heraclitus_compiler::{
    error,
    prelude::{Failure, Message, Token},
};

use crate::utils::ParserMetadata;

pub mod block;
pub mod builtin;
pub mod command;
pub mod condition;
pub mod expression;
pub mod function;
pub mod imports;
pub mod keywords;
pub mod loops;
pub mod main;
pub mod prelude;
pub mod shorthand;
pub mod statement;
pub mod test;
pub mod typecheck;
pub mod types;
pub mod variable;

pub fn handle_symbol_scope_declaration(
    meta: &mut ParserMetadata,
    name: &str,
    tok: Option<Token>,
) -> Result<(), Failure> {
    if meta.get_function_in_current_scope(name).is_some()
        || meta.get_var_in_current_scope(name).is_some()
    {
        return error!(
            meta,
            tok,
            format!("Cannot redeclare '{}' in the same scope", name)
        );
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FlowSummary {
    pub terminates: bool,
    pub reachable_break: bool,
}

impl FlowSummary {
    pub fn then(self, next: impl FnOnce() -> Self) -> Self {
        if self.terminates {
            self
        } else {
            let rhs = next();
            Self {
                terminates: rhs.terminates,
                reachable_break: self.reachable_break || rhs.reachable_break,
            }
        }
    }

    pub fn join(self, other: Self) -> Self {
        Self {
            terminates: self.terminates && other.terminates,
            reachable_break: self.reachable_break || other.reachable_break,
        }
    }
}

pub trait ControlFlow {
    fn flow(&self) -> FlowSummary;
}

impl<T: ControlFlow> ControlFlow for Box<T> {
    fn flow(&self) -> FlowSummary {
        (**self).flow()
    }
}

impl<T: ControlFlow> ControlFlow for Option<T> {
    fn flow(&self) -> FlowSummary {
        self.as_ref().map_or_else(FlowSummary::default, ControlFlow::flow)
    }
}

impl<T: ControlFlow> ControlFlow for Vec<T> {
    fn flow(&self) -> FlowSummary {
        self.as_slice().flow()
    }
}

impl<T: ControlFlow> ControlFlow for [T] {
    fn flow(&self) -> FlowSummary {
        let mut acc = FlowSummary::default();
        for item in self {
            acc = acc.then(|| item.flow());
            if acc.terminates {
                return acc;
            }
        }
        acc
    }
}

#[macro_export]
macro_rules! impl_flow {
    ($ty:ty $(, $field:ident)*) => {
        impl $crate::modules::ControlFlow for $ty {
            fn flow(&self) -> $crate::modules::FlowSummary {
                let acc = $crate::modules::FlowSummary::default();
                $(
                    let acc = acc.then(|| $crate::modules::ControlFlow::flow(&self.$field));
                )*
                acc
            }
        }
    };
}
