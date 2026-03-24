//! AI-friendly diagnostic patterns for BMB compiler errors.
//!
//! Maps (error_kind, message_triggers) → (suggestion, example).
//! Used by report_error_machine to enrich JSONL output.

mod patterns;

pub use patterns::PATTERNS;

/// A diagnostic pattern that matches compiler errors and provides AI-friendly hints.
#[derive(Debug)]
pub struct DiagPattern {
    pub id: &'static str,
    /// Error kind filter: "parser", "type", "resolve", or "" for any
    pub kind: &'static str,
    /// Trigger substrings to match in the error message (any match = pattern applies)
    pub triggers: &'static [&'static str],
    /// Human/AI readable suggestion text
    pub suggestion: &'static str,
    /// What the AI likely wrote (wrong)
    pub example_wrong: &'static str,
    /// What it should be (correct BMB)
    pub example_correct: &'static str,
}

/// Find matching patterns for a given error kind and message.
pub fn find_patterns(kind: &str, message: &str) -> Vec<&'static DiagPattern> {
    let msg_lower = message.to_lowercase();
    PATTERNS
        .iter()
        .filter(|p| {
            let kind_ok = p.kind.is_empty() || p.kind == kind;
            let trigger_ok = p.triggers.iter().any(|t| msg_lower.contains(&t.to_lowercase()));
            kind_ok && trigger_ok
        })
        .collect()
}
