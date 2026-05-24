# Cycle 3081: String SMT 확장 — contains/starts_with/ends_with
Date: 2026-05-25

## Re-plan
Cycle 3080 Carry-Forward: String SMT 확장 (contains/starts_with/ends_with) 착수.
Cycle 3079에서 `str.len()` 지원 추가 — 동일 패턴으로 3종 확장 가능.

## Scope & Implementation

### 변경 파일: `bmb/src/smt/translator.rs`

**MethodCall 핸들러 확장** (기존 `len`에서 3종 추가):

```
s.contains(t)    → (str.contains s t)
s.starts_with(t) → (str.prefixof t s)   // SMT-LIB2: prefix first, string second
s.ends_with(t)   → (str.suffixof t s)   // SMT-LIB2: suffix first, string second
```

**코드 변경**:
- String 변수(`SmtSort::Str`)에 대한 1-arg method call 처리 추가
- `arg = translate_expr(args[0])` → `contains`/`starts_with`/`ends_with` match 분기

**신규 테스트 3개**:
- `test_string_contains_literal`: `s.contains("bmb_")` → `(str.contains s "bmb_")`
- `test_string_starts_with_literal`: `s.starts_with("bmb_")` → `(str.prefixof "bmb_" s)`
- `test_string_ends_with_literal`: `s.ends_with("_fn")` → `(str.suffixof "_fn" s)`

## Verification & Defect Resolution

- `cargo test --release`: **6274 PASS** ✅ (6271 → 6274, +3 신규)
- `bmb verify bootstrap/compiler.bmb`: **1513/1513** ✅ (회귀 없음)

### 설계 제약 발견

`post it.starts_with("bmb_")` 형태의 post-condition은 현재 미지원:
- `Expr::It` → SMT `"__it__"` 로 번역되지만
- `verify_post`는 `__it__`를 선언하지 않음 (refinement type 경로에만 선언)
- 따라서 Track B 함수에 post-condition 추가는 별도 수정 필요 (다음 사이클)

## Reflection

- **Scope fit**: 100%
- **Philosophy drift**: 없음
- **Roadmap impact**: SMT String theory가 더 강한 Track B pre-condition 작성 가능

## Carry-Forward

- **Actionable**: 없음
- **Structural Improvement Proposals**:
  1. `verify_post`에서 `__it__` 선언 추가 — `post it.starts_with(...)` 지원 가능
  2. Track B 함수 post-condition 강화 (`post it.starts_with("bmb_")` 등)
  3. M7-3 착수 — 복합 계약 문법 (let-in-pre, quantifiers)
- **Pending Human Decisions**: M8 계획 수립 (외부 신호 기반)
- **Roadmap Revisions**: 없음
- **Next Recommendation**: `verify_post` __it__ 선언 수정 → Track B post-condition 강화
