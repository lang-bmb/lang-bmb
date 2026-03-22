# Cycle 2013-2016: bmb-algo expansion to 34 + runtime symbol catalog
Date: 2026-03-22

## Inherited -> Addressed
- Cycle 2009: No carry-forward

## Scope & Implementation

### 7 New Algorithms in bmb-algo
| Algorithm | Description |
|-----------|------------|
| bit_popcount | Count set bits in integer |
| array_rotate | Rotate array left by k positions |
| unique_count | Count distinct values in sorted array |
| prefix_sum | In-place prefix sum |
| array_sum | Sum of elements |
| array_min | Minimum value |
| array_max | Maximum value |

### Runtime Symbol Collision: bmb_popcount
- Renamed to `bmb_bit_popcount` (runtime already has `bmb_popcount`)
- **Pattern identified**: BMB runtime exports many `bmb_*` symbols
- All library @export functions must avoid: bmb_abs, bmb_min, bmb_max, bmb_clamp,
  bmb_pow, bmb_popcount, bmb_string_hash, and others in bmb_runtime.c

## Review & Resolution
- bmb-algo Python: 34 algorithms working ✅
- All values verified (popcount(255)=8, rotate, unique_count, prefix_sum, etc.)

## Carry-Forward
- Pending Human Decisions: None
- Next Recommendation: Quality pass + comprehensive test update
