# Cycle 3036: M6-P1 추가 도구 구현 (bmb_verify + 파일 검색 도구)
Date: 2026-05-22

## Re-plan
Cycle 3035 carry-forward: 추가 도구 구현. Plan valid.

## Scope & Implementation

### 추가된 도구 (3종)
1. **`bmb_verify`** — Z3 contract 검증: `exec_output(bmb, "verify " + path)` 
2. **`bmb_spec_lookup`** — SPECIFICATION.md 키워드 검색
3. **`bmb_example`** — BY_EXAMPLE.md 키워드 검색

### 파일 검색 도구 구현 패턴
- `find_repo_root()` → `getenv("BMB_REPO_ROOT")` (fallback: "" → tool error)
- `read_file(spec_path)` → `str_split(content, "\n##")` → `SvecHandle`
- `collect_matches(sections: SvecHandle, ...)` + `count_matches(...)` — 재귀 탐색
- Case-insensitive: `str_to_lower(s).contains(query)`

### 버그 수정 (STEP 4에서 발견)
- `bmb_spec_lookup` / `bmb_example` 반환값이 `tool_ok_result()`로 래핑되지 않았음
  → `send_response(id, raw_json)` 대신 `send_response(id, tool_ok_result(body))`로 수정

### 함수 서명 교정
- `collect_matches(sections: i64, ...)` → `(sections: SvecHandle, ...)` (타입 검사기 요구)

## Verification & Defect Resolution

**Type check**: ✅ success (109 warnings, all lint/postcondition — no errors)

**통합 테스트** (모두 ✅):
| 도구 | 테스트 | 결과 |
|------|--------|------|
| `bmb_spec_lookup("contracts")` | 19 총 매칭, 3 반환 | ✅ |
| `bmb_example("struct")` | 3 총 매칭, 2 반환 | ✅ |
| `bmb_verify` | postcondition 위반 정확히 감지 | ✅ |
| `bmb_check` (type error) | type error 메시지 반환 | ✅ |

## Reflection
- **Scope fit**: 3 도구 추가 완료. 총 7 도구 (bmb_check, bmb_run, bmb_lint, bmb_ir, bmb_verify, bmb_spec_lookup, bmb_example).
- **Latent defects**: `bmb_spec_lookup`이 "contracts" 검색 시 문서 서두(철학 섹션)도 반환하는데, 이는 "contracts"라는 단어가 컨텍스트 설명에도 등장하기 때문. 허용 가능.
- **Philosophy drift**: None — 순수 BMB 자체구현 진행.
- **Roadmap impact**: 7/13 도구 구현 완료. Python 서버의 핵심 도구는 모두 포함됨.

## Carry-Forward
- Actionable: 
  - `bmb_context_pack` 도구 추가 (context_pack 바이너리 호출)
  - `bmb_compile` 도구 추가 (native build)
  - `bmb_from_rust` 도구는 AI-based라 BMB 포트 범위 밖 (HUMAN 결정 필요)
  - MCP 서버 설정 파일 (`ecosystem/bmb-mcp/mcp_server_config.json`) 추가
- Structural Improvement Proposals: `run_bmb_on_source` 함수 시그니처 개선 — `extra` 파라미터 재도입으로 `-o outfile` 지원
- Pending Human Decisions: `bmb_from_rust` 도구 범위 결정 (AI 기반이므로 순수 BMB 포트 불가)
- Roadmap Revisions: None
- Next Recommendation: Cycle 3037 — MCP 서버 설정 파일 + cargo test 실행 + commit
