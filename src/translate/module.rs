use crate::utils::TranslateMetadata;

pub trait TranslateModule {
    fn translate(&self, meta: &mut TranslateMetadata) -> String;

    fn translate_eval(&self, meta: &mut TranslateMetadata, is_eval: bool) -> String {
        let prev = meta.eval_ctx;
        meta.eval_ctx = is_eval;
        let expr = self.translate(meta);
        meta.eval_ctx = prev;
        expr
    }
}
