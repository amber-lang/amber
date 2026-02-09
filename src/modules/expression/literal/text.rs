use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::interpolated_region::{
    parse_interpolated_region, InterpolatedRegionType,
};
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::module::TranslateModule;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub struct Text {
    strings: Vec<String>,
    interps: Vec<Expr>,
}

impl Typed for Text {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Text {
    syntax_name!("Text");

    fn new() -> Self {
        Text {
            strings: vec![],
            interps: vec![],
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        (self.strings, self.interps) =
            parse_interpolated_region(meta, &InterpolatedRegionType::Text)?;
        Ok(())
    }
}

impl TypeCheckModule for Text {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        // Type check all interpolated expressions
        for expr in &mut self.interps {
            expr.typecheck(meta)?;
        }
        Ok(())
    }
}

impl TranslateModule for Text {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        // Translate all interpolations
        let interps = self
            .interps
            .iter()
            .map(|item| {
                let frag = item.translate(meta).with_quotes(false);
                // ShellCheck SC2145: Use [*] for array interpolation to treat array as single string argument
                if let FragmentKind::VarExpr(mut var) = frag {
                    if var.kind.is_array() {
                        var = var.with_array_to_string(true);
                    }
                    var.to_frag()
                } else {
                    frag
                }
            })
            .collect::<Vec<FragmentKind>>();
        InterpolableFragment::new(
            self.strings.clone(),
            interps,
            InterpolableRenderType::StringLiteral,
        )
        .to_frag()
    }
}

impl DocumentationModule for Text {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
