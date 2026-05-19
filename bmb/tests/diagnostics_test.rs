use bmb::diagnostics::find_patterns;

#[test]
fn test_option_pattern_matches() {
    let matches = find_patterns("type", "unknown type `Option<i64>`");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "option_type");
    assert!(matches[0].suggestion.contains("T?"));
}

#[test]
fn test_vec_method_call_matches() {
    let matches = find_patterns("type", "cannot call .push( on type i64");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"method_call"));
}

#[test]
fn test_no_false_positive() {
    // "integer overflow" should not match any pattern
    let matches = find_patterns("type", "integer overflow in constant expression");
    assert!(matches.is_empty());
}

#[test]
fn test_case_insensitive() {
    let matches = find_patterns("", "Unknown type `Vec<i64>`");
    assert!(!matches.is_empty());
}

#[test]
fn test_closure_lambda() {
    let matches = find_patterns("parser", "unexpected token `|`");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"closure_lambda"));
}

#[test]
fn test_print_string_type_mismatch() {
    let matches = find_patterns("type", "expected &str, got i64");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "print_string_fn");
}

#[test]
fn test_iterator_methods() {
    let matches = find_patterns("", "unknown method .iter() on type i64");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"iterator_methods"));
}

#[test]
fn test_type_cast() {
    let matches = find_patterns("", "unexpected token as usize");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"type_cast"));
}

#[test]
fn test_void_return_regression() {
    let matches = find_patterns("type", "expected i64, got ()");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "void_return_used");
}

#[test]
fn test_total_pattern_count() {
    // After removing 7 incorrect patterns: 34 - 7 = 27
    use bmb::diagnostics::PATTERNS;
    assert!(
        PATTERNS.len() >= 27,
        "Expected at least 27 patterns, got {}",
        PATTERNS.len()
    );
}

#[test]
fn test_option_none_no_false_positive() {
    // "memory(none)" in LLVM IR errors must NOT trigger option_type
    let msg = "Linker error: clang compile failed: invalid redefinition of function 'clamp'\n  define private i64 @clamp(...) memory(none) nounwind";
    let matches = find_patterns("", msg);
    let opt_matches: Vec<_> = matches.iter().filter(|m| m.id == "option_type").collect();
    assert!(opt_matches.is_empty(), "option_type must not fire on LLVM IR memory(none)");
}

#[test]
fn test_function_name_reserved() {
    let matches = find_patterns("", "Linker error: clang compile failed: invalid redefinition of function 'clamp'");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"function_name_reserved"), "expected function_name_reserved, got {:?}", ids);
}

#[test]
fn test_if_stmt_no_semicolon() {
    let matches = find_patterns("parser", "Unrecognized token `if` found at 123:125\nExpected one of \";\" or \"}\"");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"if_stmt_no_semicolon"), "expected if_stmt_no_semicolon, got {:?}", ids);
}

#[test]
fn test_if_stmt_no_semicolon_identifier() {
    // "Unrecognized token `i`... Expected one of \"else\", \";\" or \"}\"" also triggers if_stmt_no_semicolon
    let msg = "Unrecognized token `i` found at 350:351\nExpected one of \"else\", \";\" or \"}\"";
    let matches = find_patterns("parser", msg);
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"if_stmt_no_semicolon"), "expected if_stmt_no_semicolon for identifier after if-block, got {:?}", ids);
}

#[test]
fn test_bool_operators_pipe() {
    // "||" in source produces "Unrecognized token `|`" — should suggest 'or'
    let matches = find_patterns("parser", "Unrecognized token `|` found at 5:6");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"bool_operators"), "expected bool_operators, got {:?}", ids);
    let pat = matches.iter().find(|m| m.id == "bool_operators").unwrap();
    assert!(pat.suggestion.contains("or"), "suggestion should mention 'or'");
}

#[test]
fn test_bool_operators_ampersand() {
    // "&&" in source produces "Unrecognized token `&`" — should suggest 'and'
    let matches = find_patterns("parser", "Unrecognized token `&` found at 5:6");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"bool_operators"), "expected bool_operators, got {:?}", ids);
}

#[test]
fn test_bool_operators_bitwise_band() {
    // "n & 1" produces "Unrecognized token `&`" — suggestion must mention 'band' for bitwise
    let matches = find_patterns("parser", "Unrecognized token `&` found at 10:11");
    assert!(!matches.is_empty());
    let pat = matches.iter().find(|m| m.id == "bool_operators").unwrap();
    assert!(pat.suggestion.contains("band"), "suggestion must mention 'band' for bitwise AND, got: {}", pat.suggestion);
}

#[test]
fn test_bool_operators_bitwise_bor() {
    // "n | 1" produces "Unrecognized token `|`" — suggestion must mention 'bor' for bitwise
    let matches = find_patterns("parser", "Unrecognized token `|` found at 10:11");
    assert!(!matches.is_empty());
    let pat = matches.iter().find(|m| m.id == "bool_operators").unwrap();
    assert!(pat.suggestion.contains("bor"), "suggestion must mention 'bor' for bitwise OR, got: {}", pat.suggestion);
}

#[test]
fn test_contract_param_undefined() {
    // "undefined variable: `n`" when contract is on fn main() should fire contract_param_undefined
    let matches = find_patterns("type", "undefined variable: `n`");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"contract_param_undefined"), "expected contract_param_undefined, got {:?}", ids);
    // Suggestion should mention main() and contracts
    let pat = matches.iter().find(|m| m.id == "contract_param_undefined").unwrap();
    assert!(pat.suggestion.contains("main"), "suggestion should mention main");
}

// Verify removed patterns don't exist (BMB now supports these features)
#[test]
fn test_no_for_loop_pattern() {
    // for loops are now supported in BMB — no pattern should trigger
    let matches = find_patterns("parser", "unexpected token `for`");
    let for_matches: Vec<_> = matches.iter().filter(|m| m.id == "for_loop").collect();
    assert!(for_matches.is_empty(), "for_loop pattern should be removed — BMB supports for loops");
}

#[test]
fn test_no_break_continue_pattern() {
    let matches = find_patterns("parser", "unexpected token `break`");
    let bc_matches: Vec<_> = matches.iter().filter(|m| m.id == "break_continue").collect();
    assert!(bc_matches.is_empty(), "break_continue pattern should be removed — BMB supports break/continue");
}

#[test]
fn test_no_return_keyword_pattern() {
    let matches = find_patterns("parser", "unexpected token `return`");
    let ret_matches: Vec<_> = matches.iter().filter(|m| m.id == "return_keyword").collect();
    assert!(ret_matches.is_empty(), "return_keyword pattern should be removed — BMB supports return");
}
