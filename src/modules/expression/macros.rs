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
                        module.set_right($prev($meta)?);
                        module.set_left(node);
                        syntax($meta, &mut module)?;
                        let end_index = $meta.get_index();
                        node = Expr {
                            kind: module.get_type(),
                            value: Some(ExprType::$cur_modules(module)),
                            pos: (start_index, end_index)
                        };
                        continue
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
                        module.set_right(parse_type($meta)?);
                        syntax($meta, &mut module)?;
                        let end_index = $meta.get_index();
                        node = Expr {
                            kind: module.get_type(),
                            value: Some(ExprType::$cur_modules(module)),
                            pos: (start_index, end_index)
                        };
                        continue
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
                        let middle = $cur($meta)?;
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
    (@internal ({$cur:ident, $prev:ident}, $meta:expr, UnOp => [$($cur_modules:ident),+])) => {{
        let start_index = $meta.get_index();
        $({
            let mut module = $cur_modules::new();
            match module.parse_operator($meta) {
                Ok(()) => {
                    module.set_expr($cur($meta)?);
                    syntax($meta, &mut module)?;
                    return Ok(Expr {
                        kind: module.get_type(),
                        value: Some(ExprType::$cur_modules(module)),
                        pos: (start_index, $meta.get_index())
                    })
                },
                Err(Failure::Quiet(_)) => {},
                Err(Failure::Loud(err)) => return Err(Failure::Loud(err))
            }
        })*
        $prev($meta)
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
        $next_name:ident @ $next_type:ident => [$($next_modules:ident),*]
    )) => {
        fn _terminal(_meta: &mut ParserMetadata) -> Result<Expr, Failure> {
            panic!("Please create a group that ends precedence recurrence");
        }

        fn $next_name(meta: &mut ParserMetadata) -> Result<Expr, Failure> {
            parse_expr_group!(@internal (
                {$next_name, _terminal},
                meta, $next_type => [$($next_modules),*]
            ))
        }

        fn $cur_name(meta: &mut ParserMetadata) -> Result<Expr, Failure> {
            parse_expr_group!(@internal (
                {$cur_name, $next_name},
                meta, $cur_type => [$($cur_modules),*]
            ))
        }
    };

    // Recursive step: Current, previous and the rest
    (@internal (
        $cur_name:ident @ $cur_type:ident => [$($cur_modules:ident),*],
        $next_name:ident @ $next_type:ident => [$($next_modules:ident),*],
        $($rest_name:ident @ $rest_type:ident => [$($rest_modules:ident),*]),+
    )) => {
        parse_expr!(@internal (
            $next_name @ $next_type => [$($next_modules),*],
            $($rest_name @ $rest_type => [$($rest_modules),*]),*)
        );

        fn $cur_name (meta: &mut ParserMetadata) -> Result<Expr, Failure> {
            parse_expr_group!(@internal (
                {$cur_name, $next_name},
                meta, $cur_type => [$($cur_modules),*]
            ))
        }
    };

    // Public interface:
    // parse_expr!(meta, [
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
    }};

    // Edge case: Single group provided
    // parse_expr!(meta, [
    //     name @ Literal => [Num, Text],
    // ]);
    ($meta:expr, [
        $name:ident @ $type:ident => [$($modules:ident),*]
    ]) => {{
        fn _terminal(_meta: &mut ParserMetadata) -> Result<Expr, Failure> {
            panic!("Please create a group that ends precedence recurrence");
        }

        fn $name(meta: &mut ParserMetadata) -> Result<Expr, Failure> {
            parse_expr_group!(@internal (
                {$name, _terminal},
                meta, $type => [$($modules),*]
            ))
        }

        $name($meta)?
    }};
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
