use super::{
    block::BlockFragment,
    compound::CompoundFragment,
    interpolable::InterpolableFragment,
    list::ListFragment,
    raw::RawFragment,
    var::VarFragment,
};
use crate::utils::TranslateMetadata;

pub trait TranslationFragmentable {
    fn render(self, meta: &mut TranslateMetadata) -> String;
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
    Empty
}

impl TranslationFragment {
    pub fn unquote(self) -> Self {
        match self {
            TranslationFragment::Var(var) => TranslationFragment::Var(var.set_quoted(false)),
            _ => self,
        }
    }
}

impl TranslationFragmentable for TranslationFragment {
    fn render(self, meta: &mut TranslateMetadata) -> String {
        match self {
            TranslationFragment::Raw(raw) => raw.render(meta),
            TranslationFragment::Var(var) => var.render(meta),
            TranslationFragment::Block(block) => block.render(meta),
            TranslationFragment::Compound(statement) => statement.render(meta),
            TranslationFragment::Interpolable(interpolable) => interpolable.render(meta),
            TranslationFragment::List(list) => list.render(meta),
            TranslationFragment::Empty => String::new(),
        }
    }

    fn to_frag(self) -> TranslationFragment {
        self
    }
}
