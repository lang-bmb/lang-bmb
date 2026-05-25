# Cycle 3104: Track B parse_* 계약 배치 추가 (155개)
Date: 2026-05-25

## Re-plan
M7-4 COMPLETE 후 첫 M8 사이클. HANDOFF P1: parse_* 148개 `pre pos >= 0` 배치 추가. 기존 Z3 baseline: 1417/1417 (0 failures) ✅. 계획 유효.

## Scope & Implementation

**parse_* 전체 계약 추가 (155개)**:

1. **Python 배치 패치** — `pos: i64` 파라미터 보유 147개:
   - 정규식: `fn parse_\w+\([^)]*\bpos: i64\b[^)]*\) -> \S+ =\n    ` → `\1\n  pre pos >= 0\n= `
   - 일치: 147개 (예상 148개 중 1개 제외)
   - 제외된 이유: `parse_len_at(s: String, start: i64, end_pos: i64)` — `end_pos` 포함으로 substring 매칭됐으나 `pos: i64` 없음

2. **parse_len_at 별도 처리** (1개):
   - `pre start >= 0 and end_pos >= 0`

3. **잔여 parse_* 7개 수동 추가**:
   - `parse_bool_lit`, `parse_unary`, `parse_paren_expr`: `pre tok >= 0`
   - `parse_simple_assign_block`: `pre t1 >= 0 and peek1 >= 0 and peek2 >= 0`
   - `parse_build_output`: `pre argc >= 0`
   - `parse_build_fast`, `parse_build_runtime`: `pre argc >= 0`

4. **parse_source** (String만 파라미터): 수치 계약 불필요, 미계약 유지

**결과**: 1342 → 1187 (155개 감소)

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ (3232 warnings, 0 errors)
- `bmb verify bootstrap/compiler.bmb`: ✅ total:1417, verified:1417, failed:0
- parse_int_lit/parse_assert_call/parse_expr 등 Z3 검증 확인 ✅
- 미계약: 1342 → 1187 (-155)
- parse_* 잔여: 1개 (parse_source - String 전용)
- exit code 1: 기존 96개 함수 Z3 처리 불가 (변화 없음, pre-existing issue)

## Reflection

- Scope fit: 100% (parse_* 완전 처리)
- Z3 검증 정상: 155개 신규 계약 모두 통과
- 효율: Python 배치로 148개 처리, 수동으로 7개 추가
- tok/argc 계약 패턴: `tok >= 0` (packed token = kind*5000000 + endpos >= 0), `argc >= 0` — 합리적

## Carry-Forward

- Actionable: Cycle 3105 — llvm_* 계약 배치 추가 (64개 예상)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M8-A Track B 진행 중 (1342→1187, 11.6% 감소)
- Next Recommendation: llvm_* 64개 → lower_* 63개 → 잔여 계속
