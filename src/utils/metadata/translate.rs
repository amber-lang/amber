use crate::translate::compute::ArithType;

pub struct TranslateMetadata {
    pub arith_module: ArithType
}

impl TranslateMetadata {
    pub fn new() -> Self {
        TranslateMetadata {
            arith_module: ArithType::BasicCalculator
        }
    }
}
