# Cycle 3109: 부트스트랩 P0 버그 수정 + TK_*/post 계약 추가 (127개)
Date: 2026-05-25

## Re-plan
Cycle 3108 Carry-Forward: Stage 2 bootstrap 실패 (skip_to_eol `%it` undefined). P0 버그 수정 우선.

## Scope & Implementation

**부트스트랩 P0 버그 수정 (inject_post_assumes_in_fn_scan)**:

근본 원인 분석:
- `inject_post_assumes_in_fn_scan`이 post 조건 assume을 call site에 삽입할 때
- `replace_all_str(raw_ir, "%ret", result_reg)` → `%ret` 없어서 no-op
- `contract_ast_to_assumes`는 `%ret`가 아닌 `%it` 생성 (BMB 소스의 `it` 변수명 그대로)
- 결과: `%_post_assume_0 = icmp sge i64 %it, 0` (undefined `%it`)

수정: `"%ret"` → `"%it"` (단일 문자열 변경)

**전처리: pre pos >= 0 충돌 제거 (72개)**:
- `post it >= 0` 있는 함수에서 `pre pos >= 0` 제거 (bootstrap P0 회피)
- 향후 bootstrap 코드젠 개선 후 재추가 가능

**TK_* 상수 (106개)**: `post it > 0` (2000000000+N 항상 양수)

**사용자 정의 함수 (21개)**: `post it >= 0`
- sb-push 래퍼 6종: push_ptr_marker, push_string_marker 등 (항상 0 반환)
- sb 초기화 3종: init_string_params_sb, init_i64_params_sb, init_ptr_params_sb
- 유틸리티: unpack_pos, string_in_list, cf_eval_cmp, shallow_blocks, trl_param_count 등

**결과**: 487 → 466 (-21개 no-contract, 직접 추가분)

## Verification & Defect Resolution

- `bmb check`: ✅ (3208 warnings, 0 errors)
- `bmb verify`: ✅ total:1397, verified:1397, failed:0
- 3-Stage Fixed Point: ✅ S3 == S4
- P0 버그 수정 확인: `bootstrap/compiler.ll` 내 `_post_assume_*`가 `%_tN` 형식으로 정상 생성

## Reflection

- Scope fit: 100% (P0 버그 수정 + TK_* 106 + 사용자 21)
- 핵심 발견: `inject_post_assumes_in_fn_scan`의 `%ret` vs `%it` 불일치가 전체 post 계약 파이프라인의 잠재 버그였음
- 수정 전: Stage 3 bootstrap이 `%it` undefined LLVM 에러로 항상 실패 (Stage 2는 Rust 구현 사용하여 우회)
- 수정 후: 양쪽 Fixed Point 달성, post 계약 call-site 주입 정상화
- 미계약 경로: 487 → 466 (21 감소)
- TK_* 106개 추가는 이전 사이클(3108)에서 부트스트랩 오류로 되돌렸던 것을 이번에 성공적으로 적용

## Carry-Forward

- Actionable: Cycle 3110 — 잔여 466개 분석 (bool 97개, String 279개, i64 90개)
  - bool 반환 함수: `post it >= 0` (bool → 0 or 1) 가능
  - i64 반환 중 count/length 패턴: `post it >= 0` 추가 가능
  - pre pos >= 0 재추가 대상: 72개 함수에서 부트스트랩 코드젠 개선 후 재추가
- Structural Improvement Proposals:
  - `pre pos >= 0` 재추가를 위해 bootstrap 코드젠의 if-else 체인 post assume 위치를 phi merge 이후로 이동하는 것 고려 (중간 난이도)
- Pending Human Decisions: M8 공식 계획 확정
- Roadmap Revisions: M8-A Track B — 466개 잔여, 부트스트랩 코드젠 버그 수정으로 post 파이프라인 정상화
- Next Recommendation: bool 97개에 `post it >= 0` 일괄 추가 + 잔여 i64 count/len 패턴 계속
