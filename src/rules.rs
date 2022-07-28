use heraclitus_compiler::prelude::*;

pub fn get_rules() -> Rules {
    let symbols = vec![
        '+', '-', '*', '/',
        '(', ')', '[', ']', '{', '}'
    ];
    let region = reg![
        reg!(string as "string literal" => {
            begin: "'",
            end: "'"
        })
    ];
    Rules::new(symbols, region)
}