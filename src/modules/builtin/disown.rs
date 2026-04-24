use crate::{fragments, raw_fragment};
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::fragments::var_stmt::VarStmtFragment;
use crate::utils::ParserMetadata;
use amber_meta::AutoKeyword;
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::syntax_name;

#[derive(Clone, Debug, AutoKeyword)]
#[keyword = "disown"]
#[kind = "builtin_stmt"]
pub struct Disown {
    pids: Option<Expr>,
}

impl SyntaxModule<ParserMetadata> for Disown {
    syntax_name!("disown");

    fn new() -> Self {
        Disown { pids: None }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "disown")?;
        token(meta, "(")?;

        if token(meta, ")").is_err() {
            let mut pids = Expr::new();
            syntax(meta, &mut pids)?;
            self.pids = Some(pids);
            token(meta, ")")?;
        }

        Ok(())
    }
}

impl TypeCheckModule for Disown {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        if let Some(ref mut pids) = self.pids {
            pids.typecheck(meta)?;

            let pids_type = pids.get_type();
            let expected_array_type = Type::array_of(Type::Int);
            if pids_type != Type::Int && !pids_type.is_allowed_in(&expected_array_type) {
                let position = pids.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `disown` can only be used with values of type Int or [Int]",
                    comment: format!("Given type: {}, expected type: {} or {}", pids_type, Type::Int, expected_array_type)
                });
            }
        }

        Ok(())
    }
}

impl TranslateModule for Disown {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let Some(pids) = &self.pids else {
            return fragments!("disown 2>/dev/null || true");
        };

        let pids_type = pids.get_type();
        let pids_var_stmt =
            VarStmtFragment::new("__disown_pids", pids_type.clone(), pids.translate(meta))
                .with_global_id(meta.gen_value_id())
                .with_optimization_when_unused(false);
        let pids_var = meta.push_ephemeral_variable(pids_var_stmt.clone());
        let pids_var_name = pids_var.get_name();

        let iter_value = if pids_type.is_array() {
            format!("\"${{{pids_var_name}[@]}}\"")
        } else {
            format!("\"${{{pids_var_name}}}\"")
        };

        let pid_var = format!("__AMBER_PID_{}", meta.gen_value_id());

        BlockFragment::new(vec![
            raw_fragment!("for {pid_var} in {iter_value}; do"),
            BlockFragment::new(
                vec![
                    raw_fragment!("disown ${pid_var} 2>/dev/null || true")
                ],
                true,
            ).to_frag(),
            raw_fragment!("done"),
        ], false).to_frag()
    }
}

impl DocumentationModule for Disown {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
