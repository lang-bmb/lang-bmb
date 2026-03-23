# Cycle 2108: Unified build script for all 5 library DLLs
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2107: Benchmarks complete, next was build infrastructure.

## Scope & Implementation
Created `ecosystem/build_all.py` — unified build script that:
- Compiles all 5 BMB libraries from source using `bmb build --shared --release`
- Copies DLLs to both root and bindings/python directories
- Supports per-library builds (`build_all.py bmb-algo`)
- Supports `--test` flag to run pytest after building
- Shows build time and DLL size for each library
- Cross-platform output naming (.dll/.so/.dylib)

### Build Results
| Library | Build Time | DLL Size | Tests |
|---------|-----------|----------|-------|
| bmb-algo | 8.2s | 296 KB | 189 passed |
| bmb-compute | 1.3s | 272 KB | 270 passed |
| bmb-crypto | 1.6s | 304 KB | 212 passed |
| bmb-text | 1.4s | 271 KB | 127 passed |
| bmb-json | 1.4s | 280 KB | 159 passed |
| **Total** | **23.8s** | **1,423 KB** | **957 passed** |

## Review & Resolution
- No defects found

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Run full test suite + type stubs (cycle 2109-2110)
