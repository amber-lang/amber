#[macro_export]
macro_rules! parse_operator {
    ($meta:expr, $name: ident $block:tt) => {{
        let mut syntax = $name $block;
        syntax.parse($meta)?;
        Expr {
            kind: syntax.get_type(),
            value: Some (ExprType:: $name (syntax)),
        }
    }}
}

#[macro_export]
macro_rules! parse_non_operators {
    ($meta:expr, [$($item:ident),*]) => {(|| {
        $(
            let mut syntax = $item::new();
            match syntax.parse($meta) {
                Ok(()) => return Ok(Expr {
                    kind: syntax.get_type(),
                    value: Some(ExprType::$item(syntax)),
                }),
                Err(failure) => {
                    if let Failure::Loud(err) = failure {
                        return Err(Failure::Loud(err))
                    }
                }
            }
        )*
        error!($meta, $meta.get_current_token(), "Expected expression")
    })()}
}

#[macro_export]
macro_rules! translate_expression {
    ($meta:expr, $value:expr, [$($item:ident),*]) => {
        match $value {
            $(
                ExprType::$item(module) => module.translate($meta)
            ),*
        }
    }
}

#[macro_export]
macro_rules! document_expression {
    ($meta:expr, $value:expr, [$($item:ident),*]) => {
        match $value {
            $(
                ExprType::$item(module) => module.document($meta)
            ),*
        }
    }
}
