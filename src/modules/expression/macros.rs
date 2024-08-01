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
macro_rules! parse_expr_group {
    // Group type that handles Binary Operators
    (@internal ({$cur:ident, $prev:ident}, $meta:expr, BinOp => [$($cur_modules:ident),+])) => {{
        let start_index = $meta.get_index();
        let mut node = $prev($meta)?;
        loop {
            $({
                let mut module = $cur_modules::new();
                match module.parse_operator($meta) {
                    Ok(()) => {
                        module.set_left(node);
                        module.set_right($prev($meta)?);
                        syntax($meta, &mut module)?;
                        let end_index = $meta.get_index();
                        node = Expr {
                            kind: module.get_type(),
                            value: Some(ExprType::$cur_modules(module)),
                            pos: (start_index, end_index)
                        };
                    }
                    Err(Failure::Quiet(_)) => {}
                    Err(Failure::Loud(err)) => return Err(Failure::Loud(err))
                }
            })*
            break
        }
        Ok(node)
    }};

    // Group type that handles Type Operators
    (@internal ({$cur:ident, $prev:ident}, $meta:expr, TypeOp => [$($cur_modules:ident),+])) => {{
        let start_index = $meta.get_index();
        let mut node = $prev($meta)?;
        loop {
            $({
                let mut module = $cur_modules::new();
                match module.parse_operator($meta) {
                    Ok(()) => {
                        module.set_left(node);
                        module.set_right(parse_type(meta)?);
                        syntax($meta, &mut module)?;
                        let end_index = $meta.get_index();
                        node = Expr {
                            kind: module.get_type(),
                            value: Some(ExprType::$cur_modules(module)),
                            pos: (start_index, end_index)
                        };
                    }
                    Err(Failure::Quiet(_)) => {}
                    Err(Failure::Loud(err)) => return Err(Failure::Loud(err))
                }
            })*
            break
        }
        Ok(node)
    }};

    // Group type that handles Ternary Operators
    (@internal ({$cur:ident, $prev:ident}, $meta:expr, TernOp => [$($cur_modules:ident),+])) => {{
        let start_index = $meta.get_index();
        let mut node = $prev($meta)?;
        loop {
            $({
                let mut module = $cur_modules::new();
                match module.parse_operator_left($meta) {
                    Ok(()) => {
                        module.set_left(node);
                        let mut middle = $cur($meta)?;
                        module.parse_operator_right($meta)?;
                        module.set_middle(middle);
                        module.set_right($cur($meta)?);
                        syntax($meta, &mut module)?;
                        let end_index = $meta.get_index();
                        node = Expr {
                            kind: module.get_type(),
                            value: Some(ExprType::$cur_modules(module)),
                            pos: (start_index, end_index)
                        };
                    }
                    Err(Failure::Quiet(_)) => {}
                    Err(Failure::Loud(err)) => return Err(Failure::Loud(err))
                }
            })*
            break
        }
        Ok(node)
    }};

    // Group type that handles Literals. Use this group as the last one in the precedence order
    (@internal ({$cur:ident, $prev:ident}, $meta:expr, Literal => [$($cur_modules:ident),+])) => {{
        let start_index = $meta.get_index();
        $({
            let mut module = $cur_modules::new();
            match syntax($meta, &mut module) {
                Ok(()) => return Ok(Expr {
                    kind: module.get_type(),
                    value: Some(ExprType::$cur_modules(module)),
                    pos: (start_index, $meta.get_index())
                }),
                Err(Failure::Quiet(_)) => {},
                Err(Failure::Loud(err)) => return Err(Failure::Loud(err))
            }
        })*
        Err(Failure::Quiet(PositionInfo::from_metadata($meta)))
    }};
}

#[macro_export]
macro_rules! parse_expr {
    // Base Case: Current and previous precedence groups remaining
    (@internal (
        $cur_name:ident @ $cur_type:ident => [$($cur_modules:ident),*],
        $prev_name:ident @ $prev_type:ident => [$($prev_modules:ident),*]
    )) => {
        let _terminal = |_meta: &mut ParserMetadata| -> Result<Expr, Failure> {
            panic!("Please create a group that ends precedence recurrence");
        };

        let $prev_name = |meta: &mut ParserMetadata| -> Result<Expr, Failure> {
            parse_expr_group!(@internal (
                {$prev_name, _terminal},
                meta, $prev_type => [$($prev_modules),*]
            ))
        };

        let $cur_name = |meta: &mut ParserMetadata| -> Result<Expr, Failure> {
            parse_expr_group!(@internal (
                {$cur_name, $prev_name},
                meta, $cur_type => [$($cur_modules),*]
            ))
        };
    };

    // Recursive step: Current, previous and the rest
    (@internal (
        $cur_name:ident @ $cur_type:ident => [$($cur_modules:ident),*],
        $prev_name:ident @ $prev_type:ident => [$($prev_modules:ident),*],
        $($rest_name:ident @ $rest_type:ident => [$($rest_modules:ident),*]),+
    )) => {
        parse_expr!(@internal (
            $prev_name @ $prev_type => [$($prev_modules),*],
            $($rest_name @ $rest_type => [$($rest_modules),*]),*)
        );

        let $cur_name = |meta: &mut ParserMetadata| -> Result<Expr, Failure> {
            parse_expr_group!(@internal (
                {$cur_name, $prev_name},
                meta, $cur_type => [$($cur_modules),*]
            ))
        };
    };

    // Public interface:
    // parse_expr!([
    //     name @ TernOp => [Ternary],
    //     name @ BinOp => [Add, Sub],
    //     name @ BinOp => [Mul, Div],
    //     name @ TypeOp => [As, Cast],
    //     name @ UnOp => [Neg, Not],
    //     name @ Literal => [Num, Text],
    // ]);
    ($meta:expr, [
        $name:ident @ $type:ident => [$($modules:ident),*],
        $($rest_name:ident @ $rest_type:ident => [$($rest_modules:ident),*]),+
    ]) => {{
        parse_expr!(@internal (
            $name @ $type => [$($modules),*],
            $($rest_name @ $rest_type => [$($rest_modules),*]),*
        ));

        $name($meta)?
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
