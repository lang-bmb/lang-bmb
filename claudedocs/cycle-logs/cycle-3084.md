# Cycle 3084: forall/exists Quantifier E2E — 갭 발견 및 수정
Date: 2026-05-25

## Re-plan
M7-3 Quantifier E2E 착수. 사용자가 선택한 방향: "forall/exists E2E 테스트 + 타입체커 검증 → Track B 함수에 forall 계약 적용".
인프라(파서·AST·SMT 번역기) 존재 확인됨 — 실제 갭 식별이 목표.

## Scope & Implementation

### 갭 1: `Expr::Forall`/`Expr::Exists` 번역 시 bound variable 미등록
**파일**: `bmb/src/smt/translator.rs` (line 480–490)

Before: bound variable을 `var_types`에 등록하지 않고 body 번역 → "undefined variable: n" 에러

After: body 번역 전 `var_types`에 삽입, 번역 후 복원 (scoped pattern).
이를 위해 `translate`/`translate_expr`를 `&self` → `&mut self`로 변경 필요.

### API 변경: `translate` → `&mut self`
- `SmtTranslator::translate(&self)` → `translate(&mut self)`
- `SmtTranslator::translate_expr(&self)` → `translate_expr(&mut self)`
- `contract.rs` 5개 verify 함수: `&SmtTranslator` → `&mut SmtTranslator`
- 테스트 코드 50군데 `let trans =` → `let mut trans =`

### 갭 2: quantifier 사용 시 Z3 logic 미전환
**파일**: `bmb/src/smt/translator.rs` (SmtLibGenerator::generate)

Before: 항상 `QF_LIA` (Quantifier-Free) → "(error "logic does not support quantifiers")"

After: `SmtTranslator::has_quantifiers` 플래그 추가. Forall/Exists 번역 시 `has_quantifiers = true` 설정. contract.rs 6개 함수에서 번역 후 `generator.set_has_quantifiers()` 전파.
`generate()` 로직: `has_strings` → ALL, `has_quantifiers` → LIA, 기본 → QF_LIA.

### 신규 골든 테스트
`tests/golden/test_forall_basic.bmb`:
- `double_positive`: simple pre/post (기존 패턴)
- `identity_with_forall`: `pre x > 0 and (forall n: i64, n > 0 implies n > 0)` — forall in pre
- `always_forty_two`: `post exists n: i64, n == it` — exists in post
→ 3/3 verified ✅

### 문법 발견사항
- `pre`/`post`는 각 함수당 1개만 허용 (파서 제약)
- `and forall` 불가 — `forall`은 `Expr` 레벨, `and`의 오른쪽은 `CmpExpr` 레벨
- `and (forall ...)` 형태로 괄호 사용 가능 (parenthesized expression은 PrimaryExpr)

## Verification & Defect Resolution

- `bmb verify tests/golden/test_forall_basic.bmb --human`: **3/3 verified** ✅
- 실패 케이스 (zero_but_claims_positive) counterexample 정상 반환 ✅
- `cargo test --release`: **6282+ PASS** (test_create_project 기존 플리키 제외) ✅

## Reflection

- **Scope fit**: 100% — 두 실제 버그 + API 정합성 수정으로 E2E 완결
- **Philosophy drift**: 없음
- **Roadmap impact**: M7-3 Quantifier E2E 1단계 완결. 다음은 Track B 계약 적용.
- **Discovered**: `forall`을 standalone으로 `pre`에 사용하려면 괄호 필요 — 언어 스펙 제약으로 허용 가능하지만 문법 확장 필요 시 논의 가능.

## Carry-Forward

- **Actionable**: None (Cycle 3085에서 Track B forall 계약 적용)
- **Structural Improvement Proposals**:
  1. `pre forall n: T, body` 형식 직접 지원 고려 (현재 `pre (forall ...)` 필요) — M7-4 이후
  2. `SmtTranslator::has_quantifiers`를 `reset()` 시 초기화 필요 여부 검토 (현재 누적됨)
- **Pending Human Decisions**: None
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 3085 — Track B compiler.bmb 함수에 forall/exists 계약 적용 첫 시도 + 더 복잡한 quantifier 패턴 테스트
