pub mod comment;
pub mod comment_doc;
pub mod stmt;

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
