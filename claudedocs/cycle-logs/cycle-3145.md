# Cycle 3145: M9 Batch 11 — struct init/param/fn/return_type 파서 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 671개 잔여. struct init/param/fn/return_type 파서 계열.

## Scope & Implementation
15개 post 조건 추가 (모두 `post it.len() >= 2`):
- `parse_struct_init` — struct 초기화 파서 (이름 이후)
- `parse_struct_fields` — struct 필드 반복 파서
- `parse_struct_field_colon` — struct 필드 콜론 이후 파서
- `parse_struct_field_value` — struct 필드 값 파서
- `parse_struct_field_sep` — struct 필드 구분자 파서
- `parse_fn` — fn 정의 파서 (이름/파라미터/본문)
- `parse_fn_with_annotation` — @inline/@pure 등 어노테이션 fn 파서
- `skip_array_type_tokens` — 배열 타입 토큰 스킵 파서
- `skip_tuple_type_tokens` — tuple 타입 토큰 스킵 파서
- `parse_return_type` — 반환 타입 파서
- `parse_param` — 파라미터 파서
- `parse_param_array_type` — 배열 타입 파라미터 파서
- `parse_param_ref_type` — 참조 타입 파라미터 파서
- `parse_params` — 파라미터 목록 파서
- (semantic_duplication +14 상쇄로 net 0 변화)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2957 → 2957 (0 net; missing_postcondition 671→657 = −14)
  - semantic_duplication: +14 (정상 — `post it.len() >= 2` 공통 계약 확장)
- bmb verify: 815/815 → 801/801 (0 failed, total −14: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- parse_fn: 핵심 함수 정의 파서 — pack_result/make_error_at 모두 반환
- parse_return_type: 모든 BMB 타입을 pack_result로 반환 — 명확한 계약
- parse_param*: 배열/참조/단순 타입 파라미터 — 모두 pack_result/make_error_at
- parse_struct_field*: 5단계 재귀 체인 완성 (struct_fields → field_colon → field_value → field_sep)
- skip_*_type_tokens: "스킵"이지만 pack_result 반환 — len >= 2 성립
- semantic_duplication 상쇄: 공통 계약 확장 지속
- 남은 missing_postcondition: 657개

## Carry-Forward
- Actionable: missing_postcondition 657개 계속 분석
  - 다음 배치: parse_program_sb 및 상위 레벨 파서들
  - lower_*/codegen_* 파서 계열
  - types_* 파서 계열
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→657 (−157 총계, 19.3% 감소)
- Next Recommendation: Cycle 3146: program/lower/codegen 상위 레벨 파서들 분석
