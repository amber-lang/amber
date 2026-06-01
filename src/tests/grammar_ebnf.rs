use crate::utils::grammar_ebnf::generate_grammar_ebnf;

#[test]
fn grammar_generates_without_panic() {
    let grammar = generate_grammar_ebnf();
    assert!(!grammar.is_empty(), "Grammar should not be empty");
    assert!(
        grammar.contains("(* Keywords - auto-generated at build time *)"),
        "Should contain keyword section"
    );
    assert!(
        grammar.contains("(* Builtins *)"),
        "Should contain builtin section"
    );
}

#[test]
fn grammar_contains_root_and_blocks() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("root = { statement_global } ;"));
    assert!(g.contains("block = singleline_block | multiline_block ;"));
    assert!(g.contains("singleline_block = ':', statement_local ;"));
    assert!(g.contains("multiline_block = '{', { statement_local }, '}' ;"));
}

#[test]
fn grammar_contains_all_statement_rules() {
    let g = generate_grammar_ebnf();
    let expected = [
        "statement_local",
        "statement_global",
        "loop_array",
        "loop_array_iterator",
        "while_loop",
        "variable_init_const",
        "variable_init_mut",
        "variable_init_destruct",
        "variable_set",
        "variable_set_destruct",
        "shorthand_add",
        "shorthand_sub",
        "shorthand_mul",
        "shorthand_div",
        "shorthand_modulo",
        "break",
        "continue",
        "return_stmt",
        "fail",
        "comment_doc",
        "command_modifier_block",
    ];
    for rule in &expected {
        assert!(
            g.contains(rule),
            "Statement rule '{rule}' not found in grammar"
        );
    }
}

#[test]
fn grammar_contains_all_expression_rules() {
    let g = generate_grammar_ebnf();
    let expected = [
        "expression",
        "binary_operation",
        "unary_operation",
        "expression_index",
        "cast",
        "is",
        "status",
        "function_call",
        "function_call_failed",
    ];
    for rule in &expected {
        assert!(
            g.contains(rule),
            "Expression rule '{rule}' not found in grammar"
        );
    }
}

#[test]
fn grammar_contains_type_operations() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("cast = expression, KEYWORD_AS, TYPE ;"));
    assert!(g.contains("is = expression, KEYWORD_IS, TYPE ;"));
}

#[test]
fn grammar_contains_variable_destructuring() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("variable_init_destruct"));
    assert!(g.contains("variable_set_destruct"));
    assert!(g.contains("( KEYWORD_LET | KEYWORD_CONST ), '[', identifier"));
}

#[test]
fn grammar_contains_shorthand_arithmetic() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("shorthand_add"));
    assert!(g.contains("shorthand_sub"));
    assert!(g.contains("shorthand_mul"));
    assert!(g.contains("shorthand_div"));
    assert!(g.contains("shorthand_modulo"));
    assert!(g.contains("'+='"));
    assert!(g.contains("'-='"));
    assert!(g.contains("'*='"));
    assert!(g.contains("'/='"));
    assert!(g.contains("'%='"));
}

#[test]
fn grammar_contains_loop_control() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("break = KEYWORD_BREAK ;"));
    assert!(g.contains("continue = KEYWORD_CONTINUE ;"));
}

#[test]
fn grammar_contains_return_and_fail() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("return_stmt = KEYWORD_RETURN, expression ;"));
    assert!(g.contains("fail = KEYWORD_FAIL, [ expression ] ;"));
}

#[test]
fn grammar_contains_function_def_with_ref_and_failable() {
    let g = generate_grammar_ebnf();
    assert!(
        g.contains("KEYWORD_REF"),
        "function_def should include ref keyword"
    );
    assert!(
        g.contains("[ '?' ]"),
        "function_def should include optional failable marker"
    );
    assert!(
        !g.contains("function_def_typed"),
        "Old function_def_typed should be merged into function_def"
    );
}

#[test]
fn grammar_contains_main_with_failable() {
    let g = generate_grammar_ebnf();
    assert!(
        g.contains("main = KEYWORD_MAIN, [ '(', identifier, ')' ], [ '?' ], block ;"),
        "main should support optional failable marker"
    );
}

#[test]
fn grammar_contains_status_expression() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("status = KEYWORD_STATUS, [ '(', ')' ] ;"));
}

#[test]
fn grammar_contains_documentation_comment() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("comment_doc = '///', { ANY_CHAR } ;"));
}

#[test]
fn grammar_contains_all_keywords() {
    let g = generate_grammar_ebnf();
    let key_keywords = [
        "KEYWORD_REF = 'ref'",
        "KEYWORD_RETURN = 'return'",
        "KEYWORD_FAIL = 'fail'",
        "KEYWORD_STATUS = 'status'",
        "KEYWORD_AS = 'as'",
        "KEYWORD_IS = 'is'",
        "KEYWORD_BREAK = 'break'",
        "KEYWORD_CONTINUE = 'continue'",
        "KEYWORD_FOR = 'for'",
    ];
    for kw in &key_keywords {
        assert!(g.contains(kw), "Missing keyword definition: {kw}");
    }
}

#[test]
fn grammar_contains_builtins() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("builtins_statement ="));
    assert!(g.contains("builtins_expression ="));
    assert!(g.contains("builtin_await"));
    assert!(g.contains("builtin_echo"));
    assert!(g.contains("builtin_pid"));
    assert!(g.contains("builtin_shellname"));
    assert!(g.contains("builtin_shellversion"));
    assert!(g.contains("builtin_len"));
}

#[test]
fn grammar_contains_tests() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("test = KEYWORD_TEST, [ test_name ], block ;"));
    assert!(g.contains("test_name ="));
}

#[test]
fn grammar_contains_all_terminal_definitions() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("LETTER = 'A'..'Z' | 'a'..'z' ;"));
    assert!(g.contains("DIGIT = '0'..'9' ;"));
    assert!(g.contains("TYPE = SIMPLE_TYPE, { '|', SIMPLE_TYPE } ;"));
    assert!(g.contains("SIMPLE_TYPE = 'Text' | 'Num' | 'Bool' | 'Null' | 'Int' | '[', TYPE, ']' ;"));
}

#[test]
fn grammar_contains_imports() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("import_all"));
    assert!(g.contains("import_ids"));
    assert!(g.contains("KEYWORD_FROM"));
    assert!(g.contains("KEYWORD_AS"));
}

#[test]
fn grammar_contains_conditionals() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("if_statement"));
    assert!(g.contains("if_chain"));
    assert!(g.contains("ternary"));
    assert!(g.contains("KEYWORD_ELSE"));
    assert!(g.contains("KEYWORD_THEN"));
}

#[test]
fn grammar_contains_handlers() {
    let g = generate_grammar_ebnf();
    assert!(g.contains("failure_handler"));
    assert!(g.contains("success_handler"));
    assert!(g.contains("exited_handler"));
    assert!(g.contains("handler"));
    assert!(g.contains("failure_propagation"));
    assert!(g.contains("KEYWORD_FAILED"));
    assert!(g.contains("KEYWORD_SUCCEEDED"));
    assert!(g.contains("KEYWORD_EXITED"));
}

#[test]
fn grammar_contains_command_modifier_block() {
    let g = generate_grammar_ebnf();
    assert!(
        g.contains("command_modifier_block"),
        "command_modifier_block rule must exist in grammar"
    );
    assert!(g.contains("command_modifier = [ KEYWORD_SILENT"));
    assert!(g.contains("KEYWORD_TRUST"));
    assert!(g.contains("KEYWORD_SUDO"));
}
