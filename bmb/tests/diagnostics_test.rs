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
