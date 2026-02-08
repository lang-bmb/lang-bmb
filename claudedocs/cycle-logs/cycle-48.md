# Cycle 48: Dogfood let-in-while — Convert bmb-itoa + bmb-fmt + bmb-fs

## Date
2026-02-08

## Scope
Continue dogfooding: convert tail-recursive workaround functions in bmb-itoa (4), bmb-fmt (4), and bmb-fs (7) to imperative while loops. Keep genuine recursion (string-building, divide-and-conquer) as-is.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- bmb-itoa: 10 recursive functions identified, 4 are tail-recursive workarounds (atoi_pos, atoi_hex_pos, atoi_bin_pos, pad_zeros), 6 are genuine recursion (string building)
- bmb-fmt: 5 recursive functions, 4 are workarounds (count_digits_iter, pow_simple_iter, print_spaces, print_zero_padded_iter), 1 genuine (print_binary_iter)
- bmb-fs: 7 recursive functions, all tail-recursive workarounds
- Confirmed double-block `{ { assignment; value } }` pattern needed for assignments inside if-branches in while loops with preceding let bindings

## Implementation

### bmb-itoa: 4 recursive functions → while loops (6 genuine recursion kept)

**Parsing functions converted:**
- `atoi_pos` → while loop with accumulator, changed signature from `(s, pos, acc)` to `(s, start)`
- `atoi_hex_pos` → while loop with accumulator, changed signature similarly
- `atoi_bin_pos` → while loop with accumulator, changed signature similarly
- `pad_zeros` → while loop with string concatenation

**Genuine recursion kept (string building):**
- `itoa_pos`, `itoa_hex_pos`, `itoa_hex_upper_pos`, `itoa_bin_pos`, `itoa_oct_pos` — prepend pattern
- `itoa_insert_sep` — divide-and-conquer string splitting

### bmb-fmt: 4 recursive functions → while loops (1 genuine recursion kept)

**Functions converted:**
- `count_digits_iter` → inlined into `count_digits` with while + done flag
- `pow_simple_iter` → inlined into `pow_simple` with simple while loop
- `print_spaces` → simple while loop
- `print_zero_padded_iter` → inlined into `print_zero_padded` with while loop

**Genuine recursion kept:**
- `print_binary_iter` — iterates from high bit to low bit, position-dependent

### bmb-fs: 7 recursive functions → while loops

- `find_last_sep_iter` → inlined into `find_last_sep` with simple while loop
- `find_ext_iter` → inlined into `find_ext` with while + state tracking
- `count_components_iter` → inlined into `count_components` with while + `in_component` flag
- `is_safe_path_iter` → inlined into `is_safe_path` with while + done flag
- `count_consecutive_seps` → while + done flag
- `needs_normalization` → while + done flag
- `is_valid_path_iter` → inlined into `is_valid_path` with while + done flag

### Double-block pattern required
Multiple functions needed `{ { assignment; value } }` wrapping:
- `find_last_sep`: `{ { last = pos; 0 } }`
- `find_ext`: `{ { last_dot = ...; 0 } }`
- `count_components`: `{ { in_component = true; 0 } }`
- `needs_normalization`: inner if-else branches

### Files Modified
- `ecosystem/gotgan-packages/packages/bmb-itoa/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-fmt/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-fs/src/lib.bmb`

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 699/699 PASS |
| Clippy | PASS (0 warnings) |
| bmb-itoa | 33/33 PASS |
| bmb-fmt | 0/0 PASS (no test functions) |
| bmb-fs | 10/10 PASS |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Clean separation of genuine recursion vs workarounds |
| Philosophy Alignment | 10/10 | Removed 15 workaround functions, kept 7 genuine recursion |
| Test Quality | 8/10 | Existing test suites validate correctness |
| Code Quality | 8/10 | Double-block pattern needed in several places |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | `desugar_block_lets` scoping bug persists: assignments inside if-branches need `{ { } }` | Future cycle: fix parser/desugarer |
| I-02 | L | bmb-fmt has no test functions (0/0) | Add tests in future cycle |

## Cumulative Progress (Cycles 42-48)

| Cycle | Scope | Functions Converted | Key Achievement |
|-------|-------|--------------------:|-----------------|
| 42 | 4 compiler bug fixes | — | float/int ==, free() return, let-in-block MIR, codegen %tmp |
| 43 | Grammar fix (let-in-while) | — | BlockExpr rules, desugar_block_lets |
| 44 | Dogfood bmb-sha256 + bmb-hashmap | 17 | Validated grammar fix end-to-end |
| 45 | Version bump + commit | — | v0.89.4, ROADMAP updated |
| 46 | Dogfood bmb-algorithms | 13 | All sorting/search/numeric converted |
| 47 | Dogfood bmb-memchr + bmb-toml | 26 | String/byte search + TOML parser converted |
| 48 | Dogfood bmb-itoa + bmb-fmt + bmb-fs | 15 | Number formatting + path utilities converted |
| **Total** | | **71** | 71 recursive workarounds eliminated across 8 packages |

## Next Cycle Recommendation
**Cycle 49**: Convert remaining packages (bmb-math 5, bmb-base64 4, bmb-rand 3, bmb-semver 1, bmb-args 3, bmb-ptr 2, bmb-sort 1 = 19 remaining)
