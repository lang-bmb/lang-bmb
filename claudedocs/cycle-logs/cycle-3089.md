# Cycle 3089: Track B 계약 확대 — tok_kind/skip_sp_tab + 발견사항
Date: 2026-05-25

## Re-plan
Cycle 3088 Carry-Forward: tok_kind + skip_sp_tab + 추가 scan 함수 계약.

## Scope & Implementation

### 추가된 계약 (2종)

**`tok_kind(r: i64) -> i64`** (line 541):
- `pre r >= 0`
- 주의: `post it >= 0` 추가 시 total 1513→1512 (unknown 제외됨) → post 제거, pre만 유지
- 원인: body = `tok_val(r)` (함수 호출). 반환값이 uninterpreted function → Z3 unknown

**`skip_sp_tab(s, pos, limit) -> i64`** (line 110):
- `pre pos >= 0 and limit >= 0`
- 재귀 함수이므로 post 없이 pre만 (호출 불변식 문서화)

### 핵심 발견사항: 함수 호출 body + post = Z3 unknown

**현상**: `fn tok_kind(r: i64) -> i64 / pre r >= 0 / post it >= 0 / = tok_val(r)` 추가 시:
- `"total":1512,"verified":1512,"failed":0` (1513→1512)
- total이 1 감소: unknown 결과는 failed가 아닌 "제외"됨

**원인**: 
```
tok_kind = tok_val(r)  →  Z3에서 tok_val은 uninterpreted function
Z3가 tok_val(r) >= 0임을 모름 (body를 직접 보지 않으면)
→ Z3 unknown (proved도 failed도 아님)
→ verifier가 total에서 제외
```

**교훈**: BMB verifier는 함수 호출의 반환값을 SMT axiom으로 처리하지 않음.
- `= arithmetic_expr`: 직접 SMT 번역 가능 → post 증명 가능
- `= fn_call(args)`: uninterpreted → post 불확실 (pre만 추가)

이는 Track B 계약 전략에 중요한 지침:
- **Non-recursive arithmetic bodies**: pre + post 모두 가능
- **Recursive bodies**: pre만 (귀납 불가)  
- **Function-call bodies**: pre만 (body uninterpreted)

## Verification & Defect Resolution

- `bmb verify bootstrap/compiler.bmb`: **1513/1513** ✅
- `cargo test --release`: **ALL PASS** ✅

## Reflection

- **Scope fit**: 100% — 의도한 2종 계약 추가 완료
- **Key finding**: 함수 호출 body의 post 검증이 unknown 처리됨 (total count 감소로 발견) — Track B 전략에 중요한 제약
- **Philosophy drift**: 없음
- **Roadmap impact**: Track B 계약 전략 명확화 — pre/post 추가 가능 범위 정의

## Carry-Forward

- **Actionable**: Cycle 3090에서 더 많은 순수 산술 함수 (non-recursive + arithmetic body) 찾아 pre+post 추가
- **Structural Improvement Proposals**: 함수 호출 body의 post 검증 개선 (verifier가 callee contract를 axiom으로 사용하면 가능)
- **Pending Human Decisions**: None
- **Next Recommendation**: Cycle 3090 — 추가 Track B (순수 산술) + M7-4 초기 구상
