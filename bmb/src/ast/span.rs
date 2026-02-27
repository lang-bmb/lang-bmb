//! Source location tracking

use serde::{Deserialize, Serialize};

/// A span in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

// v0.90.46: Display for human-readable span output
impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<Span> for std::ops::Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Span::new(range.start, range.end)
    }
}

/// A value with source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // Span tests (Cycle 428)
    // ====================================================================

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
    }

    #[test]
    fn test_span_merge_non_overlapping() {
        let a = Span::new(0, 5);
        let b = Span::new(10, 15);
        let merged = a.merge(b);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_span_merge_overlapping() {
        let a = Span::new(5, 15);
        let b = Span::new(10, 20);
        let merged = a.merge(b);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 20);
    }

    #[test]
    fn test_span_merge_contained() {
        let outer = Span::new(0, 100);
        let inner = Span::new(20, 30);
        let merged = outer.merge(inner);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 100);
    }

    #[test]
    fn test_span_merge_same() {
        let a = Span::new(5, 10);
        let merged = a.merge(a);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 10);
    }

    #[test]
    fn test_span_merge_reversed_order() {
        let a = Span::new(10, 20);
        let b = Span::new(0, 5);
        let merged = a.merge(b);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 20);
    }

    #[test]
    fn test_span_display() {
        let span = Span::new(42, 99);
        assert_eq!(format!("{}", span), "42..99");
    }

    #[test]
    fn test_span_display_zero() {
        let span = Span::new(0, 0);
        assert_eq!(format!("{}", span), "0..0");
    }

    #[test]
    fn test_span_to_range() {
        let span = Span::new(5, 15);
        let range: std::ops::Range<usize> = span.into();
        assert_eq!(range, 5..15);
    }

    #[test]
    fn test_range_to_span() {
        let range = 10..20usize;
        let span: Span = range.into();
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
    }

    #[test]
    fn test_span_equality() {
        assert_eq!(Span::new(1, 2), Span::new(1, 2));
        assert_ne!(Span::new(1, 2), Span::new(1, 3));
        assert_ne!(Span::new(1, 2), Span::new(0, 2));
    }

    #[test]
    fn test_span_clone_copy() {
        let span = Span::new(5, 10);
        let cloned = span;
        // Span is Copy, so original still available
        assert_eq!(span, cloned);
    }

    // ====================================================================
    // Spanned tests (Cycle 428)
    // ====================================================================

    #[test]
    fn test_spanned_new() {
        let s = Spanned::new(42i64, Span::new(0, 5));
        assert_eq!(s.node, 42);
        assert_eq!(s.span, Span::new(0, 5));
    }

    #[test]
    fn test_spanned_map() {
        let s = Spanned::new(10, Span::new(0, 5));
        let mapped = s.map(|n| n * 2);
        assert_eq!(mapped.node, 20);
        assert_eq!(mapped.span, Span::new(0, 5));
    }

    #[test]
    fn test_spanned_map_type_change() {
        let s = Spanned::new(42, Span::new(1, 3));
        let mapped = s.map(|n| format!("{}", n));
        assert_eq!(mapped.node, "42");
        assert_eq!(mapped.span, Span::new(1, 3));
    }

    // --- Cycle 1225: Additional Span/Spanned Tests ---

    #[test]
    fn test_span_merge_adjacent() {
        // Adjacent spans: end of first == start of second
        let a = Span::new(0, 5);
        let b = Span::new(5, 10);
        let merged = a.merge(b);
        assert_eq!(merged, Span::new(0, 10));
    }

    #[test]
    fn test_span_merge_zero_width() {
        let a = Span::new(5, 5);
        let b = Span::new(10, 10);
        let merged = a.merge(b);
        assert_eq!(merged, Span::new(5, 10));
    }

    #[test]
    fn test_span_roundtrip_range_conversion() {
        let span = Span::new(42, 99);
        let range: std::ops::Range<usize> = span.into();
        let back: Span = range.into();
        assert_eq!(span, back);
    }

    #[test]
    fn test_spanned_with_string() {
        let s = Spanned::new("hello".to_string(), Span::new(0, 5));
        assert_eq!(s.node, "hello");
        let mapped = s.map(|s| s.len());
        assert_eq!(mapped.node, 5);
    }

    #[test]
    fn test_span_display_large_values() {
        let span = Span::new(100000, 200000);
        assert_eq!(format!("{}", span), "100000..200000");
    }

    #[test]
    fn test_spanned_new_preserves_span() {
        let span = Span::new(100, 200);
        let s = Spanned::new(vec![1, 2, 3], span);
        assert_eq!(s.span.start, 100);
        assert_eq!(s.span.end, 200);
        assert_eq!(s.node.len(), 3);
    }

    // ================================================================
    // Additional span tests (Cycle 1236)
    // ================================================================

    #[test]
    fn test_span_debug_format() {
        let span = Span::new(3, 7);
        let debug = format!("{:?}", span);
        assert!(debug.contains("Span"));
        assert!(debug.contains("3"));
        assert!(debug.contains("7"));
    }

    #[test]
    fn test_span_merge_commutative() {
        let a = Span::new(10, 20);
        let b = Span::new(5, 15);
        assert_eq!(a.merge(b), b.merge(a));
    }

    #[test]
    fn test_span_merge_associative() {
        let a = Span::new(0, 10);
        let b = Span::new(5, 15);
        let c = Span::new(12, 25);
        assert_eq!(a.merge(b).merge(c), a.merge(b.merge(c)));
    }

    #[test]
    fn test_spanned_clone() {
        let s = Spanned::new("hello".to_string(), Span::new(0, 5));
        let cloned = s.clone();
        assert_eq!(cloned.node, "hello");
        assert_eq!(cloned.span, Span::new(0, 5));
    }

    #[test]
    fn test_spanned_debug() {
        let s = Spanned::new(42, Span::new(0, 2));
        let debug = format!("{:?}", s);
        assert!(debug.contains("42"));
        assert!(debug.contains("Spanned"));
    }

    #[test]
    fn test_span_zero_length() {
        let span = Span::new(5, 5);
        assert_eq!(span.start, span.end);
        assert_eq!(format!("{}", span), "5..5");
        let range: std::ops::Range<usize> = span.into();
        assert!(range.is_empty());
    }

    #[test]
    fn test_spanned_map_preserves_span_exactly() {
        let span = Span::new(42, 99);
        let s = Spanned::new(100, span);
        let mapped = s.map(|n| n.to_string());
        assert_eq!(mapped.span.start, 42);
        assert_eq!(mapped.span.end, 99);
    }

    #[test]
    fn test_span_from_range_zero() {
        let range = 0..0usize;
        let span: Span = range.into();
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 0);
    }

    #[test]
    fn test_spanned_map_to_option() {
        let s = Spanned::new(42, Span::new(0, 5));
        let mapped = s.map(|n| if n > 0 { Some(n) } else { None });
        assert_eq!(mapped.node, Some(42));
    }

    #[test]
    fn test_span_ne_different_start() {
        assert_ne!(Span::new(0, 10), Span::new(1, 10));
    }

    // ================================================================
    // Additional span tests (Cycle 1241)
    // ================================================================

    #[test]
    fn test_span_merge_first_zero_width() {
        let a = Span::new(0, 0);
        let b = Span::new(5, 10);
        let merged = a.merge(b);
        assert_eq!(merged, Span::new(0, 10));
    }

    #[test]
    fn test_span_merge_second_zero_width() {
        let a = Span::new(5, 10);
        let b = Span::new(0, 0);
        let merged = a.merge(b);
        assert_eq!(merged, Span::new(0, 10));
    }

    #[test]
    fn test_spanned_map_identity() {
        let s = Spanned::new(42, Span::new(0, 5));
        let mapped = s.map(|n| n);
        assert_eq!(mapped.node, 42);
        assert_eq!(mapped.span, Span::new(0, 5));
    }

    #[test]
    fn test_span_inverted_allowed() {
        // Span doesn't validate start <= end, it just stores values
        let span = Span::new(10, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 5);
    }

    #[test]
    fn test_span_eq_reflexive() {
        let span = Span::new(42, 99);
        assert_eq!(span, span);
    }

    #[test]
    fn test_spanned_map_chain() {
        let s = Spanned::new(2, Span::new(0, 1));
        let result = s.map(|n| n * 3).map(|n| n + 1);
        assert_eq!(result.node, 7);
        assert_eq!(result.span, Span::new(0, 1));
    }

    #[test]
    fn test_span_into_range_single_char() {
        let span = Span::new(5, 6);
        let range: std::ops::Range<usize> = span.into();
        assert_eq!(range.len(), 1);
    }

    #[test]
    fn test_spanned_new_with_unit() {
        let s = Spanned::new((), Span::new(0, 0));
        assert_eq!(s.node, ());
        assert_eq!(s.span, Span::new(0, 0));
    }

    #[test]
    fn test_span_display_single_position() {
        let span = Span::new(42, 43);
        assert_eq!(format!("{}", span), "42..43");
    }

    #[test]
    fn test_span_ne_different_end() {
        assert_ne!(Span::new(0, 5), Span::new(0, 6));
    }
}
