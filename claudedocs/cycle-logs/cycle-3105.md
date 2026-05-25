# Cycle 3105: Track B llvm_* 계약 배치 추가 (55개)
Date: 2026-05-25

## Re-plan
Cycle 3104 Carry-Forward: llvm_* 64개 계약 추가. 계획 유효.

## Scope & Implementation

**llvm_* 계약 55개 추가**:

1. **Python 배치 패치 — `pos: i64` 보유 50개**:
   - 정규식: `fn llvm_\w+\([^)]*\bpos: i64\b[^)]*\) -> \S+ =\n    ` → `pre pos >= 0`

2. **Python 배치 패치 — `p: i64` 보유 5개**:
   - `llvm_handle_mark_f64_ptr`, `llvm_handle_mark_str_ptr`, `llvm_handle_mark_str_ptr_if`, `llvm_gen_conc_stmt`, `llvm_gen_channel_new`
   - `pre p >= 0`

3. **잔여 9개 (str_sb/String 전용)**: numeric 파라미터 없어 계약 불필요, 미계약 유지
   - `llvm_gen_fn_line_structs`, `llvm_gen_line_structs`, 문자열 cmp/concat 헬퍼들
   - `str_sb: i64`는 raw pointer (포인터 값 >= 0 보장 불가) — 계약 스킵

**결과**: 1187 → 1132 (55개 감소)

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ (3231 warnings, 0 errors — 경고 수 1 감소는 unused_function 위치 변경으로 중복 제거)
- `bmb verify bootstrap/compiler.bmb`: ✅ total:1417, verified:1417, failed:0
- 미계약: 1187 → 1132 (-55)
- llvm_* 잔여: 9개 (전부 String/str_sb 전용)

## Reflection

- Scope fit: 100% (llvm_* pos/p 전부 처리)
- str_sb >= 0 계약은 raw pointer 특성상 잘못된 가정 — 올바른 판단
- 패턴 일관성: pos: i64 → pre pos >= 0, p: i64 → pre p >= 0 (tok_end와 동일 패턴)

## Carry-Forward

- Actionable: Cycle 3106 — lower_* 계약 배치 추가 (63개 예상)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: lower_* → step_/emit_/gen_/check_ → 잔여
