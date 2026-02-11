# Cycle 262: Debug Format Cleanup in Error Messages

## Date
2026-02-12

## Scope
Replace `{:?}` (Rust Debug format) with `{}` (Display format) in user-facing error messages across interpreter, type checker, build system, resolver, error module, and main CLI.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 4/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Found 31 instances of `{:?}` in user-facing error messages across the codebase
- Type enum already has Display impl — no reason to use Debug
- Span had no Display impl — added one
- Key files affected: interp/eval.rs, types/mod.rs, build/mod.rs, resolver/mod.rs, error/mod.rs, main.rs

## Implementation

### Span Display (`bmb/src/ast/span.rs`)
- Added `impl Display for Span` — formats as `start..end`

### Error Message Fixes
| File | Fix |
|------|-----|
| `interp/eval.rs:1467-1468` | Cast error: `{:?}` → `{}` for Type |
| `interp/eval.rs:307` | Impl block type name: `{:?}` → `{}` for Type |
| `types/mod.rs:2876` | Cast validation: `{:?}` → `{}` for Type |
| `build/mod.rs:425` | Type check error: `{:?}` → `{}` for CompileError |
| `resolver/mod.rs:186` | Module not found: `{:?}` → `.display()` for PathBuf |
| `error/mod.rs:435` | Resolve error span: `{:?}` → `{}` for Span |
| `main.rs:3051` | compile_program error: `{:?}` → `{}` for type_name |

## Test Results
- Standard tests: 3327 / 3327 passed (no new tests, quality-only change)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All Display impls verified |
| Architecture | 10/10 | Consistent Display usage |
| Philosophy Alignment | 9/10 | UX improvement, not performance |
| Test Quality | 8/10 | No new tests (formatting change) |
| Code Quality | 10/10 | Human-readable error messages |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | ~20 more {:?} in panic!/internal code | Low priority, not user-facing |
| I-02 | L | Lexer {:?} for unexpected chars kept deliberately | Provides escaping for non-printable chars |

## Next Cycle Recommendation
- Additional quality improvements
- Interpreter feature gaps
- WASM codegen improvements
