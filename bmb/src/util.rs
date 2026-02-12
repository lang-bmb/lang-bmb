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
}
