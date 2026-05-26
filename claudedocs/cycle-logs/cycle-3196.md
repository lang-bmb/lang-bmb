# Cycle 3196: TK_*() 체인 integer literal 치환으로 chained_comparison 186→91
Date: 2026-05-26

## Re-plan
Plan valid. Inherited scope: TK_*() 비교 체인을 integer literal 치환 후 match 변환.

## Scope & Implementation

**convert_chains_to_match.py 수정 (TK_* 지원)**:
- TK_* 함수 추출: `fn TK_XYZ() -> i64 ... = 2000000000 + N;` 패턴에서 106개 추출
- `parse_literal`: TK_XYZ() 호출을 정수값으로 치환 (예: `TK_FN()` → `2000000097`)
- `is_literal_rhs`: TK_*() 패턴도 literal로 분류
- `is_fn_call_rhs`: TK_*() 제외하도록 수정
- PRELUDE 상수 제거 (부정확했음): line-based search만 사용
- rhs 추출 regex 개선: `([^\s{(]+(?:\(\))?)` — TK_XYZ() 패턴 포착

**4회 스크립트 실행으로 27개 체인 변환**:
- Run 1: 23개 변환 (kind/k/nk/k1/k2 변수, TK_* 비교 체인)
- Run 2: 2개 추가 변환
- Run 3: 1개 추가 변환
- Run 4: 0개 (수렴)
- Total: 27 chains (chained_comparison 186 → 91, −95)

**fix_else_match.py 재실행**: 9개 `else match VAR { }` → `else { match VAR { } }` 수정

**나머지 91개 분석 (변환 불가 원인)**:
1. **복합 조건 or**: `k == TK_IDENT() or k == TK_I64() or ...` — parse_chain 올바르게 거부
2. **fn-call RHS**: `if is_int_literal(kind)` — 함수 호출 기반 분기, 패턴 아님
3. **mixed match+else-if**: 이미 match인데 외부 else-if 체인 혼합 (line 15261)
4. **no-else chains**: 기본 else 없는 체인 — SKIP (dangling code 방지)
5. **중복 TK_* 값**: TK_AS = TK_BREAK = 2000000127 — 충돌 위험

## Verification & Defect Resolution
- `bmb check`: ~1,318 warnings (chained_comparison: 91, non_snake_case: 108, semantic_duplication: 1,119)
- Stage 1 bootstrap: ✅ `{"type":"build_success","output":"bootstrap/compiler.exe"}`

## Reflection
- **Scope fit**: TK_*() integer 치환 성공. chained_comparison 186 → 91 (−95). 총 sycle 3192 기준 ~757 → 91 (−666)
- **잔여 91개 분석**: 모두 정당한 이유로 변환 불가 (복합 조건, 함수 분기, no-else, 중복값)
- **non_snake_case 108개**: 전부 TK_*() 함수 의도적 대문자 명명 — 언어 수준 억제 없이는 제거 불가
- **semantic_duplication 1119개**: 구조적 장기 작업 — M10 단기 범위 밖

## Carry-Forward
- Actionable: chained_comparison 91개 중 수동 변환 가능 케이스 조사 (compound or, mixed match+else-if)
- Actionable: non_snake_case 108개 — TK_* 처리 방향 Human Decision 필요
  - Option A: TK_FN → tk_fn (대규모 리네이밍, 가독성 변화)
  - Option B: BMB `@allow(non_snake_case)` 어노테이션 지원 추가
- Structural Improvement Proposals: None
- Pending Human Decisions: TK_* non_snake_case 처리 방향
- Roadmap Revisions: None
- Next Recommendation: Cycle 3197 — compound or 조건 수동 수정 + mixed match+else-if 케이스 처리로 추가 감축 시도
