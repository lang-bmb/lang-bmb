# BMB v0.2 Implementation Plan

> **ARCHIVED**: This plan was completed on 2026-01-02.
> Current version is v0.10. See [ROADMAP.md](ROADMAP.md) for current status.

---

> Spec Proposal 통합 + SMT(Z3) 기반 계약 검증

## Executive Summary

`local-docs/spec-sugest.md` 제안서 분석 결과, v0.2를 **SMT 통합 + Spec 개선**으로 재정의합니다.

| 결정 | 항목 수 | 설명 |
|------|---------|------|
| ✅ 수용 | 5개 | 범위연산자, ret바인딩, 계약블록, 상태참조, 정제타입 |
| ❌ 거절 | 4개 | 통합정의구문, SIMD타입, low블록, 24키워드교체 |
| 🔄 수정 | 1개 | 성능속성 16개 → 5개 |

---

## ✅ 수용 항목

### 1. 범위 연산자 확장

**현재**: `0..n` (의미 모호)

**v0.2**:
```bmb
0..<n    -- [0, n) 반열린 (기본)
0..=n    -- [0, n] 닫힌
0..n     -- 기존 호환 = 0..<n 으로 처리
```

**구현**:
- `token.rs`: `DotDotLt`, `DotDotEq` 추가
- `grammar.lalrpop`: `RangeExpr` 확장
- `ast/expr.rs`: `RangeKind` enum

### 2. ret 명시적 바인딩

**현재**: 암시적 `ret` 키워드

**v0.2**:
```bmb
fn divide(a: i64, b: i64) -> r: i64  -- r이 반환값 바인딩
  where r * b == a
= a / b;
```

**구현**:
- `grammar.lalrpop`: `-> <name:RawIdent> ":" <ty:SpannedType>`
- `ast/mod.rs`: `FnDef.ret_name: Option<String>`
- 기존 `-> Type` 문법도 계속 지원 (ret_name = None)

### 3. 이름 있는 계약 (`where {}` 블록)

**현재**: `pre expr` / `post expr` 분리

**v0.2**:
```bmb
fn binary_search(arr: &[i64], target: i64) -> r: ?usize
  where {
    sorted_input: forall(i in 0..<len(arr)-1): arr[i] <= arr[i+1],
    found_correct: r is Some(i) => arr[i] == target,
    not_found_correct: r is None => forall(x in arr): x != target
  }
= ...;
```

**구현**:
- `token.rs`: `where` 키워드 추가
- `grammar.lalrpop`: `where "{" <contracts:NamedContracts> "}"`
- `ast/mod.rs`:
```rust
pub struct NamedContract {
    pub name: Option<Spanned<String>>,
    pub condition: Spanned<Expr>,
    pub span: Span,
}
```

**이점**:
- 에러 메시지: "Contract 'sorted_input' violated"
- pre/post 통합 관리

### 4. 상태 참조 `.pre`, `.post`

**현재**: `old(x)` 함수 스타일

**v0.2**:
```bmb
fn increment(x: &mut i64) -> ()
  where x.post == x.pre + 1
= *x = *x + 1;
```

**구현**:
- `token.rs`: 별도 토큰 불필요 (`.` + `pre`/`post`)
- `grammar.lalrpop`: `PostfixExpr`에 `.pre`, `.post` 추가
- `ast/expr.rs`:
```rust
pub enum StateKind { Pre, Post }

pub enum Expr {
    StateRef {
        expr: Box<Spanned<Expr>>,
        state: StateKind,
    },
    // ...
}
```
- `smt/translator.rs`: `x.pre` → `x_pre`, `x.post` → `x_post`

### 5. 인라인 정제 타입

**현재**: `Type where P` (별도 정의 필요)

**v0.2**:
```bmb
fn safe_divide(a: i64, b: i64{!= 0}) -> i64 = a / b;

fn clamp(x: i64, lo: i64, hi: i64{>= lo}) -> i64{>= lo, <= hi}
= if x < lo then lo else if x > hi then hi else x;
```

**구현**:
- `grammar.lalrpop`:
```lalrpop
Type: Type = {
    <base:BaseType> "{" <constraints:RefinementConstraints> "}" =>
        Type::Refined { base: Box::new(base), constraints },
    // ...
};

RefinementConstraints: Vec<Spanned<Expr>> = {
    <v:(<SpannedExpr> ",")*> <e:SpannedExpr?> => ...
};
```
- `ast/types.rs`:
```rust
pub enum Type {
    Refined {
        base: Box<Type>,
        constraints: Vec<Spanned<Expr>>,
    },
    // ...
}
```

---

## 🔄 수정 항목

### 성능 속성: 16개 → 5개

**수용 (LLVM 없이 활용 가능)**:

| 속성 | 용도 | SMT 활용 |
|------|------|----------|
| `@inline` | 인라인 힌트 | 인터프리터 최적화 |
| `@pure` | 순수 함수 마킹 | 부작용 검증 |
| `@decreases(expr)` | 종료성 증명 | 재귀 검증 |
| `@invariant(expr)` | 루프 불변식 | 루프 검증 |
| `@likely/@unlikely` | 분기 힌트 | 향후 LLVM 매핑 |

**거절 (LLVM 의존)**:
- `@aligned`, `@contiguous`, `@disjoint`, `@exclusive`
- `@vectorize`, `@tile`, `@unroll`, `@fusable`
- `@temporal`, `@nontemporal`

→ v0.4 LLVM 통합 시 재검토

---

## ❌ 거절 항목

### 1. 통합 정의 구문 `def Name: Kind = Body`

**거절 이유**:

1. **LR(1) 파싱 복잡도**
   - `def foo: (x: i32) -> r: i32` - 첫 번째 `:` 후 Kind vs 파라미터 구분 불가
   - 룩어헤드 증가 또는 백트래킹 필요

2. **가독성 저하**
   - `fn`을 보면 함수, `struct`를 보면 구조체 - 즉시 파악
   - `def`는 Kind까지 읽어야 타입 파악

3. **기존 생태계 호환성**
   - Rust 개발자에게 친숙한 문법 유지
   - AI도 `fn`/`struct`/`enum` 패턴 학습 가능

### 2. SIMD 타입 (`f32x8`, `i64x4` 등)

**거절 이유**:
- LLVM 백엔드 없이 의미 없음
- 인터프리터에서 시뮬레이션은 성능 이점 없음
- v0.4 LLVM 통합 후 재검토

### 3. `low { }` 블록

**거절 이유**:
- SIMD 타입 의존
- 포인터 연산 검증에 고급 SMT 모델 필요
- v0.4+ 검토

### 4. 24개 키워드 전체 교체

**거절 이유**:
- stdlib 140+ 함수 전면 무효화
- 점진적 전환 불가
- 기존 테스트 케이스 전체 수정 필요

---

## Implementation Timeline

### Week 1-2: 기반 문법 ✅ 완료

| 일 | 작업 | 파일 | 상태 |
|----|------|------|------|
| 1-2 | 범위 연산자 `..<`, `..=` | token.rs, grammar.lalrpop, expr.rs | ✅ |
| 3-4 | ret 명시적 바인딩 | grammar.lalrpop, mod.rs | ✅ |
| 5-7 | 속성 파서 `@name`, `@name(args)` | token.rs, grammar.lalrpop, mod.rs | ✅ |

**구현 완료 (2026-01-02)**:
- `RangeKind::Exclusive` (..<, ..) 및 `RangeKind::Inclusive` (..=) 추가
- `FnDef.ret_name: Option<Spanned<String>>` 필드 추가
- `Attribute` enum (Simple, WithArgs) 및 `FnDef.attributes` 필드 추가
- 테스트 케이스: `012_range_operators.bmb`, `013_ret_binding.bmb`, `014_attributes.bmb`

### Week 3-4: 계약 시스템 ✅ 완료

| 일 | 작업 | 파일 | 상태 |
|----|------|------|------|
| 1-3 | `where {}` 블록 파싱 | grammar.lalrpop | ✅ |
| 4-5 | NamedContract AST | mod.rs | ✅ |
| 6-7 | 상태 참조 `.pre`, `.post` | grammar.lalrpop, expr.rs | ✅ |
| 8-9 | `it` 키워드 (정제 자기참조) | token.rs, expr.rs | ✅ |

**구현 완료 (2026-01-02)**:
- `where { name: condition }` 블록 파싱 및 NamedContract AST
- `StateKind` enum (Pre, Post) 및 `Expr::StateRef` 추가
- `.pre`, `.post` 접미사로 상태 참조 (SMT: x_pre, x_post)
- 테스트 케이스: `015_where_contracts.bmb`

### Week 5-6: 정제 타입 + SMT ✅ 완료

| 일 | 작업 | 파일 | 상태 |
|----|------|------|------|
| 1-3 | 인라인 정제 타입 파싱 | grammar.lalrpop, types.rs | ✅ |
| 4-5 | `it` 키워드로 정제값 참조 | token.rs, grammar.lalrpop | ✅ |
| 6-7 | Type::Refined AST 및 핸들러 | types.rs, lower.rs, translator.rs | ✅ |

**구현 완료 (2026-01-02)**:
- `T{constraints}` 정제 타입 파싱 (i32, i64, f64, bool)
- `it` 키워드로 정제되는 값 참조 (e.g., `i64{it > 0}`)
- Type::Refined 핸들러 (lower.rs, translator.rs, types/mod.rs, main.rs)
- 테스트 케이스: `016_inline_refinement.bmb`

### Week 7: 검증 시스템 통합 ✅ 완료

| 일 | 작업 | 파일 | 상태 |
|----|------|------|------|
| 1-2 | NamedContract 검증 통합 | verify/contract.rs | ✅ |
| 3-4 | 정제 타입 제약조건 검증 | verify/contract.rs, translator.rs | ✅ |
| 5-6 | 검증 테스트 케이스 작성 | verify/contract.rs (tests) | ✅ |

**구현 완료 (2026-01-02)**:
- `where {}` 블록의 NamedContract가 SMT 검증에 통합
- `FunctionReport`에 `contract_results`, `refinement_results` 필드 추가
- 반환 타입 정제 (`-> i64{it >= 0}`) 검증 지원
- `ret_name` 명시적 바인딩의 SMT 변수 선언
- 단위 테스트 6개 추가 (총 33개 테스트)

### Week 8: 마이그레이션 ✅ 완료

| 일 | 작업 | 파일 | 상태 |
|----|------|------|------|
| 1-3 | 정제 타입 타입체커 수정 | types/mod.rs, ast/types.rs | ✅ |
| 4-5 | v0.2 showcase 함수 작성 | 017_stdlib_v02_showcase.bmb | ✅ |
| 6-8 | 테스트 케이스 확장 | tests/**/*.bmb | ✅ |
| 9-10 | 문서 업데이트 | docs/*.md | ✅ |

**구현 완료 (2026-01-02)**:
- `Type::base_type()` 헬퍼 메서드 추가 (정제 타입의 기저 타입 추출)
- `Type::is_numeric()`, `Type::is_comparable()` 헬퍼 메서드 추가
- 타입체커: 산술/비교/배열인덱스 연산에서 정제 타입 처리
- 대표 stdlib 함수 12개를 v0.2 문법으로 마이그레이션 (`017_stdlib_v02_showcase.bmb`)
- Z3 미설치 환경에서 graceful degradation 확인

---

## Migration Examples

### Before (v0.1 현재)

```bmb
fn safe_divide(a: i64, b: i64) -> i64
  pre b != 0
  post ret * b == a
= a / b;

fn clamp(x: i64, lo: i64, hi: i64) -> i64
  pre lo <= hi
  post ret >= lo and ret <= hi
= if x < lo then lo else if x > hi then hi else x;
```

### After (v0.2)

```bmb
fn safe_divide(a: i64, b: i64{!= 0}) -> r: i64
  where {
    correct: r * b == a
  }
= a / b;

fn clamp(x: i64, lo: i64, hi: i64{>= lo}) -> r: i64{>= lo, <= hi}
= if x < lo then lo else if x > hi then hi else x;
```

---

## File Change Summary

| 파일 | 변경 유형 | 설명 |
|------|----------|------|
| `bmb/src/lexer/token.rs` | Modify | `..<`, `..=`, `where`, `@`, `it` 토큰 |
| `bmb/src/grammar.lalrpop` | Modify | 모든 새 문법 규칙 |
| `bmb/src/ast/mod.rs` | Modify | NamedContract, Attribute |
| `bmb/src/ast/expr.rs` | Modify | RangeKind, StateKind, StateRef, It |
| `bmb/src/ast/types.rs` | Modify | Type::Refined |
| `bmb/src/types/mod.rs` | Modify | 정제 타입 체크, base_type() 헬퍼 |
| `bmb/src/ast/types.rs` | Modify | Type::base_type(), is_numeric(), is_comparable() |
| `bmb/src/smt/translator.rs` | Modify | 새 AST 노드 번역, ret_name 선언 |
| `bmb/src/verify/contract.rs` | Modify | NamedContract/정제타입 검증 통합 |
| `bmb/src/error/mod.rs` | Modify | 이름 있는 계약 에러 |
| `tests/examples/valid/017_stdlib_v02_showcase.bmb` | New | v0.2 문법 showcase (12개 함수) |
| `stdlib/**/*.bmb` | Migrate | 새 문법 적용 (v0.2.1+) |
| `tests/**/*.bmb` | Migrate | 새 문법 테스트 |

---

## Success Criteria

- [x] `..<`, `..=` 범위 연산자 작동 ✅
- [x] `-> r: Type` 명시적 ret 바인딩 작동 ✅
- [x] `where { name: constraint }` 파싱 및 검증 ✅
- [x] `.pre`, `.post` 상태 참조 작동 ✅
- [x] `T{constraints}` 정제 타입 파싱 및 체크 ✅
- [x] `it` 키워드로 정제값 자기참조 ✅
- [x] `@inline`, `@pure`, `@decreases`, `@invariant` 속성 파싱 ✅
- [x] stdlib 대표 함수 마이그레이션 (12개 showcase) ✅
- [x] 정제 타입 타입체커 통합 (`Type::base_type()`) ✅
- [x] 기존 테스트 + 새 테스트 통과 ✅ (33개)
- [x] Z3 검증 시스템 통합 ✅ (where 블록, 정제 타입)

---

## Notes

1. **하위 호환성**: 기존 `pre`/`post`, `old()`, `..` 문법은 deprecated 경고 후 v0.3에서 제거
2. **점진적 마이그레이션**: 기존 문법과 새 문법 모두 파싱 가능하도록 전환 기간 제공
3. **SIMD/low 블록**: v0.4 LLVM 통합 시 별도 RFC로 재검토
4. **stdlib 전체 마이그레이션**: 140+ 함수 전체 마이그레이션은 기계적 작업으로, v0.2.1 패치에서 점진적 진행

---

*Last Updated: 2026-01-02*
*Week 1-2 구현 완료: 2026-01-02*
*Week 3-6 구현 완료: 2026-01-02*
*Week 7 검증 시스템 통합 완료: 2026-01-02*
*Week 8 마이그레이션 완료: 2026-01-02*
*v0.2 핵심 기능 완료: 2026-01-02*
