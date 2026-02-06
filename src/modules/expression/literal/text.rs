use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::interpolated_region::{
    parse_interpolated_region, InterpolatedRegionType,
};
use crate::modules::prelude::*;
use crate::modules::types::{Type, Typed};
use crate::translate::fragments::interpolable::InterpolablePart;
use crate::translate::module::TranslateModule;
use heraclitus_compiler::prelude::*;

#[derive(Debug, Clone)]
pub enum TextPart {
    String(String),
    Expr(Box<Expr>),
}

impl TextPart {
    pub fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        match self {
            TextPart::String(_) => Ok(()),
            TextPart::Expr(expr) => expr.typecheck(meta),
        }
    }

    /// Converts TextParts to InterpolableParts for translation
    pub fn to_interpolable_parts(
        parts: &[TextPart],
        meta: &mut TranslateMetadata,
    ) -> Vec<InterpolablePart> {
        parts
            .iter()
            .map(|part| match part {
                TextPart::String(s) => InterpolablePart::String(s.clone()),
                TextPart::Expr(expr) => {
                    let frag = expr.translate(meta).with_quotes(false);
                    if let FragmentKind::VarExpr(mut var) = frag {
                        if var.kind.is_array() {
                            var = var.with_array_to_string(true);
                        }
                        InterpolablePart::Interp(var.to_frag())
                    } else {
                        InterpolablePart::Interp(frag)
                    }
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    parts: Vec<TextPart>,
}

impl Typed for Text {
    fn get_type(&self) -> Type {
        Type::Text
    }
}

impl SyntaxModule<ParserMetadata> for Text {
    syntax_name!("Text");

    fn new() -> Self {
        Text { parts: vec![] }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.parts = parse_interpolated_region(meta, &InterpolatedRegionType::Text)?;
        Ok(())
    }
}

impl TypeCheckModule for Text {
    fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        for part in &mut self.parts {
            part.typecheck(meta)?;
        }
        Ok(())
    }
}

impl TranslateModule for Text {
    fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
        let parts = TextPart::to_interpolable_parts(&self.parts, meta);
        InterpolableFragment::new(parts, InterpolableRenderType::StringLiteral).to_frag()
    }
}

impl DocumentationModule for Text {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
