# Cycle 1900-1901: TRL Bug Root Cause — norecurse Fix + Deeper Investigation
Date: 2026-03-15

## Inherited → Addressed
- TRL codegen bug investigation from Cycle 1899 carry-forward

## Scope & Implementation

### 1. norecurse Indirect Recursion Fix
- **Bug found**: `pobj_ms` was marked `norecurse` despite being indirectly recursive (pobj_ms → pval → pobj → pobj_ms)
- **Fix**: Added `detect_recursive_functions()` call-graph analysis that detects cycles
- **Implementation**: BFS from each function through call graph, detect back-edges to original
- **Result**: Correct attribute — `pobj_ms` no longer has `norecurse`
- **Impact**: Independent correctness improvement, but does NOT fix the rendering bug

### 2. Root Cause Investigation
- MIR is IDENTICAL between working and failing cases
- LLVM IR for `pobj_ms` is IDENTICAL (excluding alloca order and string constant indices)
- The bug triggers when `unescape` function coexists in the same compilation unit
- Even at O0 (no LLVM optimization), the compiled binary produces wrong results
- **Key finding**: `pobj_ms` outputs count=4 without unescape, count=2 with unescape, from IDENTICAL IR
- **Hypothesis**: This may be an LLVM bug related to how functions with certain patterns interact at the codegen level. Or a very subtle text backend issue in phi node load/store emission affected by the total number of function definitions.

## Review & Resolution
- `cargo test --release`: 6,186 pass ✅
- norecurse fix is correct and beneficial ✅
- Root cause of pobj_ms bug NOT found — requires LLVM-level debugging

## EARLY TERMINATION
After 2 cycles of deep investigation, the root cause remains elusive. The bug is reproducible and well-characterized but requires LLVM-level debugging tools to resolve. The string-search workaround in the LSP server remains the pragmatic solution.

## Carry-Forward
- TRL bug: Requires disassembly-level analysis or LLVM IR mutation testing
- Next: Fix void phi merge bug (simpler), then continue LSP/ecosystem work
