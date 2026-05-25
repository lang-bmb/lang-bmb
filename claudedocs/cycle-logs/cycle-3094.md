# Cycle 3094: `bmb verify --list-uncontracted` CLI 추가
Date: 2026-05-25

## Re-plan
계획 유효. M7-4 첫 구성요소 착수: `bmb verify --list-uncontracted` CLI.

## Scope & Implementation

`Verify` 명령에 2개 flag 추가:
- `--list-uncontracted`: pre/post/contracts 없는 함수 JSON 목록 출력
- `--suggest`: counterexample → pre 힌트 (verify_suggest 구현)

**핵심 함수**:
- `list_uncontracted_fns`: AST 스캔, FnDef.pre/post/contracts 검사, 라인 번호 계산
- `verify_suggest`: Z3 failed 함수의 counterexample 파싱 → pre 힌트 출력
- 기존 `format_type` 함수 재사용 (중복 제거)

**출력 형식**:
```json
{"type":"uncontracted_functions","file":"...","count":1467,"functions":[{"name":"fn_name","line":N,"params":[...]},...]}
```

**검증 결과**:
- `bootstrap/compiler.bmb`: 총 1513 함수 중 1467개 미계약 (46개 이미 계약)
- `test_range_contracts.bmb`: count=0 (전부 계약 있음) ✅
- `contract_assume_opt.bmb`: count=1 (main만 미계약) ✅

## Verification & Defect Resolution

- cargo build --release: ✅
- bmb verify --list-uncontracted: 정상 작동 ✅
- cargo test --release: 실행 중

## Reflection

- Scope fit: 100%
- 기존 format_type 중복 버그 즉시 수정 (E0428)
- verify_suggest 스켈레톤 구현 완료 (counterexample 파싱)
- 1467개 미계약 함수 식별 — Track B 자동화의 기반 마련

## Carry-Forward

- Actionable: Cycle 3095 — `suggest_contracts` MCP tool 추가 (mcp_server.bmb)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M7-4 Phase 1 진척
- Next Recommendation: suggest_contracts tool 구현 (heuristic 기반)
