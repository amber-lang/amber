use super::{handle_identifier_name, variable_name_extensions};
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::prelude::{
    BlockFragment, FragmentKind, RawFragment, TranslateModule, VarFragment,
};
use crate::modules::types::{Type, Typed};
use crate::translate::fragments::var::VarIndexValue;
use crate::translate::gen_intermediate_variable;
use crate::utils::{ParserMetadata, TranslateMetadata};
use heraclitus_compiler::prelude::*;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct VariableDefinition {
    name: String,
    tok: Option<Token>,
    global_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct VariableInit {
    definitions: Vec<VariableDefinition>,
    expr: Box<Expr>,
    is_global_ctx: bool,
    is_fun_ctx: bool,
    is_destructured: bool,
    is_const: bool,
}

impl VariableInit {
    fn handle_add_variable(
        &mut self,
        meta: &mut ParserMetadata,
        def: &mut VariableDefinition,
    ) -> SyntaxResult {
        handle_identifier_name(meta, &def.name, def.tok.clone())?;
        def.global_id = meta.add_var(&def.name, self.expr.get_type(), self.is_const);
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for VariableInit {
    syntax_name!("Variable Initialize");

    fn new() -> Self {
        VariableInit {
            definitions: vec![],
            expr: Box::new(Expr::new()),
            is_global_ctx: false,
            is_fun_ctx: false,
            is_destructured: false,
            is_const: false,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let keyword = token_by(meta, |word| ["let", "const"].contains(&word.as_str()))?;
        self.is_const = keyword == "const";

        let mut definitions: Vec<VariableDefinition> = vec![];

        match token(meta, "[") {
            Ok(_) => {
                self.is_destructured = true;
                let mut idx = 0;
                loop {
                    if token(meta, "]").is_ok() {
                        break;
                    }
                    if idx > 0 {
                        token(meta, ",")?;
                    }
                    let tok = meta.get_current_token();
                    let name = variable(meta, variable_name_extensions())?;
                    definitions.push(VariableDefinition {
                        name: name.clone(),
                        tok,
                        global_id: None,
                    });
                    idx += 1;
                }
            }
            Err(_) => {
                let tok = meta.get_current_token();
                let name = variable(meta, variable_name_extensions())?;
                definitions.push(VariableDefinition {
                    name: name.clone(),
                    tok,
                    global_id: None,
                });
            }
        }

        if definitions.is_empty() {
            panic!("Expected at least one variable definition");
        }

        context!(
            {
                token(meta, "=")?;
                syntax(meta, &mut *self.expr)?;

                if self.is_destructured && !self.expr.get_type().is_array() {
                    panic!("Expected array type for destructured variable");
                }

                for def in &mut definitions {
                    self.handle_add_variable(meta, def)?;
                }

                self.is_global_ctx = definitions.iter().any(|x| x.global_id.is_some());
                self.is_fun_ctx = meta.context.is_fun_ctx;
                self.definitions = definitions.clone();
                Ok(())
            },
            |position| {
                error_pos!(
                    meta,
                    position,
                    format!("Expected '=' after variable definition")
                )
            }
        )
    }
}

impl TranslateModule for VariableInit {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let expr = self.expr.translate(meta);

        if !self.is_destructured {
            let definition = self.definitions[0].clone();
            let (stmt, _var) = gen_intermediate_variable(&definition.name, definition.global_id, self.expr.get_type(), false, None, "=", expr);
            return stmt;
        }

        // TODO: Add warning when the count of definitions is not equal to the count of elements in the array, or the size of the array is unknown.

        let mut block = BlockFragment::new(vec![], false);

        let reference = if self.is_global_ctx {
            format!(
                "__ref_{}_{}",
                // all ids
                self.definitions
                    .iter()
                    .map(|x| x.global_id.unwrap())
                    .join("i_i"),
                // all names
                self.definitions.iter().map(|x| x.name.clone()).join("_")
            )
        } else {
            format!(
                "__ref_{}",
                // all names
                self.definitions.iter().map(|x| x.name.clone()).join("n_n")
            )
        };

        let (ref_stmt, _var) = gen_intermediate_variable(&reference, None, self.expr.get_type(), false, None, "=", expr);
        block.append(ref_stmt);

        let sub_type = match self.expr.clone().kind.clone() {
            Type::Array(expr) => expr,
            _ => Box::new(Type::Generic),
        };

        for (idx, def) in self.definitions.iter().enumerate() {
            let index = VarIndexValue::Index(FragmentKind::Raw(RawFragment::from(idx.to_string())));

            let mut var_fragment = VarFragment::new(&reference, *sub_type.clone(), false, None);
            var_fragment.index = Some(Box::new(index));

            let (stmt, _var) = gen_intermediate_variable(
                &def.name,
                def.global_id,
                *sub_type.clone(),
                false,
                None,
                "=",
                FragmentKind::Var(var_fragment),
            );
            block.append(stmt);
        }

        FragmentKind::Block(block)
    }
}

impl DocumentationModule for VariableInit {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
