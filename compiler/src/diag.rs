//! Structured diagnostics with stable codes. Output is JSONL, one object per line,
//! sorted so the same input always produces the same bytes.

use crate::source::{SourceMap, Span};

/// Every diagnostic code the compiler can emit. Codes are stable: never renumber,
/// never reuse a retired code.
pub const CODES: &[(&str, &str)] = &[
    // E01xx: lexing and syntax
    ("E0101", "unexpected character"),
    ("E0102", "unterminated string literal"),
    ("E0103", "syntax error: unexpected token"),
    (
        "E0104",
        "integer literal suffix not allowed; the type comes from context",
    ),
    ("E0105", "integer literal too large"),
    ("E0106", "block comments are not allowed; use `//`"),
    // E02xx: Rust habits rejected by BMB (each carries a fix when mechanical)
    ("E0201", "`let` requires an explicit type"),
    ("E0202", "macros are not allowed"),
    (
        "E0203",
        "`return` is not allowed; end the block with a tail expression",
    ),
    (
        "E0204",
        "`for` and `loop` are not allowed; use `while` with `invariant` and `decreases`",
    ),
    ("E0205", "`as` casts are not allowed"),
    ("E0206", "the `?` operator is not allowed"),
    ("E0207", "construct not supported in milestone 1"),
    ("E0208", "lifetime annotations are not allowed"),
    (
        "E0209",
        "function return type is required (write `-> ()` for unit)",
    ),
    ("E0210", "`mut` parameters are not allowed"),
    ("E0211", "comparison operators cannot be chained"),
    // E03xx: names
    ("E0301", "unknown name"),
    ("E0302", "duplicate definition"),
    (
        "E0303",
        "name already bound in this function (shadowing is not allowed)",
    ),
    ("E0304", "unknown function"),
    // E04xx: types and ownership
    ("E0401", "type mismatch"),
    ("E0402", "unsupported type"),
    ("E0403", "references are only allowed in parameter position"),
    (
        "E0404",
        "unknown method (no method lookup; only built-in methods exist)",
    ),
    ("E0405", "wrong number of arguments"),
    ("E0406", "integer literal out of range for its type"),
    ("E0407", "value is not indexable"),
    ("E0408", "assignment to an immutable place"),
    (
        "E0409",
        "`&` / `&mut` may only appear directly as a call argument",
    ),
    (
        "E0410",
        "a `&mut` argument aliases another argument of the same call",
    ),
    ("E0411", "cannot negate an unsigned value"),
    // E05xx: specifications
    (
        "E0501",
        "contract clause must be a call to a named predicate (`spec fn ... -> bool`)",
    ),
    ("E0502", "vacuous predicate used in a contract"),
    (
        "E0503",
        "specification-only construct used in executable code",
    ),
    ("E0504", "executable function called in a specification"),
    ("E0505", "spec function called in executable code"),
    ("E0506", "`while` requires `decreases`"),
    ("E0507", "recursive function requires `decreases`"),
    (
        "E0508",
        "`old(..)` is only allowed in `ensures` on a `&mut` parameter",
    ),
    ("E0509", "`#[uses(..)]` entry is not a spec function"),
    ("E0510", "`#[trigger]` is only allowed inside `forall`"),
    // E06xx: effects and attributes
    ("E0601", "function requires an `#[effects(..)]` attribute"),
    ("E0602", "unknown effect"),
    ("E0603", "`pure` function calls a function with effect `io`"),
    ("E0604", "spec functions take no `#[effects(..)]` attribute"),
    ("E0605", "unknown or misplaced attribute"),
    // E07xx: program structure
    ("E0701", "`main` must be `#[effects(io)] fn main() -> ()`"),
    ("E0702", "more than one `main`"),
    ("E0703", "items must be declared inside a `mod`"),
    ("E0704", "`mod` requires a `#[summary(\"..\")]` attribute"),
];

pub fn describe(code: &str) -> &'static str {
    CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, d)| *d)
        .expect("unregistered diagnostic code")
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub message: String,
    /// Replacement text for the diagnostic span, when the fix is mechanical.
    pub replacement: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Diag {
    pub code: &'static str,
    pub span: Span,
    pub message: String,
    pub fix: Option<Fix>,
}

impl Diag {
    pub fn new(code: &'static str, span: Span, message: impl Into<String>) -> Self {
        let _ = describe(code);
        Diag {
            code,
            span,
            message: message.into(),
            fix: None,
        }
    }
    pub fn fix(mut self, message: impl Into<String>, replacement: Option<String>) -> Self {
        self.fix = Some(Fix {
            message: message.into(),
            replacement,
        });
        self
    }
}

#[derive(Default)]
pub struct Diags {
    pub items: Vec<Diag>,
}

impl Diags {
    pub fn push(&mut self, d: Diag) {
        self.items.push(d);
    }
    pub fn has_errors(&self) -> bool {
        !self.items.is_empty()
    }
    /// Deterministic JSONL rendering: sorted by position then code, duplicates removed.
    pub fn to_jsonl(&self, sm: &SourceMap) -> String {
        let mut rows: Vec<(u32, u32, &'static str, String)> = self
            .items
            .iter()
            .map(|d| (d.span.file, d.span.lo, d.code, render(d, sm)))
            .collect();
        rows.sort();
        rows.dedup_by(|a, b| a.3 == b.3);
        let mut out = String::new();
        for r in rows {
            out.push_str(&r.3);
            out.push('\n');
        }
        out
    }
}

fn render(d: &Diag, sm: &SourceMap) -> String {
    let (line, col) = sm.line_col(d.span.file, d.span.lo);
    let (eline, ecol) = sm.line_col(d.span.file, d.span.hi);
    let mut s = String::from("{");
    s.push_str(&format!("\"code\":{},", json_str(d.code)));
    s.push_str("\"severity\":\"error\",");
    s.push_str(&format!("\"message\":{},", json_str(&d.message)));
    s.push_str(&format!("\"file\":{},", json_str(sm.name(d.span.file))));
    s.push_str(&format!(
        "\"line\":{line},\"col\":{col},\"end_line\":{eline},\"end_col\":{ecol}"
    ));
    if let Some(f) = &d.fix {
        s.push_str(&format!(",\"fix\":{{\"message\":{}", json_str(&f.message)));
        if let Some(r) = &f.replacement {
            s.push_str(&format!(",\"replacement\":{}", json_str(r)));
        }
        s.push('}');
    }
    s.push('}');
    s
}

pub fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

pub fn codes_jsonl() -> String {
    let mut out = String::new();
    for (c, d) in CODES {
        out.push_str(&format!(
            "{{\"code\":{},\"description\":{}}}\n",
            json_str(c),
            json_str(d)
        ));
    }
    out
}
