# Cycle 1895: TRL Codegen Bug Investigation + LSP String-Search Workaround
Date: 2026-03-15

## Inherited → Addressed
- TRL+CopyProp codegen bug from Cycle 1893-1894: **INVESTIGATED, WORKAROUND APPLIED**

## Scope & Implementation

### 1. TRL Bug Investigation
- Examined TailRecursiveToLoop implementation (optimize.rs:3780-4050)
- Examined CopyPropagation implementation (optimize.rs:2027-2070)
- Examined text codegen phi node handling (llvm_text.rs:1935-2023, 5126+)
- **Finding**: Generated LLVM IR is correct — phi nodes have proper edges, loop structure is sound
- **Finding**: CopyPropagation's `copies.retain(|_, v| v.name != dest_name)` fix handles TRL loops correctly
- **Hypothesis**: The bug may be in a subtler interaction between multiple optimization passes
  (possibly AggressiveInlining or CommonSubexpressionElimination interacting with TRL-generated loops)
- **Status**: Root cause not conclusively identified. Filed as issue for dedicated investigation.

### 2. LSP String-Search Workaround
- Replaced JSON parser-based dispatch with direct string search:
  - `json_extract_str(msg, key)` — finds `"key":"value"` pattern
  - `json_extract_num(msg, key)` — finds `"key":number` pattern
  - `str_find(haystack, needle, pos)` — substring search
  - `str_find_char(s, ch, pos)` — character search
- This completely bypasses the JSON parser for method/id extraction
- LSP initialize/shutdown/exit handshake: ✅ working

## Review & Resolution
- `cargo test --release`: 6,186 pass ✅
- LSP handshake verified with Python test ✅
- TRL bug filed as issue, workaround applied ✅

## EARLY TERMINATION
The TRL codegen bug requires a dedicated debugging session with MIR dump comparison between working and failing cases. The workaround is sufficient for the LSP server MVP.

## Carry-Forward
- **TRL Codegen Bug**: Requires dedicated session with MIR-level debugging (dump MIR before/after each pass)
- Next Recommendation: Continue LSP development (didOpen diagnostics testing) or move to Phase 3 (Bootstrap SAE/nonnull)
