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
    // Should match method_call pattern
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"method_call"));
}

#[test]
fn test_for_loop_parser_error() {
    let matches = find_patterns("parser", "unexpected token `for`");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "for_loop");
}

#[test]
fn test_reassign_error() {
    let matches = find_patterns("type", "cannot assign to immutable variable `x`");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "reassign_set");
}

#[test]
fn test_no_false_positive() {
    // "integer overflow" should not match any Rust-ism pattern
    let matches = find_patterns("type", "integer overflow in constant expression");
    assert!(matches.is_empty());
}

#[test]
fn test_kind_filter() {
    // for_loop has kind="parser" — should NOT match with kind="type"
    let matches = find_patterns("type", "unexpected token `for`");
    let for_matches: Vec<_> = matches.iter().filter(|m| m.id == "for_loop").collect();
    assert!(for_matches.is_empty());
}

#[test]
fn test_case_insensitive() {
    let matches = find_patterns("", "Unknown type `Vec<i64>`");
    assert!(!matches.is_empty());
}

// --- Phase 2 pattern tests ---

#[test]
fn test_return_keyword() {
    let matches = find_patterns("parser", "unexpected token `return`");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "return_keyword");
}

#[test]
fn test_break_continue() {
    let matches = find_patterns("parser", "unexpected token `break`");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "break_continue");
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
    // Phase 1: 23 patterns, Phase 2: +11 = 34 total
    use bmb::diagnostics::PATTERNS;
    assert!(PATTERNS.len() >= 34, "Expected at least 34 patterns, got {}", PATTERNS.len());
}
