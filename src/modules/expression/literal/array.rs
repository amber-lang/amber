use heraclitus_compiler::prelude::*;
use crate::modules::expression::expr::Expr;
use crate::modules::types::{try_parse_type, Type, Typed};
use crate::modules::prelude::*;

#[derive(Debug, Clone)]
pub struct Array {
    exprs: Vec<Expr>,
    kind: Type
}

impl Typed for Array {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Array {
    syntax_name!("Array");

    fn new() -> Self {
        Array {
            exprs: vec![],
            kind: Type::Generic
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "[")?;
        let tok = meta.get_current_token();
        if token(meta, "]").is_ok() {
            self.kind = Type::Array(Box::new(Type::Generic));
            return Ok(())
        }
        // Try to parse array type
        match try_parse_type(meta) {
            Ok(kind) => {
                if matches!(kind, Type::Array(_)) {
                    return error!(meta, tok, "Arrays cannot be nested due to the Bash limitations")
                }
                self.kind = Type::Array(Box::new(kind));
                token(meta, "]")?;
            },
            Err(Failure::Loud(err)) => {
                return Err(Failure::Loud(err))
            },
            // Parse the array values
            Err(Failure::Quiet(_)) => {
                loop {
                    // Skip comments and newlines
                    if token_by(meta, |token| token.starts_with("//") || token.starts_with('\n')).is_ok() {
                        continue;
                    }
                    if token(meta, "]").is_ok() {
                        break;
                    }
                    // Parse array value
                    let mut value = Expr::new();
                    syntax(meta, &mut value)?;
                    let tok = meta.get_current_token();
                    if token(meta, "]").is_ok() {
                        self.exprs.push(value);
                        break;
                    }
                    if token(meta, ",").is_ok() {
                        self.exprs.push(value);
                        continue;
                    }
                    return error!(meta, tok, "Expected ',' or ']' after array value");
                }
            }
        };
        Ok(())
    }
}

impl TypeCheckModule for Array {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // First type-check all the expressions
        for expr in &mut self.exprs {
            expr.typecheck(meta)?;
            // Handle nested arrays
            if expr.get_type().is_array() {
                let pos = expr.get_position();
                return error_pos!(meta, pos, "Arrays cannot be nested due to the Bash limitations")
            }
        }

        // Then determine the array type
        if self.exprs.is_empty() {
            // Empty array keeps its existing type (from explicit type annotation or default)
            return Ok(());
        }

        match self.kind {
            Type::Generic => {
                // Infer type from first element
                self.kind = Type::Array(Box::new(self.exprs[0].get_type()));
            },
            Type::Array(ref expected_type) => {
                // Type already specified, validate all elements match
                for expr in &self.exprs {
                    let expr_type = expr.get_type();
                    if expr_type != **expected_type {
                        let pos = expr.get_position();
                        return error_pos!(meta, pos, format!("Expected array value of type '{expected_type}'"))
                    }
                }
            },
            _ => unimplemented!("Unexpected array type state {0}.", self.kind)
        }

        // Validate all elements have the same type
        if let Type::Array(ref element_type) = self.kind {
            for expr in &self.exprs[1..] {
                let expr_type = expr.get_type();
                if expr_type != **element_type {
                    let pos = expr.get_position();
                    return error_pos!(meta, pos, format!("Array elements must have the same type. Expected '{}', found '{}'", element_type, expr_type));
                }
            }
        }

        Ok(())
    }
}

impl TranslateModule for Array {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let id = meta.gen_value_id();
        let args = self.exprs.iter().map(|expr| expr.translate_eval(meta, false)).collect::<Vec<FragmentKind>>();
        let args = ListFragment::new(args).with_spaces().to_frag();
        let var_stmt = VarStmtFragment::new("array", self.kind.clone(), args).with_global_id(id);
        meta.push_ephemeral_variable(var_stmt).to_frag()
    }
}

impl DocumentationModule for Array {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
