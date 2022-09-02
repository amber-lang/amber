use crate::translate::compute::ArithType;

pub struct TranslateMetadata {
    pub arith_module: ArithType,
    pub indent: usize
}

impl TranslateMetadata {
    pub fn new() -> Self {
        TranslateMetadata {
            arith_module: ArithType::BcSed,
            indent: 0
        }
    }

    pub fn gen_indent(&self) -> String {
        "  ".repeat(self.indent)
    }

    pub fn increase_indent(&mut self) {
        self.indent += 1;
    }

    pub fn decrease_indent(&mut self) {
        self.indent -= 1;
    }
}
