# Cycle 3095: `suggest_contracts` MCP Tool (mcp_server.bmb)
Date: 2026-05-25

## Re-plan
계획 유효. mcp_server.bmb에 `suggest_contracts` tool 추가.

## Scope & Implementation

**신규 함수 (mcp_server.bmb에 추가)**:
- `str_find_from(s, pattern, start)`: 3-인수 str_find 대체 헬퍼
- `extract_param_name`: "name: Type" → "name" 추출
- `param_is_pos_like` / `param_is_len_like`: 파라미터 이름 heuristic
- `fn_name_is_bool_like` / `fn_name_is_nonneg_like`: 함수명 패턴
- `make_pre_suggestion` / `make_post_suggestion`: JSON 제안 빌더
- `suggest_for_i64_param`: 개별 파라미터 계약 제안
- `extract_fn_name` / `extract_ret_type` / `extract_params_block`: 서명 파싱
- `process_params`: 파라미터 목록 재귀 처리
- `tool_suggest_contracts`: MCP tool 핸들러

**handle_tools_list + handle_tools_call에 등록**: 9번째 tool.

**BMB 타입 이슈 수정**:
- `str_starts_with` returns `i64` → `== 1` 비교 필요 (if 조건에서)
- `and` 연산자 → 중첩 if로 대체 (bool/i64 혼동)
- `str_find(s, p, pos)` 3-인수 → `str_find_from` 헬퍼 사용

## Verification & Defect Resolution

- `bmb check mcp_server.bmb`: ✅ (warnings only, no errors)
- `suggest_contracts` for `find_separator(s: String, pos: i64)`:
  → `pre pos >= 0` (high) + `post it >= 0` (medium) ✅
- `suggest_contracts` for `is_whitespace(c: i64)`:
  → `[]` (c는 character code, 위치 아님) ✅ 올바른 음수 결과

## Reflection

- Scope fit: 100%
- `fn_name_is_bool_like` 미사용 — 향후 bool 반환 분석 시 사용 예정, orphan 아님
- heuristic 품질: pos/idx/start/end 파라미터군 커버, find_/skip_ 함수명 패턴 커버
- `str_starts_with` 반환 i64 이슈 — BMB 문법 주의 사항으로 확인

## Carry-Forward

- Actionable: Cycle 3096 — Track B 자동화 스크립트 (`list-uncontracted.bmb`)
- Structural Improvement Proposals: fn_name_is_bool_like 향후 bool 반환 타입 분석과 통합
- Pending Human Decisions: None
- Roadmap Revisions: M7-4 Phase 2 완료 (suggest_contracts ✅)
- Next Recommendation: Track B 자동화 스크립트로 미계약 함수 목록 활용
