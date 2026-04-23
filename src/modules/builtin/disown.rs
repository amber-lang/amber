use crate::fragments;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::utils::ParserMetadata;
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::syntax_name;

#[derive(Clone, Debug, AutoKeyword)]
#[keyword = "disown"]
#[kind = "builtin_stmt"]
pub struct Disown {
    jobs: Vec<Expr>,
}

impl Typed for Disown {
    fn get_type(&self) -> Type {
        Type::Null
    }
}

impl SyntaxModule<ParserMetadata> for Disown {
    syntax_name!("disown");

    fn new() -> Self {
        Disown { jobs: Vec::new() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let tok = meta.get_current_token();
        token(meta, "disown")?;

        if token(meta, "(").is_ok() {
            let mut job = Expr::new();
            syntax(meta, &mut job)?;
            self.jobs.push(job);
            while token(meta, ",").is_ok() {
                let mut next_job = Expr::new();
                syntax(meta, &mut next_job)?;
                self.jobs.push(next_job);
            }
            token(meta, ")")?;
        } else {
            let warning = Message::new_warn_at_token(meta, tok)
                .message("Calling `disown` without parentheses is deprecated")
                .comment("Use `disown(job)` instead");
            meta.add_message(warning);

            let mut job = Expr::new();
            syntax(meta, &mut job)?;
            self.jobs.push(job);
        }

        Ok(())
    }
}

impl TypeCheckModule for Disown {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        for job in &mut self.jobs {
            job.typecheck(meta)?;
            if job.get_type() != Type::Int {
                let pos = job.get_position();
                return error_pos!(meta, pos => {
                    message: "disown job argument must be an integer",
                    comment: format!("Found {} instead", job.get_type())
                });
            }
        }
        Ok(())
    }
}

impl TranslateModule for Disown {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        if self.jobs.is_empty() {
            fragments!("disown")
        } else {
            let mut result = fragments!("disown ");
            for (i, job) in self.jobs.iter().enumerate() {
                if i > 0 {
                    result = fragments!(result, " ");
                }
                result = fragments!(result, job.translate(meta));
            }
            result
        }
    }
}

impl DocumentationModule for Disown {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
