use crate::fragments;
use crate::raw_fragment;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use crate::utils::metadata::{ParserMetadata, TranslateMetadata};
use crate::modules::prelude::*;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct LinesInvocation {
    path: Box<Option<Expr>>,
}

impl Typed for LinesInvocation {
    fn get_type(&self) -> Type {
        Type::array_of(Type::Text)
    }
}

impl SyntaxModule<ParserMetadata> for LinesInvocation {
    syntax_name!("Lines Invocation");

    fn new() -> Self {
        LinesInvocation {
            path: Box::new(None),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "lines")?;
        token(meta, "(")?;
        let mut path = Expr::new();
        syntax(meta, &mut path)?;
        token(meta, ")")?;
        *self.path = Some(path);
        Ok(())
    }
}

impl TypeCheckModule for LinesInvocation {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Some(path) = &mut *self.path {
            path.typecheck(meta)?;
            if path.get_type() != Type::Text {
                let msg = format!(
                    "Expected value of type 'Text' but got '{}'",
                    path.get_type()
                );
                let pos = path.get_position();
                return error_pos!(meta, pos, msg);
            }
            Ok(())
        } else {
          unreachable!()
        }
    }
}

impl TranslateModule for LinesInvocation {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let temp = format!("__AMBER_LINE_{}", meta.gen_value_id());
        let path = (*self.path)
            .as_ref()
            .map(|p| p.translate(meta))
            .expect("Cannot read lines without provided path");
        let indent = TranslateMetadata::single_indent();
        let id = meta.gen_value_id();
        let var_stmt = VarStmtFragment::new("__array", Type::array_of(Type::Text), FragmentKind::Empty).with_global_id(id);
        let var_expr = meta.push_ephemeral_variable(var_stmt);
        meta.stmt_queue.extend([
            raw_fragment!("while IFS= read -r {temp}; do"),
            raw_fragment!("{indent}{}+=(\"${}\")", var_expr.get_name(), temp),
            fragments!("done <", path),
        ]);
        var_expr.to_frag()
    }
}

impl LinesInvocation {
    pub fn translate_path(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        (*self.path)
            .as_ref()
            .map(|p| p.translate(meta))
            .expect("Cannot read lines without provided path in iterator loop")
    }
}

impl DocumentationModule for LinesInvocation {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
