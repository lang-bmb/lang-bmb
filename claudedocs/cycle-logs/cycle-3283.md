# Cycle 3283: M13 Phase 5 — .bmb-contracts 구현
Date: 2026-05-29

## Re-plan
Plan valid. M13 Phase 5: .bmb-contracts 세션 영속 계약.

## Scope & Implementation
- `.bmb-contracts` 파일 형식 설계: `key = value` (# 주석)
- `contracts-check <file>` 명령 추가
- 지원 규칙:
  - `require_postcondition = true`: 함수별 post 계약 필수
  - `forbid_effect = Net`: 특정 효과 사용 금지
- 함수 post clause 탐지에 `vr_after_params_pos` 활용 (올바른 rp 계산)
- PLAT: prefix로 platform 함수 제외
- `bc_check_post_scan` 6 params 유지
- `tests/golden/test_golden_contracts_check.bmb` 작성

## Verification & Defect Resolution
- Stage 1 빌드 성공
- `require_postcondition`: missing_post 감지, has_post 안전
- `forbid_effect = Net`: net_user 감지
- Safe 케이스: {"status":"safe"} 정확
- lint: 178 warnings ✅
- **제한 사항**: platform 블록이 있는 파일에서 callers_collect_source가 platform 함수 이후 일부 함수를 건너뜀. 일반 모듈 파일에서는 정상.

## Reflection
- M13 Phase 5 ✅ COMPLETE (기본 기능)
- contracts-check 골든 테스트 확인
- 제한 사항은 callers_collect_source의 기존 platform 블록 처리 한계

## Carry-Forward
- Actionable: Fixed Point 검증, M14 Phase 4
- Structural Improvement Proposals: platform 블록 내 함수 파싱 개선 (기존 bug)
- Next Recommendation: M14 Phase 4 SemanticDuplicate
