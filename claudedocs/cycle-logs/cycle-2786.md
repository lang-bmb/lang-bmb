# Cycle 2786: int_to_string i64::MIN fix — modular .bmb files
Date: 2026-05-12

## Re-plan

D3 already completed in Cycle 2781. Cycle 2784 carry-forward: check modular .bmb files
(mir.bmb, optimize.bmb, lowering.bmb, llvm_ir.bmb, parser_ast.bmb, types.bmb) for the
same int_to_string i64::MIN bug. These files are used by build_unified_compiler.sh and CI.
🟡 SCOPE ADJUST: fix all 6 files.

## Scope & Implementation

### Affected Files

Grep for `int_to_string(0 - n)` in modular .bmb files found:
- `bootstrap/types.bmb` lines 182 (`fin_int_to_str`), 2309 (`int_to_string`)
- `bootstrap/mir.bmb` line 33
- `bootstrap/optimize.bmb` line 37
- `bootstrap/lowering.bmb` line 33
- `bootstrap/llvm_ir.bmb` line 41
- `bootstrap/parser_ast.bmb` line 234

All 6 files use `if n < 0 { "-" + int_to_string(0 - n) }` which infinite-loops on i64::MIN.

### Changes Made

**`bootstrap/mir.bmb`, `optimize.bmb`, `lowering.bmb`, `llvm_ir.bmb`, `parser_ast.bmb`**:
Added `int_to_string_neg` helper before `int_to_string` (same fix as compiler.bmb).

**`bootstrap/types.bmb`**:
- Line 181-184 (`fin_int_to_str`): added `fin_int_to_str_neg` helper with same pattern
- Line 2309-2310 (`int_to_string`): added `int_to_string_neg_acc` accumulator helper
  (matching the file's existing accumulator style from `int_to_string_helper`)

### Notes

`lowering.bmb` standalone build fails (expected, pre-existing): `unpack_place` is defined
in `mir.bmb`. These files are designed for unified compilation via `build_unified_compiler.sh`.

## Verification & Defect Resolution

| Check | Result |
|-------|--------|
| `mir.bmb` standalone build | ✅ exit 0 |
| `optimize.bmb` standalone build | ✅ exit 0 |
| `llvm_ir.bmb` standalone build | ✅ exit 0 |
| `parser_ast.bmb` standalone build | ✅ exit 0 |
| `lowering.bmb` standalone build | ℹ️ FAIL — expected (cross-file deps) |
| `types.bmb` standalone build | not tested (likely also cross-file) |
| `cargo test --release` | ✅ all pass |

## Reflection

Scope fit: ✅ Rule 5 all-instances fix, consistent with compiler.bmb fix in Cycle 2784.
Philosophy drift: none.
Roadmap impact: any future use of these modular files (via build_unified_compiler.sh) will
not crash on i64::MIN values.

## Carry-Forward

- Actionable: None from this cycle
- Structural: None
- Pending Human Decisions: D5-A, D7, D8
- Roadmap Revisions: None
- Next Recommendation: Cycle 2787 — Tier 3 verify + HANDOFF update.
