use super::{
    block::BlockFragment,
    comment::CommentFragment,
    eval::EvalFragment,
    interpolable::InterpolableFragment,
    list::ListFragment,
    raw::RawFragment,
    subprocess::SubprocessFragment,
    var::VarFragment,
};
use crate::utils::TranslateMetadata;

pub trait FragmentRenderable {
    fn to_string(self, meta: &mut TranslateMetadata) -> String;
    fn to_frag(self) -> FragmentKind;
}

#[derive(Debug, Clone)]
pub enum FragmentKind {
    Raw(RawFragment),
    Var(VarFragment),
    Block(BlockFragment),
    Interpolable(InterpolableFragment),
    List(ListFragment),
    Eval(EvalFragment),
    Subprocess(SubprocessFragment),
    Comment(CommentFragment),
    Empty
}

impl FragmentKind {
    pub fn with_quotes(self, value: bool) -> Self {
        match self {
            FragmentKind::Var(var) => FragmentKind::Var(var.with_quotes(value)),
            FragmentKind::Interpolable(inter) => FragmentKind::Interpolable(inter.with_quotes(value)),
            FragmentKind::Subprocess(sub) => FragmentKind::Subprocess(sub.with_quotes(value)),
            _ => self,
        }
    }

    pub fn is_empty_logic(&self) -> bool {
        match self {
            FragmentKind::Empty => true,
            FragmentKind::Comment(_) => true,
            FragmentKind::Block(block) => block.is_empty_logic(),
            FragmentKind::List(list) => list.is_empty_logic(),
            _ => false,
        }
    }
}

impl FragmentRenderable for FragmentKind {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        match self {
            FragmentKind::Raw(raw) => raw.to_string(meta),
            FragmentKind::Var(var) => var.to_string(meta),
            FragmentKind::Block(block) => block.to_string(meta),
            FragmentKind::Interpolable(interpolable) => interpolable.to_string(meta),
            FragmentKind::List(list) => list.to_string(meta),
            FragmentKind::Eval(eval) => eval.to_string(meta),
            FragmentKind::Subprocess(subprocess) => subprocess.to_string(meta),
            FragmentKind::Comment(comment) => comment.to_string(meta),
            FragmentKind::Empty => String::new(),
        }
    }

    fn to_frag(self) -> FragmentKind {
        self
    }
}
