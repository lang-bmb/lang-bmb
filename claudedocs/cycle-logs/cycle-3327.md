# Cycle 3327: MCP bmb_diagnose 스키마 업데이트 (P3)
Date: 2026-05-30

## Re-plan
이전 Carry-Forward: MCP bmb_diagnose 스키마가 구 형식 기준. 신 5섹션 형식으로 업데이트 필요.

## Scope & Implementation
- `ecosystem/bmb-mcp/mcp_server.bmb` — `bmb_diagnose` 설명 문자열 업데이트:
  - Old: "Unified effect diagnostics: effect-verify + contracts-check + lint-effects in one JSON response"
  - New: "Unified AI-native diagnostics: 5 sections (effect_verify, contracts_check, module_capability, lint_effects, semantic_duplicate) + summary in one JSON. Violations use unified {\"type\":\"rule_name\",\"function\":\"...\", ...} format."
- MCP 서버는 .bmb 소스만 존재 (바이너리 없음), 다음 빌드 시 반영

## Verification & Defect Resolution
- 문자열 변경 확인 ✅
- MCP 서버 재빌드 불필요 (소스만 수정, 실행 시 컴파일됨)

## Reflection
- 순수 문서 업데이트. P3 완료.
- bmb_diagnose 도구가 이제 5섹션 구조와 violations 통일 형식을 명확히 설명함.
- 로드맵 영향: P3 MCP 업데이트 완료. 다음은 bootstrap P-track 회귀 분석 (P2).

## Carry-Forward
- Actionable: bootstrap P-track 회귀 분석 — csv 1.134×, lexer 1.459× (P2)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3328 — bootstrap P-track IR 비교 분석 시작
