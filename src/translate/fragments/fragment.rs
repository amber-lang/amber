use super::{
    block::BlockFragment, comment::CommentFragment, interpolable::InterpolableFragment,
    list::ListFragment, log::LogFragment, raw::RawFragment, subprocess::SubprocessFragment,
    var_expr::VarExprFragment, var_stmt::VarStmtFragment,
};
use crate::{translate::fragments::arithmetic::ArithmeticFragment, utils::TranslateMetadata};

pub trait FragmentRenderable {
    fn to_string(self, meta: &mut TranslateMetadata) -> String;
    fn to_frag(self) -> FragmentKind;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum FragmentKind {
    Raw(RawFragment),
    VarExpr(VarExprFragment),
    VarStmt(VarStmtFragment),
    Block(BlockFragment),
    Interpolable(InterpolableFragment),
    List(ListFragment),
    Subprocess(SubprocessFragment),
    Arithmetic(ArithmeticFragment),
    Comment(CommentFragment),
    Log(LogFragment),
    #[default]
    Empty,
}

impl FragmentKind {
    pub fn with_quotes(self, value: bool) -> Self {
        match self {
            FragmentKind::VarExpr(var) => FragmentKind::VarExpr(var.with_quotes(value)),
            FragmentKind::Interpolable(inter) => {
                FragmentKind::Interpolable(inter.with_quotes(value))
            }
            FragmentKind::Subprocess(sub) => FragmentKind::Subprocess(sub.with_quotes(value)),
            FragmentKind::Arithmetic(arith) => FragmentKind::Arithmetic(arith.with_quotes(value)),
            _ => self,
        }
    }

    pub fn is_empty_logic(&self) -> bool {
        match self {
            FragmentKind::Empty => true,
            FragmentKind::Comment(_) => true,
            FragmentKind::Block(block) => block.is_empty_logic(),
            FragmentKind::List(list) => list.is_empty_logic(),
            FragmentKind::Log(log) => log.value.is_empty_logic(),
            _ => false,
        }
    }

    pub fn is_mutating(&self) -> bool {
        match self {
            FragmentKind::VarStmt(var_stmt) => var_stmt.value.is_mutating(),
            FragmentKind::Block(block) => block.statements.iter().any(|stmt| stmt.is_mutating()),
            FragmentKind::Interpolable(interpolable) => {
                interpolable.interps.iter().any(|item| item.is_mutating())
            }
            FragmentKind::List(list) => list.values.iter().any(|item| item.is_mutating()),
            FragmentKind::Arithmetic(arithmetic) => {
                arithmetic
                    .left
                    .as_ref()
                    .as_ref()
                    .is_some_and(|l| l.is_mutating())
                    || arithmetic
                        .right
                        .as_ref()
                        .as_ref()
                        .is_some_and(|r| r.is_mutating())
            }
            FragmentKind::Log(log) => log.value.is_mutating(),
            FragmentKind::Subprocess(_) => true,
            _ => false,
        }
    }

    pub fn is_running_command(&self) -> bool {
        match self {
            FragmentKind::VarStmt(var_stmt) => var_stmt.value.is_running_command(),
            FragmentKind::Block(block) => block
                .statements
                .iter()
                .any(|stmt| stmt.is_running_command()),
            FragmentKind::Interpolable(interpolable) => interpolable
                .interps
                .iter()
                .any(|item| item.is_running_command()),
            FragmentKind::List(list) => list.values.iter().any(|item| item.is_running_command()),
            FragmentKind::Arithmetic(arithmetic) => {
                arithmetic
                    .left
                    .as_ref()
                    .as_ref()
                    .is_some_and(|l| l.is_running_command())
                    || arithmetic
                        .right
                        .as_ref()
                        .as_ref()
                        .is_some_and(|r| r.is_running_command())
            }
            FragmentKind::Log(log) => log.value.is_running_command(),
            FragmentKind::Subprocess(_) => true,
            _ => false,
        }
    }
}

impl FragmentRenderable for FragmentKind {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        match self {
            FragmentKind::Raw(raw) => raw.to_string(meta),
            FragmentKind::VarExpr(var) => var.to_string(meta),
            FragmentKind::VarStmt(var) => var.to_string(meta),
            FragmentKind::Block(block) => block.to_string(meta),
            FragmentKind::Interpolable(interpolable) => interpolable.to_string(meta),
            FragmentKind::List(list) => list.to_string(meta),
            FragmentKind::Subprocess(subprocess) => subprocess.to_string(meta),
            FragmentKind::Arithmetic(arithmetic) => arithmetic.to_string(meta),
            FragmentKind::Comment(comment) => comment.to_string(meta),
            FragmentKind::Log(log) => log.to_string(meta),
            FragmentKind::Empty => String::new(),
        }
    }

    fn to_frag(self) -> FragmentKind {
        self
    }
}
