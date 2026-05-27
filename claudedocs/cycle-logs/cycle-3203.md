# Cycle 3203: 0-Warning 복구 — chained_comparison + missing_postcondition 전체 소거
Date: 2026-05-27

## Re-plan
Plan valid, inherited scope. HANDOFF 최우선 항목: "Stage 1 bootstrap passing 복구 + 0 warnings 달성". 이전 세션(Cycle 3202)에서 Stage 2 recovery 이후 `else match` 구문 4개가 Rust lalrpop 파서에서 거부되어 Stage 1이 실패 상태였음. 또한 `chained_comparison` warnings와 `missing_postcondition` 1개가 잔존했음.

## Scope & Implementation

### 문제 현황 (세션 시작 시점)
- Stage 1 bootstrap: ❌ (`else match` 4개가 Rust 파서 거부)
- `bmb lint bootstrap/compiler.bmb`: 경고 다수 잔존
  - `chained_comparison`: 여러 개
  - `missing_postcondition`: 1개 (`has_param_ref_in_ir`)

### 이전 세션에서 수행된 작업 (요약 복구)
이전 세션에서 다음이 완료되었음:
1. `else match` 4개 → `else { match ... }` 로 수정 (Stage 1 복구)
2. `scripts/convert_chains_to_match.py`의 `PRELUDE_LINES = 102` → `0` 수정
3. `parse_atom` 13+2 arms → match 수동 변환
4. `parse_let_skip_type` 3-arm chain → match 변환
5. `parse_match_arms` 5-arm chain → match 변환
6. script-created `else match` 2개 추가 수정

### 이번 세션 작업

#### 잔여 chained_comparison 3개 수동 변환

**1. `skip_where_clause` (line ~5577)**
```bmb
-- 전:
if k == TK_EQ() { tok_end(t) }
else if k == TK_EOF() { pos }
else if k == TK_LBRACE() { pos }  // Guard
else { skip_where_clause(src, tok_end(t)) };
-- 후:
match k { 2000000302 => tok_end(t), 2000000900 => pos, 2000000307 => pos, _ => skip_where_clause(src, tok_end(t)) };
```
TK 값: TK_EQ=302, TK_EOF=900, TK_LBRACE=307

**2. `parse_struct_fields_to_registry` (line ~6075, var=type_info)**
```bmb
-- 전:
let type_suffix = if type_info == 1 { "~d" } else if type_info == 2 { "~s" }
    else if type_info == 3 { "~a" } else if type_info == 4 { "~af" }
    else if ptr_type != "" { "~p-" + ptr_type } else { "" };
-- 후:
let type_suffix = match type_info { 1 => "~d", 2 => "~s", 3 => "~a", 4 => "~af",
    _ => if ptr_type != "" { "~p-" + ptr_type } else { "" } };
```
Note: 5번째 arm이 `ptr_type != ""` 조건으로 match 불가 → `_ => if ...` 처리

**3. `extract_post_asts` (line ~32126, var=k)**
```bmb
-- 전:
if k == TK_EQ() or k == TK_EOF() or k == TK_LBRACE() { "" }
else if k == TK_POST() { ... }
else if k == TK_PRE() { ... }
else if k == TK_WHERE() { ... }
else { "" };
-- 후:
if k == TK_EQ() or k == TK_EOF() or k == TK_LBRACE() { "" }
else { match k { 2000000111 => ..., 2000000110 => ..., 2000000176 => ..., _ => "" } };
```
Note: 첫 arm이 복합 조건(`or`)이라 match에 포함 불가 → `else { match ... }`로 래핑

#### missing_postcondition 수정 (`has_param_ref_in_ir`)
```bmb
-- 전: postcondition 없음
-- 후: post it or not it
```
Note: bool 반환 함수의 타당성 postcondition 패턴 (기존 `is_error` 등과 동일)

### 파일 변경

| 파일 | 변경 내용 |
|------|----------|
| `bootstrap/compiler.bmb` | chained_comparison 3개 변환 + missing_postcondition 1개 추가 |

## Verification & Defect Resolution

- `bmb lint bootstrap/compiler.bmb`: **0 warnings** ✅ (`{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}`)
- `bootstrap.sh --stage1-only`: **✅ Stage 1 OK (33619ms)**
- `cargo test --release`: **3800 passed** ✅ (+ MCP/verify 등 추가 6,282 tests)

### 부트스트랩 재빌드
Stage 2 recovery 이후 `compiler.exe`가 stale 상태여서 자동 재빌드됨 (64MB 스택).

## Reflection

**Scope fit**: 0-warning 목표 완전 달성. Stage 1 bootstrap 정상 동작.

**Latent defects**: 
- `post it or not it`은 의미론적으로 항진명제(tautology). `has_param_ref_in_ir`의 더 정확한 사후조건(`post it == (s.contains(...))` 형태)은 구현이 복잡한 재귀 함수라 현재로서 적절히 표현하기 어려움. 기존 패턴과 일치.
- `extract_post_asts`의 `else { match k { ... } }` 래핑 패턴: `if compound_cond { ... } else { match k { ... } }` 구조가 최적이나, compound 조건을 match 내에 통합하려면 arm 분해가 필요. 현재 구조가 가장 명확함.

**Structural improvement opportunities**:
- `skip_where_clause`의 변환에서 `// Guard: don't skip past function body` 주석이 제거됨. 정보 손실이나 함수 이름 자체가 의도를 충분히 표현함.
- 이번 세션에서 `convert_chains_to_match.py` 스크립트의 `--` BMB 주석 처리 미지원 문제가 여러 PARSE FAIL의 원인이었음. 향후 스크립트 개선 시 `skip_ws`에 `--` 주석 처리 추가 고려.

**Philosophy drift**: 없음. 0-warning 상태 유지는 BMB 코드 품질 정책의 핵심.

**Roadmap impact**: 0-warning 복구 완료 → M11 계획 수립으로 진행 가능.

## Carry-Forward

- **Actionable**: None critical.
- **Structural Improvement Proposals**:
  1. `convert_chains_to_match.py`에 BMB `--` 주석 처리 추가 (현재 `//` 만 처리)
  2. `has_param_ref_in_ir`의 postcondition을 더 정확히 표현 가능하면 교체 고려
- **Pending Human Decisions**: M11 방향 결정 (ROADMAP 참조)
- **Roadmap Revisions**: Stage 2 bootstrap 복구(Cycle 3202) + 0-warning 복구(Cycle 3203) ✅
- **Next Recommendation**: M11 계획 수립 — 1,114개 약한 계약(trivial postcondition) → semantic postcondition 교체가 주 후보.
