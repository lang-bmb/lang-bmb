# Cycle 3195: unused_binding 64개 + single_arm_match 11개 제거
Date: 2026-05-26

## Re-plan
Plan valid. Inherited scope: fix_else_match.py binary write 수정 + TK_* 체인 처리 결정. Scope expand: unused_binding + single_arm_match도 처리.

## Scope & Implementation

**fix_else_match.py binary write 수정**: `open('r')` → `open('rb')` + decode, `open('w')` → `open('wb')` + encode — CRLF 재발 방지

**unused_binding 64개 제거**:
- 스크립트 기반 자동 수정: 경고 line → actual line 매핑 (warn_line - PRELUDE_LINES - 1)
- 함수 파라미터: `varname: Type` → `_varname: Type` (57개)
- let 바인딩: `let varname =` → `let _varname =` (7개)
- 대상: `cur_exit_label`, `sb`, `item`, `loop_exit`, `ast`, `line`, `fn_name`, `p1`, `tail_block`, `fn_mir`, `name`, `left`
- Stage 1 ✅ — 경고 64→0

**single_arm_match 11개 제거**:
- 4개 단순 변환: `match V { LIT => A, _ => B }` → `if V == LIT { A } else { B }`
  - `ntype` (7565), `fn_name` (14443) — 독립 단일 if, 체인 미발생
- 7개 전체 if-chain → match 변환:
  - `ntype` (7561-7565): 3-arm `match ntype { "float" => ..., "unary" => ..., "binop" => ..., _ => false }`
  - `op` (14069): `match op { "add" => "add nsw", "sub" => "sub nsw", "mul" => "mul nsw", _ => op }`
  - `layer` (38115-38119): `match layer { 0 => "leaf", 1 => "near-leaf", 2 => "mid", _ => "entry" }`
  - `typ` (15779-15807): 4-arm match including multi-statement let-binding arms (합산 1줄)
  - `ret_type` (15951-15967): 4-arm match
- nested match 평탄화 3개 (12451, 14483, 28791): `_ => match V { A, _ => match V { B, _ => C } }` → `A, B, C` 단일 level

**반복 수정 사이클**:
- match→if 변환이 기존 if-chain에 합류해 새 chained_comparison 발생 (186→191)
- 해결: 외부 체인 전체를 match로 변환 (186 복원)
- `ntype` (7561)도 동일 문제 → 전체 chain을 match로 변환

## Verification & Defect Resolution
- `bmb check`: 1,413 warnings (chained_comparison: 186, semantic_duplication: 1,119, non_snake_case: 108)
- `unused_binding: 0` ✅, `single_arm_match: 0` ✅
- Stage 1 bootstrap: ✅ `{"type":"build_success","output":"bootstrap/compiler.exe"}`

## Reflection
- **Scope fit**: unused_binding 64→0 + single_arm_match 11→0. 경고 1488 → 1413 (−75)
- **Cumulative**: chained_comparison 186, non_snake_case 108, semantic_duplication 1119 (장기)
- **TK_* non_snake_case**: 108개 모두 TK_*() 함수 의도적 대문자 명명. 언어 수준 억제 메커니즘 없이는 해소 불가
- **chained_comparison 186**: 대부분 TK_*() 비교 체인 (`kind == TK_FN()` 등) — match 변환 불가

## Carry-Forward
- Actionable: TK_* non_snake_case 처리 방향 결정 — 두 가지 옵션:
  1. `TK_FN` → `tk_fn` snake_case 전환 (대규모 리네이밍, 가독성 변화)
  2. BMB 언어에 경고 억제 메커니즘 추가 (`@allow(non_snake_case)` 어노테이션)
- Actionable: chained_comparison 186개 — TK_*() 체인은 integer literal 치환 없이는 match 변환 불가. Integer 치환 스크립트 고려.
- Structural Improvement Proposals: None
- Pending Human Decisions: TK_* 처리 방향
- Roadmap Revisions: None
- Next Recommendation: Cycle 3196 — TK_* 체인 integer literal 치환으로 chained_comparison 186개 중 최대한 제거 시도
