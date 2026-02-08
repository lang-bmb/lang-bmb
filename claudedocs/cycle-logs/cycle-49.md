# Cycle 49: Dogfood let-in-while — Convert bmb-math + bmb-base64 + bmb-rand + bmb-semver

## Date
2026-02-08

## Scope
Continue dogfooding: convert tail-recursive workaround functions in bmb-math (4), bmb-base64 (4), bmb-rand (3), and bmb-semver (1) to imperative while loops. Keep genuine recursion (Newton's method convergence, Euclidean algorithm) as-is.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- bmb-math: 6 recursive functions identified, 4 are tail-recursive workarounds (pow_iter, factorial_iter, fib_iter, is_prime_check), 2 are genuine recursion (sqrt_iter: Newton's method convergence, gcd: Euclidean algorithm)
- bmb-base64: 4 recursive functions, all tail-recursive workarounds (b64_encode_loop, b64_decode_loop, buf_to_string_loop, str_to_buf)
- bmb-rand: 3 recursive functions, all tail-recursive workarounds (rand_n, count_below_iter, count_distinct_iter)
- bmb-semver: 1 recursive function, tail-recursive workaround (parse_num_iter)
- Double-block `{ { assignment; value } }` pattern needed in bmb-rand (count updates inside if-branches)

## Implementation

### bmb-math: 4 recursive functions -> while loops (2 genuine recursion kept)

**Functions converted:**
- `pow_iter` -> inlined into `pow` with simple while loop + accumulator
- `factorial_iter` -> inlined into `factorial` with while loop + accumulator
- `fib_iter` -> inlined into `fib` with while loop + dual accumulator (a, b)
- `is_prime_check` -> inlined into `is_prime` with while + done flag + trial division

**Genuine recursion kept:**
- `sqrt_iter` -- Newton's method convergence (compares guess == prev, not a simple counter)
- `gcd` -- Euclidean algorithm (mathematical recursion, natural expression)

### bmb-base64: 4 recursive functions -> while loops

**Functions converted:**
- `b64_encode_loop` -> inlined into `b64_encode` with while loop, advancing si/di by 3/4
- `b64_decode_loop` -> inlined into `b64_decode` with while + done/error flags
- `buf_to_string_loop` -> simple while loop, removed `idx` parameter (now internal)
- `str_to_buf` -> simple while loop, removed `idx` parameter (now internal)

**Signature changes:**
- `buf_to_string_loop(sb, buf, idx, len)` -> `buf_to_string_loop(sb, buf, len)`
- `str_to_buf(s, buf, idx, len)` -> `str_to_buf(s, buf, len)`

### bmb-rand: 3 recursive functions -> while loops

**Functions converted:**
- `rand_n` -> while loop with state mutation
- `count_below_iter` -> inlined into `test_distribution` (was only used there)
- `count_distinct_iter` -> inlined into `test_no_short_cycle` (was only used there)

**Double-block pattern required:**
- `test_distribution`: `if v < 50 { { count = count + 1; 0 } } else { 0 }`
- `test_no_short_cycle`: `if ns == prev1 or ns == prev2 { { ok = 0; 0 } } else { 0 }`

### bmb-semver: 1 recursive function -> while loop

- `parse_num_iter` -> inlined into `parse_num` with while + done flag + accumulator

### Files Modified
- `ecosystem/gotgan-packages/packages/bmb-math/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-base64/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-rand/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-semver/src/lib.bmb`

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 699/699 PASS |
| Clippy | PASS (0 warnings) |
| bmb-math | 12/12 PASS |
| bmb-base64 | 15/15 PASS |
| bmb-rand | 12/12 PASS |
| bmb-semver | 14/14 PASS |
| Ecosystem total | 198/215 PASS (17 pre-existing *i64 failures) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, no regressions |
| Architecture | 9/10 | Clean separation of genuine recursion vs workarounds |
| Philosophy Alignment | 10/10 | Removed 12 workaround functions, kept 2 genuine recursion |
| Test Quality | 9/10 | All 4 packages have comprehensive test suites |
| Code Quality | 9/10 | Clean conversions, minimal double-block pattern usage |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | `desugar_block_lets` scoping bug persists: assignments inside if-branches need `{ { } }` | Future cycle: fix parser/desugarer |
| I-02 | M | bmb-args/bmb-ptr/bmb-sort have recursive workarounds but blocked by *i64 interpreter issue | Cycle 50: fix interpreter *i64 support |
| I-03 | L | bmb-base64 `buf_to_string_loop` and `str_to_buf` signature changes remove `idx` parameter | Internal functions, no external API impact |

## Cumulative Progress (Cycles 42-49)

| Cycle | Scope | Functions Converted | Key Achievement |
|-------|-------|--------------------:|-----------------|
| 42 | 4 compiler bug fixes | -- | float/int ==, free() return, let-in-block MIR, codegen %tmp |
| 43 | Grammar fix (let-in-while) | -- | BlockExpr rules, desugar_block_lets |
| 44 | Dogfood bmb-sha256 + bmb-hashmap | 17 | Validated grammar fix end-to-end |
| 45 | Version bump + commit | -- | v0.89.4, ROADMAP updated |
| 46 | Dogfood bmb-algorithms | 13 | All sorting/search/numeric converted |
| 47 | Dogfood bmb-memchr + bmb-toml | 26 | String/byte search + TOML parser converted |
| 48 | Dogfood bmb-itoa + bmb-fmt + bmb-fs | 15 | Number formatting + path utilities converted |
| 49 | Dogfood bmb-math + bmb-base64 + bmb-rand + bmb-semver | 12 | Math/encoding/PRNG/versioning converted |
| **Total** | | **83** | 83 recursive workarounds eliminated across 12 packages |

## Remaining Packages with Recursive Workarounds
| Package | Count | Status |
|---------|------:|--------|
| bmb-args | ~5 | Blocked by *i64 interpreter issue |
| bmb-ptr | ~3 | Blocked by *i64 interpreter issue |
| bmb-sort | ~5 | Blocked by *i64 interpreter issue |

## Next Cycle Recommendation
**Cycle 50**: Fix interpreter `*i64` typed pointer support to unblock bmb-args/bmb-ptr/bmb-sort conversions, then convert remaining recursive workarounds in those packages.
