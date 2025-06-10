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
            kind: Type::Null
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "[")?;
        let tok = meta.get_current_token();
        if token(meta, "]").is_ok() {
            return error!(meta, tok, "Expected array type or value before ']'", "Eg. insert 'Num' for empty array or '1, 2, 3' for array with values")
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
                    let tok = meta.get_current_token();
                    if token(meta, "[").is_ok() {
                        return error!(meta, tok, "Arrays cannot be nested due to the Bash limitations")
                    }
                    if token(meta, "]").is_ok() {
                        break;
                    }
                    // Parse array value
                    let mut value = Expr::new();
                    syntax(meta, &mut value)?;
                    match self.kind {
                        Type::Null => self.kind = Type::Array(Box::new(value.get_type())),
                        Type::Array(ref mut kind) => {
                            if value.get_type() != **kind {
                                return error!(meta, tok, format!("Expected array value of type '{kind}'"))
                            }
                        },
                        _ => ()
                    }
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

impl TranslateModule for Array {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let id = meta.gen_value_id();
        let args = self.exprs.iter().map(|expr| expr.translate_eval(meta, false)).collect::<Vec<FragmentKind>>();
        let args = ListFragment::new(args).with_spaces().to_frag();
        let var_stmt = VarStmtFragment::new("__array", self.kind.clone(), args).with_global_id(id);
        meta.push_intermediate_variable(var_stmt).to_frag()
    }
}

impl DocumentationModule for Array {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
