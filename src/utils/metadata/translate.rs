use crate::{translate::compute::ArithType, utils::memory::Memory};
use super::ParserMetadata;

pub struct TranslateMetadata {
    pub arith_module: ArithType,
    pub mem: Memory,
    pub indent: i64
}

impl TranslateMetadata {
    pub fn new(meta: &ParserMetadata) -> Self {
        TranslateMetadata {
            arith_module: ArithType::BcSed,
            mem: meta.mem.clone(),
            indent: -1
        }
    }

    pub fn gen_indent(&self) -> String {
        "    ".repeat(self.indent as usize)
    }

    pub fn increase_indent(&mut self) {
        self.indent += 1;
    }

    pub fn decrease_indent(&mut self) {
        self.indent -= 1;
    }
}
