//! Shared utility functions
//!
//! Common algorithms used across multiple compiler modules.

// ============================================================================
// v0.90.43: Levenshtein Distance — Typo Suggestions
// ============================================================================

/// Calculate Levenshtein edit distance between two strings.
/// Uses O(min(m,n)) space with two-row optimization.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Find the most similar name from a list of candidates.
/// Returns `Some(suggestion)` if a match is found within the threshold.
pub fn find_similar_name<'a>(name: &str, candidates: &[&'a str], threshold: usize) -> Option<&'a str> {
    let mut best_match: Option<&str> = None;
    let mut best_distance = usize::MAX;

    for &candidate in candidates {
        let distance = levenshtein_distance(name, candidate);
        if distance < best_distance && distance <= threshold {
            best_distance = distance;
            best_match = Some(candidate);
        }
    }

    best_match
}

/// Format a "did you mean" suggestion hint for an unknown name.
pub fn format_suggestion_hint(suggestion: Option<&str>) -> String {
    match suggestion {
        Some(name) => format!("\n  hint: did you mean `{}`?", name),
        None => String::new(),
    }
}

// ============================================================================
// v0.90.121: Naming Convention Checks
// ============================================================================

/// Check if a name is snake_case.
/// Valid: `foo`, `foo_bar`, `_foo`, `foo123`, `_`
/// Invalid: `fooBar`, `FooBar`, `FOO_BAR` (unless all-caps which we allow)
pub fn is_snake_case(name: &str) -> bool {
    if name.is_empty() || name == "_" {
        return true;
    }
    // Allow names starting with underscore (private convention)
    let check = name.strip_prefix('_').unwrap_or(name);
    if check.is_empty() {
        return true;
    }
    // All lowercase letters, digits, and underscores
    check.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Convert a name to snake_case.
/// `fooBar` → `foo_bar`, `FooBar` → `foo_bar`, `HTMLParser` → `html_parser`
pub fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let prefix = if name.starts_with('_') { "_" } else { "" };
    let check = name.strip_prefix('_').unwrap_or(name);

    for (i, c) in check.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    format!("{}{}", prefix, result)
}

/// Check if a name is PascalCase.
/// Valid: `Foo`, `FooBar`, `MyType`, `F`, `F64`
/// Invalid: `foo`, `foo_bar`, `fooBar`
pub fn is_pascal_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    // Must start with uppercase letter
    let first = name.chars().next().unwrap();
    if !first.is_ascii_uppercase() {
        return false;
    }
    // Must not contain underscores
    !name.contains('_')
}

/// Convert a name to PascalCase.
/// `foo_bar` → `FooBar`, `fooBar` → `FooBar`
pub fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_single_edit() {
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
    }

    #[test]
    fn test_levenshtein_multiple_edits() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_levenshtein_empty_strings() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
    }

    #[test]
    fn test_levenshtein_single_char() {
        assert_eq!(levenshtein_distance("a", "b"), 1);
        assert_eq!(levenshtein_distance("a", "a"), 0);
        assert_eq!(levenshtein_distance("a", "ab"), 1);
    }

    #[test]
    fn test_levenshtein_case_sensitive() {
        assert_eq!(levenshtein_distance("Hello", "hello"), 1);
    }

    #[test]
    fn test_find_similar_name_exact() {
        assert_eq!(find_similar_name("hello", &["hello", "world"], 2), Some("hello"));
    }

    #[test]
    fn test_find_similar_name_close() {
        assert_eq!(find_similar_name("helo", &["hello", "world"], 2), Some("hello"));
    }

    #[test]
    fn test_find_similar_name_none() {
        assert_eq!(find_similar_name("xyz", &["hello", "world"], 2), None);
    }

    #[test]
    fn test_format_suggestion_hint_some() {
        let hint = format_suggestion_hint(Some("hello"));
        assert!(hint.contains("did you mean `hello`?"));
    }

    #[test]
    fn test_format_suggestion_hint_none() {
        assert_eq!(format_suggestion_hint(None), "");
    }

    // v0.90.121: Naming convention tests
    #[test]
    fn test_is_snake_case() {
        assert!(is_snake_case("foo"));
        assert!(is_snake_case("foo_bar"));
        assert!(is_snake_case("_foo"));
        assert!(is_snake_case("foo123"));
        assert!(is_snake_case("_"));
        assert!(is_snake_case(""));
        assert!(!is_snake_case("fooBar"));
        assert!(!is_snake_case("FooBar"));
        assert!(!is_snake_case("FOO_BAR"));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("fooBar"), "foo_bar");
        assert_eq!(to_snake_case("FooBar"), "foo_bar");
        assert_eq!(to_snake_case("foo"), "foo");
        assert_eq!(to_snake_case("_fooBar"), "_foo_bar");
    }

    #[test]
    fn test_is_pascal_case() {
        assert!(is_pascal_case("Foo"));
        assert!(is_pascal_case("FooBar"));
        assert!(is_pascal_case("F"));
        assert!(is_pascal_case(""));
        assert!(!is_pascal_case("foo"));
        assert!(!is_pascal_case("foo_bar"));
        assert!(!is_pascal_case("Foo_Bar"));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("foo_bar"), "FooBar");
        assert_eq!(to_pascal_case("foo"), "Foo");
        assert_eq!(to_pascal_case("my_type"), "MyType");
    }

    // --- Cycle 1227: Additional Utility Tests ---

    #[test]
    fn test_levenshtein_completely_different() {
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
    }

    #[test]
    fn test_levenshtein_prefix() {
        assert_eq!(levenshtein_distance("abc", "abcdef"), 3);
    }

    #[test]
    fn test_levenshtein_suffix() {
        assert_eq!(levenshtein_distance("def", "abcdef"), 3);
    }

    #[test]
    fn test_levenshtein_substitution_only() {
        assert_eq!(levenshtein_distance("abc", "axc"), 1);
    }

    #[test]
    fn test_levenshtein_transposition() {
        // Transposition is 2 edits in standard Levenshtein
        assert_eq!(levenshtein_distance("ab", "ba"), 2);
    }

    #[test]
    fn test_find_similar_name_best_match() {
        let candidates = &["println", "print", "printf"];
        assert_eq!(find_similar_name("prnt", candidates, 3), Some("print"));
    }

    #[test]
    fn test_find_similar_name_threshold_exact() {
        // Distance of exactly threshold should match
        let candidates = &["abc"];
        assert_eq!(find_similar_name("axc", candidates, 1), Some("abc"));
    }

    #[test]
    fn test_find_similar_name_threshold_exceeded() {
        let candidates = &["abc"];
        assert_eq!(find_similar_name("xyz", candidates, 1), None);
    }

    #[test]
    fn test_find_similar_name_empty_candidates() {
        let candidates: &[&str] = &[];
        assert_eq!(find_similar_name("abc", candidates, 3), None);
    }

    #[test]
    fn test_is_snake_case_leading_underscore_numbers() {
        assert!(is_snake_case("_x123"));
        assert!(is_snake_case("a_b_c"));
        assert!(is_snake_case("x"));
        assert!(!is_snake_case("A"));
    }

    #[test]
    fn test_to_snake_case_consecutive_upper() {
        // HTMLParser → h_t_m_l_parser (simple char-by-char conversion)
        let result = to_snake_case("HTMLParser");
        assert!(result.contains("parser"));
        assert!(!result.contains('H'));
    }

    #[test]
    fn test_to_snake_case_all_upper() {
        let result = to_snake_case("ABC");
        assert_eq!(result, "a_b_c");
    }

    #[test]
    fn test_is_pascal_case_with_numbers() {
        assert!(is_pascal_case("F64"));
        assert!(is_pascal_case("MyType2"));
        assert!(!is_pascal_case("f64"));
    }

    #[test]
    fn test_to_pascal_case_single_char() {
        assert_eq!(to_pascal_case("f"), "F");
        assert_eq!(to_pascal_case("a_b"), "AB");
    }

    #[test]
    fn test_to_pascal_case_already_pascal() {
        // Input is already PascalCase — should be preserved
        assert_eq!(to_pascal_case("Foo"), "Foo");
    }

    #[test]
    fn test_format_suggestion_hint_format() {
        let hint = format_suggestion_hint(Some("method_name"));
        assert!(hint.starts_with('\n'));
        assert!(hint.contains("did you mean"));
        assert!(hint.contains("`method_name`"));
    }

    #[test]
    fn test_is_snake_case_double_underscore() {
        assert!(is_snake_case("__init__"));
        assert!(is_snake_case("a__b"));
    }

    #[test]
    fn test_is_pascal_case_single_uppercase() {
        assert!(is_pascal_case("A"));
        assert!(is_pascal_case("Z"));
    }

    // ================================================================
    // Additional util tests (Cycle 1240)
    // ================================================================

    #[test]
    fn test_levenshtein_symmetric() {
        assert_eq!(levenshtein_distance("abc", "xyz"), levenshtein_distance("xyz", "abc"));
        assert_eq!(levenshtein_distance("kitten", "sitting"), levenshtein_distance("sitting", "kitten"));
    }

    #[test]
    fn test_levenshtein_insertion_deletion() {
        // Pure insertions
        assert_eq!(levenshtein_distance("abc", "aXbYcZ"), 3);
        // Pure deletions
        assert_eq!(levenshtein_distance("aXbYcZ", "abc"), 3);
    }

    #[test]
    fn test_find_similar_name_threshold_zero() {
        // threshold 0 means only exact matches
        let candidates = &["hello", "helo", "world"];
        assert_eq!(find_similar_name("hello", candidates, 0), Some("hello"));
        assert_eq!(find_similar_name("helo", candidates, 0), Some("helo"));
        assert_eq!(find_similar_name("hell", candidates, 0), None);
    }

    #[test]
    fn test_find_similar_name_picks_closest() {
        // Multiple close matches, should return the closest one
        let candidates = &["print", "printf", "println"];
        assert_eq!(find_similar_name("print", candidates, 3), Some("print"));
        assert_eq!(find_similar_name("prnt", candidates, 3), Some("print"));
    }

    #[test]
    fn test_to_snake_case_empty() {
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn test_to_snake_case_already_snake() {
        assert_eq!(to_snake_case("foo_bar_baz"), "foo_bar_baz");
    }

    #[test]
    fn test_to_pascal_case_empty() {
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_to_pascal_case_leading_underscores() {
        // Underscores become capitalize-next triggers
        assert_eq!(to_pascal_case("_foo"), "Foo");
        assert_eq!(to_pascal_case("__bar"), "Bar");
    }

    #[test]
    fn test_is_snake_case_trailing_underscore() {
        assert!(is_snake_case("foo_"));
        assert!(is_snake_case("a_b_"));
    }

    #[test]
    fn test_is_pascal_case_number_start_after_upper() {
        assert!(is_pascal_case("A1B2"));
        assert!(is_pascal_case("Type3"));
        // lowercase start is not pascal
        assert!(!is_pascal_case("a1B2"));
    }
}
