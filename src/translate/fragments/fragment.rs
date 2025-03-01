use super::{
    block::BlockFragment, compound::CompoundFragment, eval::EvalFragment, interpolable::InterpolableFragment, list::ListFragment, raw::RawFragment, subprocess::SubprocessFragment, var::VarFragment
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
    Eval(EvalFragment),
    Subprocess(SubprocessFragment),
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
            TranslationFragment::Eval(eval) => eval.render(meta),
            TranslationFragment::Subprocess(subprocess) => subprocess.render(meta),
            TranslationFragment::Empty => String::new(),
        }
    }

    fn to_frag(self) -> TranslationFragment {
        self
    }
}
