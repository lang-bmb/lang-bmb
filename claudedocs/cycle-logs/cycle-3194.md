# Cycle 3194: CRLF 복원 + find_chain_start_byte 개선 + 19개 추가 변환
Date: 2026-05-26

## Re-plan
Plan valid. Inherited scope: find_chain_start_byte ±50 범위 확장으로 SKIP 해소 후 추가 변환.

## Scope & Implementation

**find_chain_start_byte 개선**:
- lookback 40 bytes: `content[max(0, raw-40):raw+50]` 범위에서 `if\s+var\s*==` 검색
- ±50 line 탐색: 기존 ±8 → ±50으로 확장

**세그먼트 크기 확장**: 12000 → 200000 chars (ntype arms=0 PARSE FAIL 해소)

**convert_chains_to_match.py 쓰기 모드 binary 전환**:
- `open('w', encoding='utf-8')` → `open('wb')` + `.encode('utf-8')`
- Windows CRLF 자동 변환 방지

**19개 추가 literal chain 변환** (219 → 186 chained_comparison):
- cmp_op, next, method, ntype, b1 등 포함
- 9개 `else match VAR { }` → `else { match VAR { } }` 수정 (fix_else_match.py 2회 실행)

**CRLF corruption 발견 및 해소**:
- 원인: `fix_else_match.py` text mode read/write가 `\r\r\n` → `\n\n` (read) → `\r\n\r\n` (write) 변환
- 결과: 2회 반복 후 파일이 137K → 550K 줄로 팽창 (2717053 → 2166109 bytes)
- 해결 1: `\r+\n` → `\r\n` 정규화 (convert script에서 binary write)
- 해결 2: `(\r\n){3,}` → `\r\n\r\n` 빈 줄 축소 (2166109 → 1150053 bytes, 550944 → 42916 줄)

## Verification & Defect Resolution
- `bmb check`: 1,488 warnings (chained_comparison: 186, semantic_duplication: 1,119, non_snake_case: 108, unused_binding: 64, single_arm_match: 11)
- Stage 1 bootstrap: ✅ `{"type":"build_success","output":"bootstrap/compiler.exe"}`

## Reflection
- **Scope fit**: find_chain_start_byte 개선 + 19개 추가 변환. chained_comparison 219 → 186 (−33)
- **Cumulative**: 사이클 3192 시작 기준 ~757 → 186 (−571)
- **TK_* 분류**: 잔여 186개 중 ~152개는 `kind`/`k`/`nk`/`k1` 변수의 TK_*() 함수 호출 체인 (e.g. `if kind == TK_FN()`) — literal이 아니므로 match 변환 불가
- **fn_name 복합 조건**: `if fn_name == "@bmb_to_binary" or fn_name == "@bmb_to_octal" ...` 형태 — parse_chain이 올바르게 거부
- **fix_else_match.py**: text write mode 버그 미수정 (이번 사이클에서 CRLF 문제의 근원)

## Carry-Forward
- Actionable: fix_else_match.py binary write mode 전환 (CRLF 재발 방지)
- Actionable: TK_* 체인 처리 방향 결정 — 두 가지 옵션:
  1. TK_*() → 정수 리터럴 치환 후 match 변환 (가독성 손실, TK_AS=TK_BREAK=2000000127 등 중복값 문제)
  2. BMB 언어에 경고 억제 메커니즘 추가 (`// bmb:no-warn(chained_comparison)`)
- Structural Improvement Proposals: 잔여 non_snake_case 108개(TK_* 의도적 명명), unused_binding 64개, single_arm_match 11개도 처리 필요
- Pending Human Decisions: TK_* 체인 접근법 선택 (언어 수준 결정)
- Roadmap Revisions: None
- Next Recommendation: Cycle 3195 — fix_else_match.py 수정 + TK_* 체인 처리 방향 결정 후 non_snake_case/unused_binding 경고 처리
