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
}
