pub mod stmt;
pub mod comment;
pub mod comment_doc;

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
    ($meta:expr, $stmt_type:expr, [$($stmt:ident),*]) => {
        match $stmt_type {
            $(
                StmtType::$stmt(stmt) => stmt.translate($meta)?,
            )*
        }
    };
}
