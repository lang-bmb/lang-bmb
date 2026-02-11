# Cycle 253: Hex/Octal/Binary Literal Parsing

## Date
2026-02-12

## Scope
Add hex (0x/0X), octal (0o/0O), and binary (0b/0B) integer literal support to the lexer. Previously only decimal literals were supported. Underscore separators also supported (e.g. 0xFF_FF).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Lexer uses logos crate with `#[regex]` attribute macros
- Existing IntLit: `r"[0-9]+"` with priority 2
- FloatLit already uses priority 3 to take precedence
- Solution: Add 3 new regex patterns for hex/oct/bin with priority 3 (higher than decimal)
- Logos processes higher priority patterns first, preventing ambiguity with `0` prefix
- Underscore separator support via `.replace('_', "")` before parsing

## Implementation

### Lexer (`bmb/src/lexer/token.rs`)
Added 3 new regex patterns for IntLit token:
- `r"0[xX][0-9a-fA-F][0-9a-fA-F_]*"` → hex parsing via `i64::from_str_radix(_, 16)`
- `r"0[oO][0-7][0-7_]*"` → octal parsing via `i64::from_str_radix(_, 8)`
- `r"0[bB][01][01_]*"` → binary parsing via `i64::from_str_radix(_, 2)`
All with priority 3 to take precedence over decimal pattern.

### Integration Tests (`bmb/tests/integration.rs`)
Added 18 new tests:
- `test_hex_literal_basic`: 0xFF = 255
- `test_hex_literal_lowercase`: 0xff
- `test_hex_literal_uppercase_prefix`: 0XFF
- `test_hex_literal_in_arithmetic`: 0x10 + 0x20
- `test_hex_literal_with_bitwise`: 0xFF band 0x0F
- `test_hex_literal_zero`: 0x0
- `test_hex_literal_large`: 0xDEAD
- `test_octal_literal_basic`: 0o77 = 63
- `test_octal_literal_uppercase_prefix`: 0O10
- `test_binary_literal_basic`: 0b1010 = 10
- `test_binary_literal_uppercase_prefix`: 0B11111111
- `test_binary_literal_single_bit`: 0b1
- `test_hex_literal_in_function_param`: Hex as arg
- `test_hex_literal_in_let_binding`: Hex in let
- `test_hex_literal_in_comparison`: Hex in if
- `test_hex_literal_in_array_index`: Hex as size
- `test_hex_literal_with_underscore`: 0xFF_FF
- `test_binary_literal_with_underscore`: 0b1111_0000

## Test Results
- Standard tests: 3271 / 3271 passed (+18 from 3253)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All bases parse correctly, underscores work |
| Architecture | 10/10 | Uses logos priority system correctly |
| Philosophy Alignment | 10/10 | Systems language needs non-decimal literals |
| Test Quality | 10/10 | Tests all bases, contexts, edge cases |
| Code Quality | 10/10 | Minimal, focused lexer addition |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | Decimal underscore separators (1_000_000) not yet supported | Future enhancement |
| I-02 | L | Bootstrap lexer.bmb doesn't support hex/oct/bin yet | Needs bootstrap update |

## Next Cycle Recommendation
- Trait impl return type validation in type checker
- Or: Trait method dispatch in interpreter
