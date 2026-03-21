# Cycle 1931: stdlib math tests
Date: 2026-03-21

## Inherited → Addressed
- Cycle 1930 clean

## Scope & Implementation
- Created `tests/stdlib/test_math.bmb` — 30 test functions covering:
  - Float ops: fabs, fsign, fmin, fmax, fclamp
  - Integer power: ipow (base case, square, cube, power of 2)
  - Number theory: gcd, lcm
  - Sequences: factorial, fibonacci
  - Roots: isqrt (perfect, non-perfect, boundary cases)
- All tests pass `bmb check` ✅
- Note: json and collections modules depend on vec_* builtins, can only test in interpreter mode

## Review & Resolution
- test_math.bmb: ✅ (57 warnings — semantic_duplication + unused_function, expected for test files)
- No defects found

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: json/collections tests require interpreter (`bmb run`) not just `bmb check`
- Next Recommendation: Cycle 1932 — fs module with runtime builtins
