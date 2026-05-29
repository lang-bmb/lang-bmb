# Cycle 3303: bmb-mcp bmb_diagnose 도구 추가
Date: 2026-05-29

## Re-plan
남은 3사이클 — bmb-mcp에 diagnose 도구 추가.

## Scope & Implementation
**ecosystem/bmb-mcp/mcp_server.bmb**:
- `tool_bmb_diagnose` 함수 추가 — `run_bmb_on_source(source, "diagnose")` 패턴
- tools list에 `bmb_diagnose` 도구 등록 (설명: "Unified effect diagnostics: effect-verify + contracts-check + lint-effects")
- `handle_tools_call`에 dispatch 추가

## Verification & Defect Resolution
- `bmb check mcp_server.bmb` → success (기존 경고만) ✅
- cargo test: 3800+2390+23 PASS ✅

## Reflection
- `bmb_diagnose` MCP tool이 있으면 AI 에이전트가 단일 MCP 호출로 전체 effect 진단 수행 가능
- M13 AI Workflow First-class 비전과 직접 정렬 (AI-consumable structured output)
- 로드맵 영향: 없음 (기능 완성도 향상)

## Carry-Forward
- Actionable: 커밋 + HANDOFF + 메모리 갱신
- Structural Improvement Proposals: diagnose에 count 필드 추가 (warn 개수)
- Pending Human Decisions: None
- Roadmap Revisions: ROADMAP § 6 업데이트 필요
- Next Recommendation: Cycle 3304 — 커밋 + HANDOFF 갱신 + 세션 종료
