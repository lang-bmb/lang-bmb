# Cycle 33: LSP Module Tests + Dead Code Cleanup

## Date
2026-02-07

## Scope
Add comprehensive unit tests for LSP module (1,775 LOC, 0 tests) targeting pure functions: formatting, coordinate conversion, identifier extraction, parsing, diagnostics. Clean up incorrect `#[allow(dead_code)]` annotations.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 4/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

LSP formatting correctness ensures accurate source representation. Dead code cleanup improves code hygiene.

## Implementation

### LSP Tests (51 new)

**format_type (10 tests)**
- Primitives: i32, i64, u32, u64, f64, bool, String, char, (), !
- Compound: Named, Array, Ref, RefMut, Nullable, Tuple, Fn, Generic, Ptr
- Concurrency: Thread, Mutex, Arc, Atomic, Sender, Receiver, Barrier, Condvar

**format_expr (15 tests)**
- Literals: int, float, bool, string, unit, null
- Variables: var, ret, it
- Operations: binary (add), unary (neg, not)
- Control: if-then-else, call, array_lit, index, block, range, control flow, todo, tuple, field_access, enum_variant

**format_pattern (8 tests)**
- Wildcard, Var, Literal, EnumVariant (simple + with bindings), Tuple, Or, Range, Array, ArrayRest

**format_literal_pattern (1 test)**: Int, Float, Bool, String

**Backend methods (8 tests)**
- offset_to_position: start, same_line, multiline
- position_to_offset: start, multiline
- Roundtrip: offset→position→offset for all positions
- get_word_at_position: word found, no word

**Utility (4 tests)**
- is_ident_char: alpha, digit, underscore, non-ident
- try_parse: valid code, invalid code
- get_diagnostics: valid code (empty), syntax error (non-empty)

### Dead Code Cleanup
- Removed incorrect `#[allow(dead_code)]` from `SymbolDef.kind` field (used at line 855)
- Removed incorrect `#[allow(dead_code)]` from `SymbolKind` enum (all variants except `Variable` are constructed)
- Added targeted `#[allow(dead_code)]` on `SymbolKind::Variable` variant only
- Kept `#[allow(dead_code)]` on `DocumentState.version` (genuinely unused read)

## Issues Encountered
- I-01 (M): Type::Tuple, Type::Fn, Type::Generic use `Vec<Box<Type>>` not `Vec<Type>`
- I-02 (L): Removing `#[allow(dead_code)]` from enum revealed `Variable` variant is never constructed

## Test Results
- Rust tests: 637/637 passed (up from 586, +51 new)
  - 484 unit tests (lib) — up from 433
  - 130 integration tests
  - 23 gotgan tests
- Clippy: PASS (0 warnings)

## Score
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 637 tests pass, clippy clean |
| Architecture | 9/10 | Tests cover formatting, coordinate conversion, parsing |
| Philosophy Alignment | 9/10 | Validates LSP module correctness |
| Test Quality | 9/10 | Roundtrip testing for coordinate conversion |
| Documentation | 9/10 | Cycle log with dead code analysis |
| Code Quality | 9/10 | Dead code annotations corrected |
| **Average** | **9.2/10** | |

## Issues
- I-01 (L): LSP trait methods (hover, completion, goto_definition, references) not unit tested — require mock LSP client setup. Integration test territory.
- I-02 (L): collect_symbols, collect_expr_refs, collect_locals not tested (need AST construction). Future work.
- I-03 (L): 10 remaining `#[allow(dead_code)]` annotations in other modules not addressed this cycle.

## Next Cycle Recommendation
Cycle 34: Index module tests + REPL tests.
