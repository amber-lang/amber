use super::{
    block::BlockFragment,
    comment::CommentFragment,
    compound::CompoundFragment,
    eval::EvalFragment,
    interpolable::InterpolableFragment,
    list::ListFragment,
    raw::RawFragment,
    subprocess::SubprocessFragment,
    var::VarFragment,
};
use crate::utils::TranslateMetadata;

pub trait TranslationFragmentable {
    fn to_string(self, meta: &mut TranslateMetadata) -> String;
    fn to_frag(self) -> TranslationFragment;
}

#[derive(Debug, Clone)]
pub enum TranslationFragment {
    Raw(RawFragment),
    Var(VarFragment),
    Block(BlockFragment),
    Compound(CompoundFragment),
    Interpolable(InterpolableFragment),
    List(ListFragment),
    Eval(EvalFragment),
    Subprocess(SubprocessFragment),
    Comment(CommentFragment),
    Empty
}

impl TranslationFragment {
    pub fn unquote(self) -> Self {
        match self {
            TranslationFragment::Var(var) => TranslationFragment::Var(var.set_quoted(false)),
            TranslationFragment::Interpolable(inter) => TranslationFragment::Interpolable(inter.set_quoted(false)),
            TranslationFragment::Subprocess(sub) => TranslationFragment::Subprocess(sub.set_quoted(false)),
            _ => self,
        }
    }

    pub fn is_empty_logic(&self) -> bool {
        match self {
            TranslationFragment::Empty => true,
            TranslationFragment::Comment(_) => true,
            TranslationFragment::Block(block) => block.is_empty_logic(),
            TranslationFragment::List(list) => list.is_empty_logic(),
            TranslationFragment::Compound(compound) => compound.is_empty_logic(),
            _ => false,
        }
    }
}

impl TranslationFragmentable for TranslationFragment {
    fn to_string(self, meta: &mut TranslateMetadata) -> String {
        match self {
            TranslationFragment::Raw(raw) => raw.to_string(meta),
            TranslationFragment::Var(var) => var.to_string(meta),
            TranslationFragment::Block(block) => block.to_string(meta),
            TranslationFragment::Compound(statement) => statement.to_string(meta),
            TranslationFragment::Interpolable(interpolable) => interpolable.to_string(meta),
            TranslationFragment::List(list) => list.to_string(meta),
            TranslationFragment::Eval(eval) => eval.to_string(meta),
            TranslationFragment::Subprocess(subprocess) => subprocess.to_string(meta),
            TranslationFragment::Empty => String::new(),
            TranslationFragment::Comment(comment) => comment.to_string(meta),
        }
    }

    fn to_frag(self) -> TranslationFragment {
        self
    }
}
