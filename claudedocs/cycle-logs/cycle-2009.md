# Cycle 2009-2012: bmb-compute library creation
Date: 2026-03-22

## Inherited -> Addressed
- Cycle 2005: No actionable carry-forward

## Scope & Implementation

### New Library: bmb-compute (20 @export functions)
| Category | Functions |
|----------|----------|
| Math | c_abs, c_min, c_max, c_clamp, sign, ipow, sqrt, factorial |
| Statistics | sum, mean_scaled, c_min_val, c_max_val, range_val, variance_scaled |
| Random | rand_seed, rand_next, rand_pos, rand_range |
| Vector | dot_product, dist_squared |

### Runtime symbol conflicts resolved
- `bmb_abs`, `bmb_min`, `bmb_max`, `bmb_clamp`, `bmb_pow` conflict with runtime
- Renamed to `bmb_c_abs`, `bmb_c_min`, `bmb_c_max`, `bmb_c_clamp`, `bmb_ipow`
- `min_val`, `max_val` renamed to `bmb_c_min_val`, `bmb_c_max_val`

### Files created
- `ecosystem/bmb-compute/src/lib.bmb`: 270 lines, 20 @export functions
- `ecosystem/bmb-compute/bindings/python/bmb_compute.py`: 20 wrappers + 28 tests

## Review & Resolution
- bmb-compute standalone: All outputs correct ✅
- bmb-compute Python: 28/28 tests PASS ✅

## Carry-Forward
- Pending Human Decisions: None
- Next Recommendation: More gotgan integration + quality pass
