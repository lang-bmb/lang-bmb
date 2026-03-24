# AI-Friendly Tooling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enrich BMB compiler's JSONL error output with AI-friendly suggestions and examples, so LLMs can self-correct BMB syntax errors in 1-2 loops instead of 5+.

**Architecture:** Add a `diagnostics` module to the Rust compiler with a PatternBank that maps (error_kind, message_triggers) → (suggestion, example_wrong, example_correct). The existing `report_error_machine` function queries PatternBank before emitting JSONL, adding optional fields. No changes to CompileError struct or compilation pipeline.

**Tech Stack:** Rust (compiler), Python (ai-proof integration)

**Spec:** `docs/superpowers/specs/2026-03-25-ai-friendly-tooling-design.md`

---

## File Structure

```
bmb/src/
├── diagnostics/           # NEW MODULE
│   ├── mod.rs             # PatternBank struct + API
│   └── patterns.rs        # All pattern definitions (data)
├── error/
│   └── mod.rs             # MODIFY: report_error_machine + report_warning_machine
└── main.rs                # NO CHANGE (--human already works)

ecosystem/ai-proof/
├── orchestrator/
│   ├── error_normalizer.py  # MODIFY: parse suggestion/example from JSONL
│   └── experiment.py        # NO CHANGE
└── protocol/
    └── prompt_templates.py  # MODIFY: include suggestion in feedback
```

---

## Task 1: PatternBank Data Module

**Files:**
- Create: `bmb/src/diagnostics/mod.rs`
- Create: `bmb/src/diagnostics/patterns.rs`
- Modify: `bmb/src/lib.rs` — add `pub mod diagnostics;`

- [ ] **Step 1: Add diagnostics module declaration**

In `bmb/src/lib.rs`, find the existing module declarations and add:

```rust
pub mod diagnostics;
```

- [ ] **Step 2: Create diagnostics/mod.rs**

```rust
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
```

- [ ] **Step 3: Create diagnostics/patterns.rs with initial patterns**

```rust
//! Pattern definitions — AI mistake patterns derived from pilot data + CLAUDE.md.

use super::DiagPattern;

pub static PATTERNS: &[DiagPattern] = &[
    // --- Rust-isms that don't exist in BMB ---
    DiagPattern {
        id: "option_type",
        kind: "",
        triggers: &["Option<", "Option::", "Some(", "None"],
        suggestion: "BMB uses T? for nullable types, not Option<T>",
        example_wrong: "let x: Option<i64> = Some(42);",
        example_correct: "let x: i64? = 42;",
    },
    DiagPattern {
        id: "vec_generic",
        kind: "",
        triggers: &["Vec<", "Vec::new", "vec!", ".push(", ".pop(", ".len()"],
        suggestion: "BMB vectors use free functions: vec_new(), vec_push(v, val), vec_get(v, idx), vec_len(v), vec_set(v, idx, val), vec_free(v). Handle type is i64.",
        example_wrong: "let v: Vec<i64> = Vec::new();\nv.push(42);\nlet n = v.len();",
        example_correct: "let v: i64 = vec_new();\nvec_push(v, 42);\nlet n: i64 = vec_len(v);",
    },
    DiagPattern {
        id: "method_call",
        kind: "",
        triggers: &[".len()", ".push(", ".pop(", ".get(", ".set(", ".contains("],
        suggestion: "BMB uses free functions, not method calls. Use func(obj, args) instead of obj.func(args).",
        example_wrong: "arr.len()",
        example_correct: "vec_len(arr)",
    },
    DiagPattern {
        id: "println_macro",
        kind: "",
        triggers: &["println!", "print!", "eprintln!", "format!"],
        suggestion: "BMB uses println(value) function, not Rust macros.",
        example_wrong: "println!(\"{}\", x);",
        example_correct: "println(x);",
    },
    DiagPattern {
        id: "string_type",
        kind: "",
        triggers: &["String::", "String::from", "&String", ".to_string()", ".as_str()"],
        suggestion: "BMB uses &str for strings. No String type.",
        example_wrong: "let s: String = String::from(\"hello\");",
        example_correct: "let s: &str = \"hello\";",
    },
    DiagPattern {
        id: "for_loop",
        kind: "parser",
        triggers: &["`for`", "for loop", "for("],
        suggestion: "BMB has no for loops or range syntax. Use while loops.",
        example_wrong: "for i in 0..n { body; }",
        example_correct: "let mut i: i64 = 0;\nwhile i < n {\n    // body\n    set i = i + 1;\n};",
    },
    DiagPattern {
        id: "reassign_set",
        kind: "",
        triggers: &["cannot assign", "immutable", "assign to"],
        suggestion: "BMB uses 'set x = value;' to reassign mutable variables, not 'x = value;'.",
        example_wrong: "let mut x: i64 = 0;\nx = 5;",
        example_correct: "let mut x: i64 = 0;\nset x = 5;",
    },
    DiagPattern {
        id: "type_annotation",
        kind: "type",
        triggers: &["type annotation", "cannot infer", "type mismatch"],
        suggestion: "BMB requires explicit type annotations on all let bindings.",
        example_wrong: "let x = 42;",
        example_correct: "let x: i64 = 42;",
    },
    DiagPattern {
        id: "fn_return_expr",
        kind: "parser",
        triggers: &["expected", "return"],
        suggestion: "BMB functions use '= expr;' for expression bodies or '{ ... }' for block bodies.",
        example_wrong: "fn add(a: i64, b: i64) -> i64 { a + b }",
        example_correct: "fn add(a: i64, b: i64) -> i64 = a + b;",
    },
    DiagPattern {
        id: "bitwise_ops",
        kind: "",
        triggers: &["bitwise", "bit and", "bit or"],
        suggestion: "BMB uses 'band', 'bor', 'bxor' for bitwise operations, not &, |, ^.",
        example_wrong: "let x: i64 = a & b;",
        example_correct: "let x: i64 = a band b;",
    },
    DiagPattern {
        id: "impl_block",
        kind: "parser",
        triggers: &["`impl`", "impl block"],
        suggestion: "BMB has no impl blocks. Define free functions instead.",
        example_wrong: "impl Foo {\n    fn bar(&self) -> i64 { ... }\n}",
        example_correct: "fn foo_bar(self: &Foo) -> i64 = ...;",
    },
    DiagPattern {
        id: "trait_def",
        kind: "parser",
        triggers: &["trait "],
        suggestion: "BMB traits use 'trait Name { fn method(self: &Self) -> Type; }' syntax.",
        example_wrong: "trait Foo {\n    fn bar(&self) -> i64;\n}",
        example_correct: "trait Foo {\n    fn bar(self: &Self) -> i64;\n}",
    },
    DiagPattern {
        id: "tuple_destruct",
        kind: "",
        triggers: &["let (", "destructur"],
        suggestion: "BMB does not support tuple destructuring. Use separate let bindings.",
        example_wrong: "let (a, b) = pair;",
        example_correct: "let a: i64 = pair_first(p);\nlet b: i64 = pair_second(p);",
    },
    DiagPattern {
        id: "match_wildcard",
        kind: "",
        triggers: &["_ =>", "_ ->"],
        suggestion: "BMB does not support underscore patterns in match. Use else.",
        example_wrong: "match x { 1 => a, _ => b }",
        example_correct: "if x == 1 { a } else { b }",
    },
    DiagPattern {
        id: "static_method",
        kind: "",
        triggers: &["::new(", "::from(", "::default(", "::with_capacity("],
        suggestion: "BMB has no static methods or associated functions. Use free functions.",
        example_wrong: "let v = Vec::new();",
        example_correct: "let v: i64 = vec_new();",
    },
    DiagPattern {
        id: "io_functions",
        kind: "",
        triggers: &["io::stdin", "read_line", "std::io", "BufRead"],
        suggestion: "BMB uses read_int() to read from stdin. No std::io.",
        example_wrong: "let mut input = String::new();\nstd::io::stdin().read_line(&mut input);",
        example_correct: "let n: i64 = read_int();",
    },
    DiagPattern {
        id: "array_syntax",
        kind: "",
        triggers: &["[i64;", "[i64 ;", "array", "&["],
        suggestion: "BMB uses vec_new()/vec_push()/vec_get() for dynamic arrays. Fixed arrays use &[T; N].",
        example_wrong: "let arr: [i64; 5] = [1, 2, 3, 4, 5];",
        example_correct: "let arr: i64 = vec_new();\nvec_push(arr, 1);\nvec_push(arr, 2);",
    },
    DiagPattern {
        id: "use_import",
        kind: "parser",
        triggers: &["`use`", "use std", "use crate"],
        suggestion: "BMB uses 'import' instead of Rust's 'use'. Standard functions are built-in.",
        example_wrong: "use std::collections::HashMap;",
        example_correct: "// No import needed — vec_new(), println(), read_int() are built-in",
    },
];
```

- [ ] **Step 4: Verify it compiles + clippy**

```bash
cd D:/data/lang-bmb && cargo build --release 2>&1 | tail -5
cd D:/data/lang-bmb && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5
```
Expected: successful compilation, no clippy warnings

- [ ] **Step 5: Commit**

```bash
git add bmb/src/diagnostics/ bmb/src/lib.rs
git commit -m "feat: diagnostics module — PatternBank with 18 AI mistake patterns"
```

---

## Task 2: PatternBank Unit Tests

**Files:**
- Create: `bmb/tests/diagnostics_test.rs` (or add to existing test module)

- [ ] **Step 1: Write tests for pattern matching**

```rust
// bmb/tests/diagnostics_test.rs
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
    let matches = find_patterns("type", "no method named `push` found");
    assert!(!matches.is_empty());
    let ids: Vec<&str> = matches.iter().map(|m| m.id).collect();
    assert!(ids.contains(&"method_call") || ids.contains(&"vec_generic"));
}

#[test]
fn test_for_loop_parser_error() {
    let matches = find_patterns("parser", "unexpected token `for`");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "for_loop");
}

#[test]
fn test_reassign_type_error() {
    let matches = find_patterns("type", "cannot assign to immutable variable");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].id, "reassign_set");
}

#[test]
fn test_no_match_for_unrelated_error() {
    let matches = find_patterns("type", "integer overflow in constant");
    // Should not match any Rust-ism patterns
    assert!(matches.is_empty() || matches.iter().all(|m| m.id != "option_type"));
}

#[test]
fn test_kind_filter_restricts_matches() {
    // "for_loop" has kind="parser" — should NOT match if kind is "type"
    let matches = find_patterns("type", "for loop not supported");
    let for_matches: Vec<_> = matches.iter().filter(|m| m.id == "for_loop").collect();
    assert!(for_matches.is_empty());
}

#[test]
fn test_case_insensitive_trigger() {
    let matches = find_patterns("", "Unknown type `Vec<i64>`");
    assert!(!matches.is_empty());
}
```

- [ ] **Step 2: Run tests**

```bash
cd D:/data/lang-bmb && cargo test diagnostics_test --release -- --nocapture
```
Expected: all tests PASS

- [ ] **Step 3: Commit**

```bash
git add bmb/tests/diagnostics_test.rs
git commit -m "test: PatternBank unit tests — 7 cases for AI pattern matching"
```

---

## Task 3: Integrate PatternBank into Error Reporting

**Files:**
- Modify: `bmb/src/error/mod.rs` — `report_error_machine` and `report_warning_machine`

- [ ] **Step 1: Modify report_error_machine**

In `bmb/src/error/mod.rs`, find `report_error_machine` (line ~907). The function uses manual JSON string formatting with `r#"..."#`. **Keep this approach** — do NOT switch to `serde_json::json!` to avoid escaping/field-order changes that could break existing tests.

After the existing `println!` that outputs the base JSON, add conditional suggestion fields. The approach: build the JSON string incrementally, appending suggestion fields only when a pattern matches.

```rust
pub fn report_error_machine(filename: &str, source: &str, error: &CompileError) {
    // ... existing kind matching code stays exactly the same ...

    let message = error.message();
    let escaped_msg = message.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    let escaped_file = filename.replace('\\', "\\\\").replace('"', "\\\"");

    let (start, end) = error.span().map(|s| (s.start, s.end)).unwrap_or((0, 0));
    let (line, col) = if start > 0 { offset_to_line_col(source, start) } else { (0, 0) };

    // Build base JSON (existing fields — no change to format)
    let mut json = format!(
        r#"{{"type":"error","kind":"{}","file":"{}","start":{},"end":{},"line":{},"col":{},"message":"{}""#,
        kind, escaped_file, start, end, line, col, escaped_msg
    );

    // AI diagnostic enrichment — append only when pattern matches
    let patterns = crate::diagnostics::find_patterns(kind, message);
    if let Some(p) = patterns.first() {
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
        json.push_str(&format!(
            r#","suggestion":"{}","example_wrong":"{}","example_correct":"{}","pattern":"{}""#,
            esc(p.suggestion), esc(p.example_wrong), esc(p.example_correct), p.id
        ));
    }

    json.push('}');
    println!("{}", json);
}
```

**Key decisions**:
- Fields omitted when no pattern matches (not null) — backward compatible
- Manual escaping preserved — exact same output for existing fields
- No `serde_json` dependency change

- [ ] **Step 2: Verify existing tests still pass**

```bash
cd D:/data/lang-bmb && cargo test --release 2>&1 | tail -5
```
Expected: all existing tests PASS (no regression)

- [ ] **Step 3: Verify JSON output includes suggestions**

```bash
# Create a test file with a Rust-ism
echo 'fn main() -> i64 = { let v: Vec<i64> = Vec::new(); 0 };' > /tmp/test_ai.bmb
D:/data/lang-bmb/target/release/bmb.exe build /tmp/test_ai.bmb -o /tmp/test_ai 2>&1
# Should show JSONL with suggestion field
```

- [ ] **Step 4: Commit**

```bash
git add bmb/src/error/mod.rs
git commit -m "feat: enrich error JSONL with AI suggestions from PatternBank"
```

---

## Task 4: Human-Readable Hint Output

**Files:**
- Modify: `bmb/src/error/mod.rs` — `report_error` (the human/ariadne path)

- [ ] **Step 1: Add hint line to human error output**

In the `report_error` function (line ~817), after the ariadne error is printed, add a hint line if a pattern matches. Note: `report_error` uses capitalized kind names ("Lexer", "Parser", etc.) — convert to lowercase for PatternBank:

```rust
// After existing ariadne output, determine kind string for pattern lookup:
let kind_lower = match error {
    CompileError::Lexer { .. } => "lexer",
    CompileError::Parser { .. } => "parser",
    CompileError::Type { .. } => "type",
    CompileError::Io { .. } => "io",
    CompileError::Parse { .. } => "parse",
    CompileError::Resolve { .. } => "resolve",
};
let patterns = crate::diagnostics::find_patterns(kind_lower, error.message());
if let Some(p) = patterns.first() {
    eprintln!("  hint: {}", p.suggestion);
    eprintln!("  wrong:   {}", p.example_wrong.lines().next().unwrap_or(""));
    eprintln!("  correct: {}", p.example_correct.lines().next().unwrap_or(""));
}
```

- [ ] **Step 2: Test human output**

```bash
echo 'fn main() -> i64 = { let v: Vec<i64> = Vec::new(); 0 };' > /tmp/test_ai.bmb
D:/data/lang-bmb/target/release/bmb.exe build /tmp/test_ai.bmb --human -o /tmp/test_ai 2>&1
# Should show ariadne error + hint line
```

- [ ] **Step 3: Commit**

```bash
git add bmb/src/error/mod.rs
git commit -m "feat: add AI hints to human-readable error output"
```

---

## Task 5: ai-proof Integration — JSON Parsing + Suggestion Feedback

**Files:**
- Modify: `ecosystem/ai-proof/orchestrator/error_normalizer.py`
- Modify: `ecosystem/ai-proof/protocol/prompt_templates.py`

- [ ] **Step 1: Extend error_normalizer to parse BMB JSONL suggestion fields**

**Do NOT replace** the existing `normalize_error` function. Instead, add a helper and enrich the existing result:

```python
# Add at the top of error_normalizer.py:
import json as _json

def _try_parse_bmb_jsonl(raw: str) -> dict:
    """Try to extract suggestion/example fields from BMB JSONL output."""
    for line in raw.strip().split("\n"):
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            data = _json.loads(line)
            if data.get("type") == "error":
                return {
                    "normalized": data.get("message", ""),
                    "location": f"{data.get('file', '')}:{data.get('line', '')}:{data.get('col', '')}",
                    "suggestion": data.get("suggestion", ""),
                    "example_wrong": data.get("example_wrong", ""),
                    "example_correct": data.get("example_correct", ""),
                }
        except (_json.JSONDecodeError, KeyError):
            continue
    return {}

# Then at the END of the existing normalize_error function, before the return:
#   if lang == "bmb" and not is_test_failure:
#       enrichment = _try_parse_bmb_jsonl(raw)
#       if enrichment:
#           result.update(enrichment)
#   # Ensure new fields exist even if not enriched
#   result.setdefault("suggestion", "")
#   result.setdefault("example_wrong", "")
#   result.setdefault("example_correct", "")
#   return result
```

This preserves all existing regex-based logic for Rust/Python and only adds BMB JSONL parsing as enrichment.

- [ ] **Step 2: Update prompt_templates to include suggestions**

```python
# In prompt_templates.py, update build_error_feedback_prompt:

def build_error_feedback_prompt(error_type: str, normalized_msg: str,
                                 location: str, raw_output: str,
                                 suggestion: str = "",
                                 example_wrong: str = "",
                                 example_correct: str = "") -> str:
    """Build error feedback. Includes AI-friendly suggestions when available."""
    raw_truncated = raw_output[:500] if len(raw_output) > 500 else raw_output

    parts = [f"{error_type}: {normalized_msg}"]
    if location:
        parts.append(f"Location: {location}")

    if suggestion:
        parts.append(f"\nSuggestion: {suggestion}")
    if example_wrong and example_correct:
        parts.append(f"Wrong: {example_wrong}")
        parts.append(f"Correct: {example_correct}")

    if not suggestion:
        # Fallback: raw output for non-enriched errors
        parts.append(f"\n{raw_truncated}")

    parts.append("\nFix the error. Output ONLY the complete corrected code in a code block.")
    return "\n".join(parts)
```

- [ ] **Step 3: Update experiment.py to pass suggestion fields through**

In `orchestrator/experiment.py`, where `build_error_feedback_prompt` is called (~line 130 and ~160), add the suggestion fields:

```python
feedback = build_error_feedback_prompt(
    error["type"],
    error["normalized"],
    error["location"],
    error["raw"],
    suggestion=error.get("suggestion", ""),
    example_wrong=error.get("example_wrong", ""),
    example_correct=error.get("example_correct", ""),
)
```

- [ ] **Step 4: Run ai-proof tests**

```bash
cd D:/data/lang-bmb/ecosystem/ai-proof && python -m pytest tests/ -v --tb=short
```
Expected: all 60 tests PASS

- [ ] **Step 5: Commit**

```bash
git add ecosystem/ai-proof/orchestrator/error_normalizer.py ecosystem/ai-proof/protocol/prompt_templates.py ecosystem/ai-proof/orchestrator/experiment.py
git commit -m "feat(ai-proof): consume enriched JSONL — suggestions in error feedback"
```

---

## Task 6: End-to-End Validation

**Files:** No new files — validation only

- [ ] **Step 1: Rebuild compiler**

```bash
cd D:/data/lang-bmb && cargo build --release
```

- [ ] **Step 2: Verify JSON output with AI mistakes**

```bash
# Test Vec pattern
echo 'fn main() -> i64 = { let v: Vec<i64> = Vec::new(); 0 };' > /tmp/test_vec.bmb
./target/release/bmb.exe check /tmp/test_vec.bmb 2>&1
# Expect: JSONL with suggestion about vec_new()

# Test println! pattern
echo 'fn main() -> i64 = { println!("hello"); 0 };' > /tmp/test_println.bmb
./target/release/bmb.exe check /tmp/test_println.bmb 2>&1
# Expect: JSONL with suggestion about println()

# Test for loop pattern
echo 'fn main() -> i64 = { for i in 0..10 { println(i); }; 0 };' > /tmp/test_for.bmb
./target/release/bmb.exe check /tmp/test_for.bmb 2>&1
# Expect: JSONL with suggestion about while loops
```

- [ ] **Step 3: Run full cargo test**

```bash
cd D:/data/lang-bmb && cargo test --release 2>&1 | tail -5
```
Expected: all tests PASS

- [ ] **Step 4: Validate pilot problems still work**

```bash
cd D:/data/lang-bmb/ecosystem/ai-proof && python scripts/validate_problems.py
```
Expected: 9/9 PASS

- [ ] **Step 5: Re-run pilot experiment**

```bash
cd D:/data/lang-bmb/ecosystem/ai-proof && rm -rf results/raw results/summary.json
python scripts/run_experiment.py --pilot --h1-only --runs 1
```

Compare Type C loop counts with previous pilot:
- Before: binary_search=7-8, quicksort=5-10
- Target: ≤ 2 per problem

- [ ] **Step 6: Commit results**

```bash
git add ecosystem/ai-proof/results/
git commit -m "results: pilot with AI-enriched errors — Type C loop reduction validated"
```

---

## Dependency Graph

```
Task 1 (PatternBank module)
  └→ Task 2 (unit tests)
       └→ Task 3 (integrate into report_error_machine)
            ├→ Task 4 (human-readable hints)
            └→ Task 5 (ai-proof integration)
                 └→ Task 6 (E2E validation + pilot re-run)
```

**Critical path**: Task 1 → 2 → 3 → 5 → 6
**Parallelizable**: Task 4 is independent after Task 3
