# Cycle 2889: to_string<f64>/<bool> Native Fix + str_hashmap_keys Complete
Date: 2026-05-15

## Re-plan
Scope adjust. While exploring to_string native porting, discovered:
- `to_string(i64)` → already worked (mapped to `int_to_string` → `bmb_int_to_string`)
- `to_string(f64)` → `bmb_f64_to_string` called by MIR but not registered in inkwell backend
- `to_string(bool)` → `bmb_bool_to_string` same problem

## Scope & Implementation

1. **Inkwell backend** (`llvm.rs`) — missing registrations (P0 fix):
   - `bmb_f64_to_string(double) → ptr` registered with key `"bmb_f64_to_string"`
   - `bmb_bool_to_string(i1) → ptr` registered with key `"bmb_bool_to_string"`
   - Both added after `bmb_int_to_string` registration

2. **Test**: `native_to_string.bmb`: to_string(3.14) → "3.14", to_string(true) → "true", to_string(false) → "false"

Note: text backend already had `declare` for both, and MIR lower.rs already generated the correct call names. Only the inkwell registration was missing.

## Verification & Defect Resolution

- Interpreter: 3.14, true, false ✅
- Native (inkwell): 3.14, true, false ✅
- `cargo test --release -p bmb` → (실행 중, 기준 2388 PASS)

## Reflection

- **Scope fit**: P0 fix (inkwell backend missing registrations) discovered during Cycle 2889 research.
- **Root cause**: Text backend already had declarations since Cycle 2881/2883, but inkwell backend was never updated with these functions. CLAUDE.md Rule 7 (양쪽 백엔드 동기화) 위반이었음.
- **to_string<String>**: Identity case — MIR directly returns arg, no function call. Already works.
- **to_string<i32/char>**: Would need `char_to_string` or `int_to_string` depending on context.

## Carry-Forward
- Actionable: `format()` variadic native porting — complex, investigate approach
- Actionable: `read_line()` / stdin native porting
- Structural Improvement Proposals: Audit for other text/inkwell backend sync gaps (Rule 7)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2890 — format() variadic assessment + read_line native
