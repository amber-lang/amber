use crate::fragments;
use crate::modules::command::modifier::CommandModifier;
use crate::modules::condition::failure_handler::FailureHandler;
use crate::modules::expression::expr::Expr;

use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::raw_fragment;
use crate::translate::compute::translate_float_computation;
use crate::translate::fragments::var_expr::format_position;
use crate::utils::ParserMetadata;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone, AutoKeyword)]
#[keyword = "sleep"]
#[kind = "builtin_stmt"]
pub struct Sleep {
    value: Expr,
    modifier: CommandModifier,
    failure_handler: FailureHandler,
}

impl SyntaxModule<ParserMetadata> for Sleep {
    syntax_name!("Sleep");

    fn new() -> Self {
        Sleep {
            value: Expr::new(),
            modifier: CommandModifier::new_expr(),
            failure_handler: FailureHandler::new(),
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        syntax(meta, &mut self.modifier)?;
        self.modifier.use_modifiers(meta, |_, meta| {
            token(meta, "sleep")?;
            token(meta, "(")?;
            syntax(meta, &mut self.value)?;
            token(meta, ")")?;

            if let Err(e) = syntax(meta, &mut self.failure_handler) {
                match e {
                    Failure::Quiet(pos) => {
                        return error_pos!(meta, pos => {
                            message: "The `sleep` builtin can fail and requires explicit failure handling. Use '?', 'failed', 'succeeded', or 'exited' to manage its result.",
                            comment: "You can use '?' to propagate failure, 'failed' block to handle failure, 'succeeded' block to handle success, 'exited' block to handle both, or 'trust' modifier to ignore results"
                        });
                    }
                    _ => return Err(e),
                }
            }
            Ok(())
        })
    }
}

impl TypeCheckModule for Sleep {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.modifier.use_modifiers(meta, |_, meta| {
            self.value.typecheck(meta)?;
            let time_type = self.value.get_type();
            if time_type != Type::Int && time_type != Type::Num {
                let position = self.value.get_position();
                return error_pos!(meta, position => {
                    message: "Builtin function `sleep` can only be used with values of type Int or Num",
                    comment: format!("Given type: {}, expected type: {} or {}", time_type, Type::Int, Type::Num)
                });
            }
            self.failure_handler.typecheck(meta)?;
            Ok(())
        })
    }
}

impl TranslateModule for Sleep {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let value_translated = self.value.translate(meta);
        let id = meta.gen_value_id();
        let var_stmt = VarStmtFragment::new("__sleep_val", self.value.get_type(), value_translated)
            .with_global_id(id);
        let var_expr = meta.push_ephemeral_variable(var_stmt);
        let position = format_position(self.value.position.as_ref());
        let location = position.as_deref().unwrap_or("unknown");

        let handler = self.failure_handler.translate(meta);
        let silent = meta.with_silenced(self.modifier.is_silent || meta.silenced, |meta| {
            meta.gen_silent().to_frag()
        });
        let suppress = meta.with_suppress(self.modifier.is_suppress || meta.suppress, |meta| {
            meta.gen_suppress().to_frag()
        });

        let check = match self.value.get_type() {
            Type::Int => ArithmeticFragment::new(
                var_expr.clone().with_quotes(false).to_frag(),
                ArithOp::Ge,
                raw_fragment!("0"),
            )
            .to_frag(),
            _ => translate_float_computation(
                meta,
                ArithOp::Ge,
                Some(var_expr.clone().to_frag()),
                Some(raw_fragment!("0")),
            ),
        };

        let sleep_cmd = ListFragment::new(
            vec![
                raw_fragment!("sleep"),
                var_expr.to_frag(),
                silent.clone(),
                suppress.clone()
            ]
        )
        .with_spaces()
        .to_frag();

        BlockFragment::new(
            vec![
                fragments!("if [ ", check, " != 0 ]; then"),
                BlockFragment::new(vec![sleep_cmd], true).to_frag(),
                raw_fragment!("else"),
                BlockFragment::new(
                    vec![
                        fragments!(
                            RawFragment::from(format!(
                                "echo \"Sleep value needs to be >= 0 (at {location})\" >&2"
                            ))
                            .to_frag(),
                            silent,
                            suppress
                        ),
                        raw_fragment!("false"),
                    ],
                    true,
                )
                .to_frag(),
                raw_fragment!("fi"),
                handler,
            ],
            false,
        )
        .to_frag()
    }
}

impl DocumentationModule for Sleep {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
