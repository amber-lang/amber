use crate::utils::TranslateMetadata;

pub trait TranslateModule {
    fn translate(&self, meta: &mut TranslateMetadata) -> String;

    fn translate_eval(&self, meta: &mut TranslateMetadata) -> String {
        let prev = meta.eval_ctx;
        meta.eval_ctx = true;
        let expr = self.translate(meta);
        meta.eval_ctx = prev;
        expr
    }
}