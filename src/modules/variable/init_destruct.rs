use heraclitus_compiler::prelude::*;

use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::modules::expression::expr::Expr;
use crate::raw_fragment;
use crate::translate::fragments::var_expr::VarIndexValue;
use crate::modules::variable::get_default_value_fragment;
use super::{variable_name_extensions, handle_identifier_name};
use crate::utils::context::{VariableDecl, VariableDeclWarn};
use crate::utils::metadata::ParserMetadata;

#[derive(Debug, Clone)]
pub struct VariableInitDestruct {
    names: Vec<String>,
    expr: Box<Expr>,
    global_ids: Vec<Option<usize>>,
    is_fun_ctx: bool,
    is_const: bool,
    toks: Vec<Option<Token>>,
}

impl SyntaxModule<ParserMetadata> for VariableInitDestruct {
    syntax_name!("Variable Initialize Destructuring");

    fn new() -> Self {
        VariableInitDestruct {
            names: vec![],
            expr: Box::new(Expr::new()),
            global_ids: vec![],
            is_fun_ctx: false,
            is_const: false,
            toks: vec![],
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        let keyword = token_by(meta, |word| ["let", "const"].contains(&word.as_str()))?;
        self.is_const = keyword == "const";
        
        token(meta, "[")?;
        loop {
            let tok = meta.get_current_token();
            let name = variable(meta, variable_name_extensions())?;
            self.names.push(name);
            self.toks.push(tok);

            if token(meta, ",").is_err() {
                break;
            }
        }
        if let Err(err) = token(meta, "]") {
            return error_pos!(meta, err.unwrap_quiet(), format!("Expected ']' after destructuring '{}'", self.names.join(", ")))
        }
        if let Err(err) = token(meta, "=") {
            return error_pos!(meta, err.unwrap_quiet(), format!("Expected '=' after destructuring '{}'", self.names.join(", ")))
        }
        syntax(meta, &mut *self.expr)?;
        self.is_fun_ctx = meta.context.is_fun_ctx;
        Ok(())
    }
}

impl TypeCheckModule for VariableInitDestruct {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.expr.typecheck(meta)?;
        
        // Ensure the expression is an array of known type
        let inner_type = match self.expr.get_type() {
            Type::Array(inner) if *inner == Type::Generic => {
                let pos = self.expr.get_position();
                return error_pos!(meta, pos => {
                    message: "Cannot destructure array because its concrete type is unknown",
                    comment: "Please add an explicit type annotation to this array value before destructuring"
                });
            },
            Type::Array(inner) => *inner,
            _ => {
                let pos = self.expr.get_position();
                return error_pos!(meta, pos, format!("Destructuring initialization requires an array type, but received '{}'", self.expr.get_type()));
            }
        };
        
        for (name, tok) in self.names.iter().zip(self.toks.iter()) {
            handle_identifier_name(meta, name, tok.clone())?;
            let var = VariableDecl::new(name.clone(), inner_type.clone())
                .with_warn(VariableDeclWarn::from_token(meta, tok.clone())
                    .warn_when_unmodified(!self.is_const && !meta.is_global_scope())
                    .warn_when_unused(!meta.is_global_scope()))
                .with_const(self.is_const);
            self.global_ids.push(meta.add_var(var));
        }
        
        Ok(())
    }
}

impl TranslateModule for VariableInitDestruct {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let expr = self.expr.translate(meta);
        
        let mut fragments = vec![];

        // Assign expression to temp array
        let temp_array_name = format!("array_destruct_{}", meta.gen_value_id());
        let assign_temp = VarStmtFragment::new(&temp_array_name, self.expr.get_type(), expr)
            .with_local(self.is_fun_ctx)
            .with_optimization_when_unused(false);
        fragments.push(assign_temp.clone().to_frag());
        
        let inner_type = match self.expr.get_type() {
            Type::Array(t) => *t,
            _ => unreachable!("Type of expression is not an array in init destructuring"), 
        };
        
        for (i, name) in self.names.iter().enumerate() {
            let assign_expr = VarExprFragment::from_stmt(&assign_temp)
                .with_index_by_value(VarIndexValue::Index(raw_fragment!("{i}")))
                .with_default_value(get_default_value_fragment(&inner_type))
                .to_frag();
            
            let assign_var = VarStmtFragment::new(name, inner_type.clone(), assign_expr)
                .with_global_id(self.global_ids[i])
                .with_local(self.is_fun_ctx)
                .to_frag();
            fragments.push(assign_var);
        }
        
        BlockFragment::new(fragments, false).to_frag()
    }
}

impl DocumentationModule for VariableInitDestruct {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
