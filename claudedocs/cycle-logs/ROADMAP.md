# Cycle-Logs 방향성 로드맵
> 최종 업데이트: 2026-05-25 (Cycle 3094 — M7-4 착수)
> 이 파일은 **방향성 앵커**다 — 각 사이클 Derive-Next에서 수정 가능.
> 실무 앵커: `claudedocs/ROADMAP.md`

## 현재 상태 (Cycle 3093 기준)

- HEAD: `66c460dc`
- M7-3: ✅ COMPLETE (forall/exists E2E + Track B 20종+)
- M7-4: ⏳ 사양 정의 완료, 미착수 (P3)
- cargo test --release: ALL PASS ✅
- 3-Stage Fixed Point: `ea550bf3` ✅
- bmb verify bootstrap/compiler.bmb: 1513/1513 ✅

## Cycles 3094-3103 방향성 — M7-4 구현

### M7-4 구성요소 (3가지)

1. `bmb verify --list-uncontracted` CLI
2. `suggest_contracts` MCP tool (heuristic 기반)
3. `bmb verify --suggest` (counterexample → pre 힌트)
4. Track B 자동화 스크립트 (BMB 코드)

### 우선순위 (계층 순)

**Phase 1 (Cycles 3094-3095)**: `bmb verify --list-uncontracted` CLI
- Rust CLI에 flag 추가
- AST 스캔: pre/post/contracts 없는 함수 JSON 목록 출력
- Track B 자동화의 기반

**Phase 2 (Cycles 3096-3097)**: `suggest_contracts` MCP tool
- mcp_server.bmb에 9번째 tool 추가
- heuristic: pos/idx/n 파라미터 → `pre param >= 0` 제안
- 함수 이름/반환 타입 패턴 기반 post 제안

**Phase 3 (Cycles 3098-3099)**: `bmb verify --suggest`
- Failed 함수의 counterexample 파싱
- 음수값 → `pre param >= 0` 힌트 출력
- JSON 출력 확장

**Phase 4 (Cycles 3100-3101)**: Track B 자동화 스크립트
- `bootstrap/list-uncontracted.bmb`
- 미계약 함수 목록 → 우선순위 출력

**Phase 5 (Cycles 3102-3103)**: 추가 Track B + M7-4 COMPLETE
- 미계약 함수 중 주요 함수 계약 추가
- ROADMAP M7-4 ✅ 마킹

### 조기 종료 조건

- M7-4 3개 구성요소 완성 → 조기 종료 가능
- HARD STOP 없이 자연 종료

## 알려진 HUMAN-blocked 항목

- B-axis 재측정 (API key 필요)
- M8 계획 수립 (외부 신호 대기)
- Active ISSUE 5개 (모두 HUMAN-blocked)
