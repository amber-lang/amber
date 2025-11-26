pub mod stmt;
pub mod comment;
pub mod comment_doc;

#[macro_export]
macro_rules! init_statement {
    ([$($stmt:ident),*]) => {
        vec![
            $(
                StmtType::$stmt($stmt::new()),
            )*
        ]
    };
}

#[macro_export]
macro_rules! parse_statement {
    ([$($stmt:ident),*], |$module:ident, $cons:ident| $body:expr) => {{
        let mut error = None;
        $(
            let mut $module = $stmt::new();
            let $cons = StmtType::$stmt;
            match $body {
                Ok(()) => return Ok(()),
                Err(failure) => {
                    match failure {
                        Failure::Loud(err) => return Err(Failure::Loud(err)),
                        Failure::Quiet(err) => error = Some(err)
                    }
                }
            }
        )*
        Err(Failure::Quiet(error.unwrap()))
    }};
}

#[macro_export]
macro_rules! typecheck_statement {
    ($meta:expr, $stmt_type:expr, [$($stmt:ident),*]) => {
        match $stmt_type {
            $(
                StmtType::$stmt(stmt) => stmt.typecheck($meta)?,
            )*
        }
    };
}

#[macro_export]
macro_rules! translate_statement {
    ($stmt_type:expr, [$($stmt:ident),*], |$var:ident| $body:expr) => {
        match $stmt_type {
            $(
                StmtType::$stmt($var) => $body,
            )*
        }
    };
}

#[macro_export]
macro_rules! document_statement {
    ($stmt_type:expr, [$($stmt:ident),*], $var:ident, $body:expr) => {
        match $stmt_type {
            $(
                StmtType::$stmt($var) => $body,
            )*
        }
    };
}
