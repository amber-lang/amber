use heraclitus_compiler::prelude::*;

pub fn get_rules() -> Rules {
    let symbols = vec![
        '+', '-', '*', '/',
        '(', ')', '[', ']', '{', '}'
    ];
    let region = reg![];
    Rules::new(symbols, region)
}