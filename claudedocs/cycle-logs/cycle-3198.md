# Cycle 3198: TK_* cluster postcondition 정확화 (semantic_duplication 1119→1016)
Date: 2026-05-27

## Re-plan
Plan valid. 상속 범위: semantic_duplication 1,119개 분석 + non_snake_case 108개 처리.

STEP 0 결과:
- semantic_duplication 알고리즘: (sig_key=(param_types → ret_type), post_key=postcondition_text) 동일 시 경고
- 122개 anchor cluster, 최대: TK_FN(105), skip_to_eol(65), scan_int(65), do_step(59), collect_lambda_params(57)
- TK_* cluster가 가장 명확: 각 함수가 고유 정수 상수를 반환하므로 `post it == VALUE` 로 완전 차별화 가능

## Scope & Implementation

**분석**:
- `bmb/src/types/mod.rs:1627`: sig_key = `"(param_types) -> ret_type"`, post_key = `format_expr(post)`
- 동일 (sig, post) 쌍을 `contract_signatures` HashMap으로 추적, 첫 등장 함수가 anchor

**TK_* cluster 수정 (106개 TK_* 함수)**:
- 모든 TK_* 함수: `() -> i64`, `post it > 0` → `post it == 2000000000 + N`
- Python script: regex 패턴으로 각 함수의 반환 표현식 캡처 → postcondition 교체
- 106개 치환 완료

**발견된 실제 토큰 ID 중복**:
- TK_BREAK = TK_AS = 2000000000 + 127 (동일 값)
- TK_LOOP = TK_BXOR = 2000000000 + 131 (동일 값)
- → 2개 경고는 정당한 semantic_duplication (정합성 이슈, 별도 판단 필요)

## Verification & Defect Resolution
- `bmb check`: semantic_duplication 1119→**1016** (−103), non_snake_case: 108, 총 1125
- Stage 1 bootstrap: ✅ `{"type":"build_success","output":"bootstrap/compiler.exe"}`

## Reflection
- **Scope fit**: TK_* 105개 목표 대비 103개 달성 (2개는 실제 토큰 ID 충돌로 정당한 경고)
- **TK_BREAK/TK_AS, TK_LOOP/TK_BXOR 충돌**: bootstrap compiler에서 토큰 ID 재사용이 의도적인지 버그인지 Human Decision 필요
- **알고리즘 이해 완료**: 나머지 클러스터 처리 전략 수립 가능
- **non_snake_case**: 다음 사이클에서 처리

## Carry-Forward
- Actionable: non_snake_case 108개 → Cycle 3199에서 lint 수정 (ALL_UPPER_CASE 예외)
- Actionable: semantic_duplication 1016개 (skip_to_eol 65, scan_int 65, do_step 59, collect_lambda_params 57, ...)
- Structural Improvement Proposals: TK_BREAK=TK_AS=127, TK_LOOP=TK_BXOR=131 토큰 ID 충돌 — 의도적이라면 주석 추가, 버그라면 수정
- Pending Human Decisions: TK_BREAK/TK_AS 토큰 ID 충돌 의도 확인
- Roadmap Revisions: None
- Next Recommendation: Cycle 3199 — non_snake_case lint fix (ALL_UPPER_CASE 예외 처리)
