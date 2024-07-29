#[macro_export]
macro_rules! parse_operator {
    ($meta:expr, $start_index:expr, $name: ident $block:tt) => {{
        use heraclitus_compiler::prelude::*;
        let mut module = $name $block;
        syntax($meta, &mut module)?;
        let end_index = $meta.get_index();
        Expr {
            kind: module.get_type(),
            value: Some (ExprType:: $name (module)),
            pos: ($start_index, end_index)
        }
    }}
}

#[macro_export]
macro_rules! parse_non_operators {
    ($meta:expr, [$($item:ident),*]) => {(|| {
        let start_index = $meta.get_index();
        $(
            let mut module = $item::new();
            match syntax($meta, &mut module) {
                Ok(()) => {
                    let end_index = $meta.get_index();
                    return Ok(Expr {
                        kind: module.get_type(),
                        value: Some(ExprType::$item(module)),
                        pos: (start_index, end_index)
                    })
                },
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
