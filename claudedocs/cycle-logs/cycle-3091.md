# Cycle 3091: M7-4 사양 정의 + Track B (find_separator/low_skip_ws/low_find_ident_end)
Date: 2026-05-25

## Re-plan
Cycle 3090 Carry-Forward: ROADMAP에 M7-4 사양 추가 + Track B 계속.

## Scope & Implementation

### M7-4 사양 정의 (ROADMAP.md 추가)

**M7-4: 자동 contract 생성 AI 파이프라인** — 3개 구성요소 정의:
1. MCP tool `suggest_contracts`: fn_source → 계약 제안 목록 (confidence 포함)
2. `bmb verify --suggest`: Z3 counterexample → pre 힌트 역방향 생성
3. Track B 자동화 스크립트: 미계약 함수 리스트 → AI 제안 → 검증 루프

ROADMAP `§ M7` 섹션 `완료 기준` 이후에 "M7-4 사양" 서브섹션 추가.

### Track B 계약 추가 (3종)

**`find_separator(s, pos)`** (line 8007):
- `pre pos >= 0`
- "###" 구분자 탐색 — 재귀 시작 위치 비음수

**`low_skip_ws(s, pos)`** (line 3373):
- `pre pos >= 0`
- 저수준 공백 스킵 — 재귀 시작 위치 비음수

**`low_find_ident_end(s, pos)`** (line 3384):
- `pre pos >= 0`
- 저수준 식별자 끝 탐색 — 재귀 시작 위치 비음수

## Verification & Defect Resolution

- `bmb verify bootstrap/compiler.bmb`: **1513/1513** ✅
- `cargo test --release`: **ALL PASS** ✅

## Reflection

- **Scope fit**: 100%
- **M7-4 사양**: 충분히 구체적으로 정의됨. 착수 가능한 상태.
- **Track B 진행**: 공통 패턴 (`pos >= 0`) 스캔 함수군 전반으로 계속 확산
- **Philosophy drift**: 없음
- **Roadmap impact**: M7-4 미착수, 사양은 ROADMAP에 영속화됨

## Carry-Forward

- **Actionable**: Cycle 3092에서 추가 Track B OR M7-4 초기 구현 착수 (MCP tool 우선)
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: M7-4 착수 승인 (P3 — 자율 가능)
- **Next Recommendation**: Cycle 3092 — Track B 계약 총 집계 + 커밋 준비 (10사이클 완료)
