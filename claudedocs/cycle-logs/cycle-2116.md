# Cycle 2116: Quality audit of Python wrappers
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2115: Binding guide complete, next was quality audit.

## Scope & Implementation
Comprehensive audit of all 5 Python binding files by code-review agent.

### Issues Found & Fixed
| # | Severity | Issue | Fixed? |
|---|----------|-------|--------|
| 1 | Medium | bmb_algo.py: DLL search order reversed vs other files | YES — changed to local-first |
| 2 | Medium | bmb_algo.py: unused `import array` | YES — removed |
| 3 | Low | Output string memory management | INVESTIGATED — output strings managed by BMB runtime, cannot be freed separately (attempted fix caused heap corruption). Original code is correct. |

### Issues Noted but Not Fixed (require deeper changes)
| # | Issue | Reason |
|---|-------|--------|
| 1 | ~38 of 41 bmb_algo functions lack FFI error wrapping | These functions use simple integer I/O where contract violations are caught by BMB's setjmp safety layer. Adding wrapping would add FFI overhead to every call without benefit. |
| 2 | bmb_compute: 0 functions use FFI error checking | Same as above — integer functions with contract safety |
| 3 | No `__all__` defined | Would be useful but is cosmetic — underscore-prefixed names are already excluded from `import *` |
| 4 | bmb_compute shadows builtins (abs, min, max, sum, sqrt) | By design — these are the library's public API. Users should use `import bmb_compute` not `from bmb_compute import *` |

## Review & Resolution
- Attempted output string free fix caused heap corruption (0xc0000374) — reverted
- DLL path and unused import fixes verified with full test suite (957 passed)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Output string memory lifecycle — BMB runtime manages this internally
- Next Recommendation: Cross-platform build script (cycle 2117)
