# Cycle 3090: Range 계약 골든 테스트 + Track B 확대 (4종)
Date: 2026-05-25

## Re-plan
Cycle 3089 Carry-Forward: 순수 산술 함수 계약 + M7-4 초기 구상.
범위/단조 계약 골든 테스트 + compiler.bmb 4종 계약 추가.

## Scope & Implementation

### 신규 골든 테스트: `tests/golden/test_range_contracts.bmb`
4종 범위·단조 계약 패턴:
- `min3`: `post it <= a and it <= b and it <= c` — Z3 LIA if-else 직접 증명 ✅
- `max3`: `post it >= a and it >= b and it >= c` — 대칭 ✅
- `sum_nonneg`: `pre a >= 0 and b >= 0 / post it >= a and it >= b` — 덧셈 단조 ✅
- `double_grows`: `pre a >= 0 / post it >= a` — `a * 2 >= a` LIA ✅

**비선형 제거**: 초안의 `product_nonneg(a * b)` → `a * b`는 variable × variable = nonlinear
→ Z3 unknown (total count 감소) → `double_grows(a * 2)`로 교체 (scalar mult = linear)

→ 4/4 verified ✅

### Track B 계약 추가 (4종)

**`make_caret_line(n, acc)`** (line 74):
- `pre n >= 0`
- n은 caret 앞 공백 수 — 음수 불가

**`get_char_value(s, pos)`** (line 514):
- `pre pos >= 0`
- 소스 위치 비음수

**`unpack_pos_acc(r, pos, acc)`** (line 745):
- `pre pos >= 0 and acc >= 0`
- 파싱 누산기는 비음수

**`find_colon(s, pos)`** (line 755):
- `pre pos >= 0`
- 탐색 시작 위치 비음수

## Verification & Defect Resolution

- `bmb verify tests/golden/test_range_contracts.bmb`: **4/4** ✅
- `bmb verify bootstrap/compiler.bmb`: **1513/1513** ✅
- `cargo test --release`: **ALL PASS** ✅

## Reflection

- **Scope fit**: 100%
- **Key finding**: `min3`/`max3`의 `post it <= all` 계약은 if-else 체인에서 Z3가 각 브랜치를 독립 검증 — 복잡한 귀납 없이 증명 가능
- **Philosophy drift**: 없음
- **LIA boundary**: `a * b` (변수 간 곱) = nonlinear → Z3 unknown. `a * 2` (상수 곱) = linear → OK

## Carry-Forward

- **Actionable**: Cycle 3091에서 더 많은 Track B + M7-4 사양 정의
- **Structural Improvement Proposals**: 
  - callee contract를 axiom으로 사용하는 verifier 개선 → 함수 호출 body post 검증 가능 (현재 unknown)
  - 이 개선 시 `tok_kind`/`scan_*` 등 post 계약 추가 가능
- **Pending Human Decisions**: None
- **Next Recommendation**: Cycle 3091 — ROADMAP에 M7-4 사양 추가 + Track B 계속
