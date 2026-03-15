# Cycle 1902: Void Type Phi Merge Fix
Date: 2026-03-15

## Inherited → Addressed
- Void type phi merge bug from Cycle 1899 carry-forward: **FIXED**

## Scope & Implementation

### Void Type Phi/Load Fix
- **Bug**: When nested if-else returns `()`, the text codegen generated `load void, ptr %addr` and `phi void` which are invalid LLVM IR
- **Root cause**: phi_load_map emits loads for ALL local variables in predecessor blocks, including void-typed ones. And the phi emission doesn't check for void type.
- **Fix**:
  1. Skip `load` emission when `llvm_ty == "void"` (line 2328)
  2. Skip `phi` emission when `ty == "void"` (line 5270)
- **Alloca**: Already handled (skips void types)
- **Result**: Functions returning `()` with nested if-else now compile correctly

## Review & Resolution
- `cargo test --release`: 6,186 pass ✅
- Bootstrap IR: zero void loads/stores/phis ✅
- Test: nested if-else returning () compiles and runs correctly ✅

## Carry-Forward
- None from this cycle
- Next: Continue with LSP diagnostic position fix or Phase 3/4
