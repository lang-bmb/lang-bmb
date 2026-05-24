# Cycle 3082: `post it.method()` 인프라 완성
Date: 2026-05-25

## Re-plan
Cycle 3081 Carry-Forward: `verify_post`에 `__it__` 선언 추가 → `post it.starts_with(...)` 지원.
`Expr::It` 수신자 MethodCall 번역 + `setup_function` `__it__` 등록 필요.

## Scope & Implementation

### 변경 파일: `bmb/src/smt/translator.rs`

1. **`setup_function`에 `__it__` 등록**:
   - `self.var_types.insert("__it__", ret_sort)` 추가
   - 생성기 선언은 각 검증 경로(verify_post/verify_return_refinement)에 위임 — 중복 선언 방지

2. **MethodCall 핸들러 확장** — `Expr::It` 수신자 지원:
   ```
   Expr::Var(name) → name
   Expr::It        → "__it__"
   ```
   → 통합 처리로 코드 중복 제거

3. **신규 테스트 2개**:
   - `test_it_starts_with_translates_via_it_receiver`: `it.starts_with("bmb_")` → `(str.prefixof "bmb_" __it__)`
   - `test_setup_function_registers_it_type`: String 반환 함수 setup 후 `__it__` = SmtSort::Str

### 변경 파일: `bmb/src/verify/contract.rs`

1. **`verify_post`**: `__it__` 생성기 선언 + `(= __it__ __ret__)` assertion 추가
2. **`verify_named_contract`**: 동일 패턴 적용 (where 절 `it` 지원)

## Verification & Defect Resolution

- `cargo test --release`: **6276 PASS** ✅ (6274 → 6276, +2 신규)
- `bmb verify bootstrap/compiler.bmb`: **1513/1513** ✅ (회귀 없음)

### 한계: Track B 함수 post-condition Z3 검증

`method_to_runtime_fn`, `get_call_return_type`, `is_string_returning_fn`은 body가 수십~수백개 if-else 분기 → Z3 String theory 시간초과 예상.
인프라는 완성됐으나 실제 계약 추가는 단순한 함수에서만 현실적.

## Reflection

- **Scope fit**: 100%
- **Philosophy drift**: 없음
- **Roadmap impact**: `post it.method()` 완전 지원 → 향후 String 반환 함수에 post-condition 작성 가능

## Carry-Forward

- **Actionable**: 없음
- **Structural Improvement Proposals**:
  1. 단순 String 반환 함수에 `post it.starts_with(...)` 계약 추가 시도
  2. M7-3 착수 — 복합 계약 문법 (let-in-pre, quantifiers)
- **Pending Human Decisions**: M8 계획 수립 (외부 신호 기반)
- **Roadmap Revisions**: 없음
- **Next Recommendation**: M7-3 착수 또는 다음 실용적 개선 탐색
