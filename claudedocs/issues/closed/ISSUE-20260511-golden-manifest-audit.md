# ISSUE-20260511 — Golden manifest expected 값 audit 필요

**Date**: 2026-05-11 (Cycle 2693)
**Severity**: medium (false negative — 실제 회귀가 manifest 미스등록에 가려질 수 있음)

## 현상

`tests/bootstrap/golden_tests.txt`의 expected 컬럼이 stdout 첫 줄과 불일치한 항목 다수.

### 확인된 미스등록 (Cycle 2693 fix)

총 19개:
- Cycle 2651-2675 추가분: arr_str_{mut_set, var_repeat, fn_return*, struct_field*} 8개
- Cycle 2680-2683 추가분: arr_str_nested*, arr_str_triple_nested, arr_i64_baseline, arr_f64_* 9개
- Cycle 2690-2692 추가분: set_field_index_*, set_field_chain_* 7개 (즉시 정정)

### 추정 잠재 영역

70개 stratified sample 검증에서 ~7% 미스등록 발견 → 매니페스트 2875개 중 잠재 ~200개 영향 가능.

## 원인

직전 세션이 `main -> i64 = ... { 42 }` 패턴을 보고 expected를 `42`로 추정 등록했으나, golden runner는 stdout 첫 줄을 비교 (exit code 아님). println이 있는 테스트는 첫 줄이 다른 값.

## 해결 방향

### Option A — 매니페스트 일괄 audit (자율, 1-2 cycles)
스크립트로 모든 골든 빌드 → stdout 첫 줄 추출 → 매니페스트와 diff → 일괄 정정.

### Option B — 골든 runner 정책 변경 (구조적)
runner를 exit code + stdout 양쪽 검증하도록 확장. 매니페스트 포맷 `file|stdout1|exit_code` (3-tuple).

## 우선순위

- ⏳ 광범위 매니페스트 audit — 별도 cycle (자율)
- 단, **세션 종료 전 1회**는 sample 200개 정확한 PASS/FAIL 측정 필수 (직전 세션 실수 재발 방지)

## 관련

- Cycle 2693 — 19개 즉시 정정
- 직전 HANDOFF (Cycle 2680-2689) — "골든 2868 PASS" 주장은 manifest 검증 누락된 상태였음
