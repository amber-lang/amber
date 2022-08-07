use heraclitus_compiler::prelude::*;

pub fn get_rules() -> Rules {
    let symbols = vec![
        '+', '-', '*', '/',
        '(', ')', '[', ']', '{', '}'
    ];
    let compounds = vec![
        ('<', '='),
        ('>', '='),
        ('!', '='),
        ('=', '='),
    ];
    let region = reg![
        reg!(string as "string literal" => {
            begin: "'",
            end: "'"
        } => [
            reg!(interp as "string interpolation" => {
                begin: "{",
                end: "}",
                tokenize: true
            } ref global)
        ])
    ];
    Rules::new(symbols, compounds, region)
}