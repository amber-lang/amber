use crate::modules::keywords::{iter_keywords, KeywordKind};
use std::collections::HashSet;

const GRAMMAR_EBNF: &str = r#"(*
    This is a basic grammar declaration for Amber written in EBNF.
    This syntax does not include features that are not fully stable yet.
*)

root = { statement_global } ;

(* Statement *)
statement_local =
    builtins_statement |
    expression |
    if_chain |
    if_statement |
    loop |
    loop_array |
    while_loop |
    variable_init_const |
    variable_init_mut |
    variable_set ;
statement_global =
    function_def |
    import_all |
    import_ids |
    main |
    test |
    statement_local ;

(* Block *)
singleline_block = ':', statement_local ;
multiline_block = '{', { statement_local }, '}' ;
block = singleline_block | multiline_block ;

(* Expression *)
expression =
    binary_operation |
    boolean |
    builtins_expression |
    command |
    function_call |
    function_call_failed |
    identifier |
    array |
    null |
    number |
    parentheses |
    range |
    range_inclusive |
    ternary |
    text |
    unary_operation |
    expression_index ;

    (* Keywords - auto-generated at build time *)

(* Terminals *)
ANY_CHAR = ? any character ? ;
LETTER = 'A'..'Z' | 'a'..'z' ;
DIGIT = '0'..'9' ;
TYPE = SIMPLE_TYPE, { '|', SIMPLE_TYPE } ;
SIMPLE_TYPE = 'Text' | 'Num' | 'Bool' | 'Null' | 'Int' | '[', TYPE, ']' ;
UNARY_OP = '-' | KEYWORD_NOT ;
BINARY_OP = '+' | '-' | '*' | '/' | '%' | KEYWORD_AND | KEYWORD_OR | '==' | '!=' | '<' | '<=' | '>' | '>=' ;
VISIBILITY = KEYWORD_PUB ;

(* Identifier *)
any_identifier = (LETTER | '_'), { LETTER | '_' | DIGIT } ;
internal_identifier = '__', { LETTER | '_' | DIGIT } ;
identifier = any_identifier - internal_identifier ;

(* `Num` literal *)
integer = DIGIT, { DIGIT } ;
real = integer, '.', integer ;
number = integer | real ;

(* `Text` literal *)
interpolation = '{', expression, '}' ;
text = '"', { ANY_CHAR | interpolation }, '"' ;

(* `Bool` literal *)
boolean = 'true' | 'false' ;

(* `Null` literal *)
null = 'null' ;

(* `Array` literal *)
empty_array = '[', [ TYPE ], ']' ;
full_array = '[', [ expression, { ',', expression } ], ']' ;
array = empty_array | full_array ;

(* Command expression *)
(* The ordering of command modifiers doesn't matter *)
command_modifier = [ KEYWORD_SILENT | KEYWORD_SUPPRESS ], [ KEYWORD_TRUST ], [ KEYWORD_SUDO ] ;
command_modifier_block = command_modifier, multiline_block ;
command_base = '$', { ANY_CHAR | interpolation }, '$' ;
command = command_modifier, command_base, [ handler ] ;

(* Operations *)
binary_operation = expression, BINARY_OP, expression ;
unary_operation = UNARY_OP, expression ;
expression_index = expression, '[', expression, ']' ;

(* Parentheses *)
parentheses = '(', expression, ')' ;

(* Failure handler *)
failure_propagation = '?';
failure_block = KEYWORD_FAILED, [ '(', identifier, ')' ], block ;
failure_handler = failure_propagation | failure_block ;

(* Success handler *)
success_block = KEYWORD_SUCCEEDED, block ;
success_handler = success_block ;

(* Exited handler *)
exited_block = KEYWORD_EXITED, [ '(', identifier, ')' ], block ;
exited_handler = exited_block ;

(* All handlers combined *)
handler = success_handler | failure_handler | exited_handler ;

(* Attributes *)
attribute_name = any_identifier ;
attribute = '#[', attribute_name, ']' ;

(* Variable *)
variable_index = '[', expression, ']' ;
variable_init_mut = { attribute }, [ VISIBILITY ], KEYWORD_LET, identifier, '=', expression ;
variable_init_const = { attribute }, [ VISIBILITY ], KEYWORD_CONST, identifier, '=', expression ;
variable_get = identifier ;
variable_set = identifier, variable_index?, '=', expression ;

(* Function *)
function_call = command_modifier, identifier, '(', [ expression, { ',', expression } ], ')' ;
function_call_failed = function_call, [ handler ] ;
function_def = { attribute }, [ VISIBILITY ], KEYWORD_FUN, identifier, '(', [ identifier, { ',', identifier } ], ')', block ;
function_def_typed = { attribute }, [ VISIBILITY ], KEYWORD_FUN, identifier, '(',
    [ identifier, ':', TYPE, { ',', identifier, ':', TYPE } ], ')', ':', TYPE, block ;

(* Loop *)
loop = KEYWORD_LOOP, block ;
loop_array = KEYWORD_FOR | KEYWORD_LOOP, identifier, KEYWORD_IN, expression, block ;
loop_array_iterator = KEYWORD_FOR | KEYWORD_LOOP, identifier, ',', identifier, KEYWORD_IN, expression, block ;
while_loop = KEYWORD_WHILE, expression, block ;

(* Ranges *)
range = expression, '..', expression ;
range_inclusive = expression, '..=', expression ;

(* Conditional *)
if_statement = KEYWORD_IF, expression, block, [ KEYWORD_ELSE, block ] ;
if_chain = KEYWORD_IF, '{', { expression, block }, [ KEYWORD_ELSE, block ],  '}' ;
ternary = expression, KEYWORD_THEN, expression, KEYWORD_ELSE, expression ;

(* Main *)
main = KEYWORD_MAIN, [ '(', identifier, ')' ], block ;

(* Imports *)
import_path = '"', { ANY_CHAR }, '"' ;
import_all = [ VISIBILITY ], KEYWORD_IMPORT, '*', KEYWORD_FROM, import_path ;
import_ids = [ VISIBILITY ], KEYWORD_IMPORT, '{', { identifier, [ KEYWORD_AS, identifier ], [ ',' ] }, '}', KEYWORD_FROM, import_path ;

(* Comment *)
comment = '//', { ANY_CHAR }, '\n' ;



"#;

pub fn generate_grammar_ebnf() -> String {
    // Collect unique keywords using HashSet to deduplicate
    let keywords: HashSet<&str> = iter_keywords()
        .filter(|r| r.kind != KeywordKind::BinaryOp || matches!(r.keyword, "and" | "or"))
        .map(|r| r.keyword)
        .collect();

    // Collect builtin statement and expression keywords separately
    let mut builtin_stmt_keywords: Vec<&str> = iter_keywords()
        .filter(|r| r.kind == KeywordKind::BuiltinStmt)
        .map(|r| r.keyword)
        .collect();
    builtin_stmt_keywords.sort_unstable();

    let mut builtin_expr_keywords: Vec<&str> = iter_keywords()
        .filter(|r| r.kind == KeywordKind::BuiltinExpr)
        .map(|r| r.keyword)
        .collect();
    builtin_expr_keywords.sort_unstable();

    // Collect unique keywords using HashSet to deduplicate
    let mut keyword_defs = String::new();
    let mut keyword_list: Vec<_> = keywords.into_iter().collect();
    // Sort for deterministic output
    keyword_list.sort();
    for kw in keyword_list {
        keyword_defs.push_str(&format!("KEYWORD_{} = '{}' ;\n", kw.to_uppercase(), kw));
    }
    keyword_defs.push('\n');

    // Generate individual builtin rules for statement builtins
    let mut builtin_stmt_rules = String::from("(* Builtins *)\n");
    for kw in &builtin_stmt_keywords {
        let kw_upper = kw.to_uppercase();
        let builtin_name = format!("builtin_{}", kw);
        match *kw {
            "clear" => {
                builtin_stmt_rules.push_str(&format!("{} = KEYWORD_{} ;\n", builtin_name, kw_upper))
            }
            _ => builtin_stmt_rules.push_str(&format!(
                "{} = KEYWORD_{}, expression ;\n",
                builtin_name, kw_upper
            )),
        }
    }
    builtin_stmt_rules.push('\n');

    // Generate individual builtin rules for expression builtins
    let mut builtin_expr_rules = String::new();
    for kw in &builtin_expr_keywords {
        let kw_upper = kw.to_uppercase();
        let builtin_name = format!("builtin_{}", kw);
        match *kw {
            "pid" | "shellname" | "shellversion" => {
                builtin_expr_rules.push_str(&format!("{} = KEYWORD_{} ;\n", builtin_name, kw_upper))
            }
            _ => builtin_expr_rules.push_str(&format!(
                "{} = KEYWORD_{}, expression ;\n",
                builtin_name, kw_upper
            )),
        }
    }
    builtin_expr_rules.push('\n');

    // Generate builtins_statement rule
    let builtins_stmt_rule = if builtin_stmt_keywords.is_empty() {
        String::new()
    } else {
        let alternatives: Vec<String> = builtin_stmt_keywords
            .iter()
            .map(|kw| format!("builtin_{}", kw))
            .collect();
        format!("builtins_statement = {} ;\n\n", alternatives.join(" | "))
    };

    // Generate builtins_expression rule
    let builtins_expr_rule = if builtin_expr_keywords.is_empty() {
        String::new()
    } else {
        let alternatives: Vec<String> = builtin_expr_keywords
            .iter()
            .map(|kw| format!("builtin_{}", kw))
            .collect();
        format!("builtins_expression = {} ;\n\n", alternatives.join(" | "))
    };

    let keywords_start = "(* Keywords - auto-generated at build time *)";
    let keywords_section_start = GRAMMAR_EBNF
        .find(keywords_start)
        .expect("Keywords marker not found");
    // Find the start of the line containing the comment
    let line_start = GRAMMAR_EBNF[..keywords_section_start]
        .rfind('\n')
        .map_or(0, |pos| pos + 1);
    let before_keywords = &GRAMMAR_EBNF[..line_start];
    let after_terminals = GRAMMAR_EBNF
        .find("(* Terminals *)")
        .map(|end| &GRAMMAR_EBNF[end..])
        .expect("Terminals marker not found");

    let keywords_section_with_comment = format!("{}\n{}", keywords_start, keyword_defs);

    // Add test section after builtins
    let test_section = r#"
(* Test *)
test_name = '"', { ANY_CHAR }, '"' ;
test = KEYWORD_TEST, [ test_name ], block ;
"#;

    // Construct the final grammar with generated builtins
    format!(
        "{}{}{}{}{}{}{}{}",
        before_keywords,
        keywords_section_with_comment,
        after_terminals,
        builtins_stmt_rule,
        builtin_stmt_rules,
        builtins_expr_rule,
        builtin_expr_rules,
        test_section
    )
}
