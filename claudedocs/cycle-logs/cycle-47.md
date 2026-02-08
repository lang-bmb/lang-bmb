# Cycle 47: Dogfood let-in-while — Convert bmb-memchr + bmb-toml

## Date
2026-02-08

## Scope
Continue dogfooding from Cycles 44/46: convert all recursive workaround functions in bmb-memchr (14) and bmb-toml (12) to imperative while loops with let bindings.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

Per CLAUDE.md Principle 2: removing workarounds that were forced by a language defect.

## Research Summary
- bmb-memchr: 14 recursive workaround functions identified
- bmb-toml: 12 recursive workaround functions identified (11 `_iter` helpers + `find_table_close`)
- Discovered parser limitation: `{ assignment; value }` blocks inside `if` branches within while loops containing preceding `let` bindings require double-block wrapping `{ { assignment; value } }`
- Root cause: `desugar_block_lets` doesn't correctly scope assignments inside if-branch blocks when preceded by let bindings in the same while body

## Implementation

### bmb-memchr: 14 recursive functions → while loops

**Byte search** (5 functions eliminated)
- `str_find_byte_from` → inlined into `str_find_byte` with while + done flag
- `str_rfind_byte_from` → inlined into `str_rfind_byte` with while + done flag
- `str_count_byte_from` → inlined into `str_count_byte` with simple while loop
- `str_find_byte2_from` → inlined into `str_find_byte2` with while + done flag
- `str_find_byte3_from` → inlined into `str_find_byte3` with while + done flag

**String search** (5 functions eliminated)
- `str_match_at` → changed signature (removed `n_pos` param), uses internal while loop
- `str_find_from` → inlined into `str_find` with while + done flag
- `str_rfind_from` → inlined into `str_rfind` with while + done flag
- `str_count_from` → inlined into `str_count` with while loop
- `str_find_nth_byte_from` → inlined into `str_find_nth_byte` with while + done flag

**Buffer search** (4 functions eliminated)
- `buf_find_byte_from` → inlined into `buf_find_byte` with while + done flag
- `buf_rfind_byte_from` → inlined into `buf_rfind_byte` with while + done flag
- `buf_match_at` → changed signature (removed `n_pos` param), uses internal while loop
- `buf_find_from` → inlined into `buf_find` with while + done flag

### bmb-toml: 12 recursive functions → while loops

**Tokenization** (3 functions eliminated)
- `skip_ws_iter` → inlined into `skip_ws` with simple while loop
- `skip_to_eol_iter` → inlined into `skip_to_eol` with while + done flag
- `skip_ws_comment` → converted from tail-recursive to while loop

**String parsing** (1 function eliminated)
- `find_string_end_iter` → inlined into `find_string_end` with while + `escaped` state

**Key parsing** (1 function eliminated)
- `parse_bare_key_iter` → inlined into `parse_bare_key` with simple while loop

**Integer parsing** (2 functions eliminated)
- `parse_int_iter` → inlined into `parse_integer` with while loop + accumulator
- `find_int_end_iter` → inlined into `find_int_end` with while loop

**Table parsing** (1 function eliminated)
- `find_table_close` → converted from recursion to while + done flag

**Validation/counting** (4 functions eliminated)
- `is_valid_toml_iter` → inlined into `is_valid_toml` with while + done flag + valid flag
- `count_keyvals_iter` → inlined into `count_keyvals` with simple while loop
- `count_tables_iter` → inlined into `count_tables` with simple while loop
- `has_section_iter` → inlined into `has_section` with while + done + found flags

### Parser Issue Discovered
- Blocks `{ assignment; value }` inside if-branches within while loops with preceding let bindings require double-block wrapping `{ { assignment; value } }`
- Affects: `parse_integer`, `find_int_end`, `count_keyvals`, `count_tables`
- Root cause: `desugar_block_lets` doesn't create separate scope for if-branch blocks

### Files Modified
- `ecosystem/gotgan-packages/packages/bmb-memchr/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-toml/src/lib.bmb`

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 699/699 PASS |
| Clippy | PASS (0 warnings) |
| bmb-memchr | 28/28 PASS |
| bmb-toml | 22/22 PASS |
| All ecosystem (21 packages) | 198/215 PASS (17 pre-existing *i64 failures in bmb-args/bmb-ptr/bmb-sort) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, code produces identical results |
| Architecture | 9/10 | While loops cleaner than recursive helpers; double-block workaround needed in some cases |
| Philosophy Alignment | 10/10 | Removed 26 recursive helper functions that were workarounds |
| Test Quality | 8/10 | Existing test suites validate correctness (28 + 22 = 50 tests) |
| Documentation | 8/10 | Cycle log documents all conversions and parser issue |
| Code Quality | 8/10 | Fewer functions, clearer control flow; double-block pattern is verbose |
| **Average** | **8.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | `desugar_block_lets` scoping bug: `{ assignment; value }` inside if-branch within while body with let bindings requires `{ { ... } }` workaround | Future cycle: fix parser/desugarer |
| I-02 | L | `done` flag + `{ }` block wrapping pattern remains verbose; `break`/`continue` would be cleaner | Language design consideration |
| I-03 | M | bmb-ptr/bmb-sort/bmb-args still can't run in interpreter (*i64 typed pointers) | Cycle 50 |

## Cumulative Progress (Cycles 42-47)

| Cycle | Scope | Functions Converted | Key Achievement |
|-------|-------|--------------------:|-----------------|
| 42 | 4 compiler bug fixes | — | float/int ==, free() return, let-in-block MIR, codegen %tmp |
| 43 | Grammar fix (let-in-while) | — | BlockExpr rules, desugar_block_lets |
| 44 | Dogfood bmb-sha256 + bmb-hashmap | 17 | Validated grammar fix end-to-end |
| 45 | Version bump + commit | — | v0.89.4, ROADMAP updated |
| 46 | Dogfood bmb-algorithms | 13 | All sorting/search/numeric algorithms converted |
| 47 | Dogfood bmb-memchr + bmb-toml | 26 | String/byte search + TOML parser converted |
| **Total** | | **56** | 56 recursive workarounds eliminated across 5 packages |

## Next Cycle Recommendation
**Cycle 48**: Convert bmb-itoa (10) + bmb-fmt (5) + bmb-fs (7) = 22 more workarounds
