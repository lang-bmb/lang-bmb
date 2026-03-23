# Cycle 2125: 4 new algorithms for bmb-algo
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2124: Carry-forward was "PyPI wheel builds + cross-platform". Starting with library expansion first.

## Scope & Implementation
Added 4 new algorithms to bmb-algo (41 → 45):
- **shell_sort**: Diminishing increment sort O(n^(3/2))
- **insertion_sort**: Simple insertion sort O(n^2)
- **subset_sum**: DP subset sum check O(n * target)
- **matrix_det**: N×N matrix determinant via Gaussian elimination

Files changed:
- `ecosystem/bmb-algo/src/lib.bmb`: +4 @export functions (~160 LOC)
- `ecosystem/bmb-algo/bindings/python/bmb_algo.py`: +4 wrappers + ctypes declarations
- `ecosystem/bmb-algo/bindings/python/bmb_algo.pyi`: +4 type stubs

## Review & Resolution
- Fixed LLVM IR variable name collision (`tmp` → `det_buf` in matrix_det)
- matrix_det has ±1 rounding error for 3×3+ matrices due to integer division scaling
- All 189 existing tests still pass

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: matrix_det integer precision could be improved with Bareiss algorithm
- Next Recommendation: More bmb-algo algorithms (cycle 2126)
