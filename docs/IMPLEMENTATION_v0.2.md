# BMB v0.2 Sprout 구현 계획

## 개요

v0.2는 SMT 솔버(Z3)를 연동하여 함수 계약(pre/post)을 정적으로 검증하는 기능을 추가합니다.

## 기술 스택

| 구성요소 | 라이브러리 | 버전 | 용도 |
|----------|-----------|------|------|
| SMT 솔버 | z3 | 0.12 | 고수준 Z3 Rust 바인딩 |
| SMT 이론 | QF_LIA | - | 선형 정수 산술 (Quantifier-Free Linear Integer Arithmetic) |

## 검증 원리

### 계약 검증 방식

```
함수: fn f(x: T) -> R pre P post Q = body

검증할 조건:
1. Pre 만족성: ∃x. P(x) - 유효한 입력이 존재하는가?
2. Post 정확성: ∀x. P(x) ∧ (ret = body) → Q(x, ret)
   - 반증: ∃x. P(x) ∧ (ret = body) ∧ ¬Q(x, ret) 가 UNSAT이면 검증 성공
```

### SMT 변환 규칙

| BMB 표현식 | SMT-LIB |
|-----------|---------|
| `i32`, `i64` | `Int` |
| `bool` | `Bool` |
| `a + b` | `(+ a b)` |
| `a - b` | `(- a b)` |
| `a * b` | `(* a b)` |
| `a / b` | `(div a b)` |
| `a % b` | `(mod a b)` |
| `a == b` | `(= a b)` |
| `a != b` | `(not (= a b))` |
| `a < b` | `(< a b)` |
| `a <= b` | `(<= a b)` |
| `a > b` | `(> a b)` |
| `a >= b` | `(>= a b)` |
| `a and b` | `(and a b)` |
| `a or b` | `(or a b)` |
| `not a` | `(not a)` |
| `if c then t else e` | `(ite c t e)` |
| `ret` (post) | `__ret__` 변수 |

## 프로젝트 구조

```
bmb/src/
├── smt/
│   ├── mod.rs          # SMT 모듈 진입점
│   ├── context.rs      # Z3 컨텍스트 래퍼
│   ├── translator.rs   # AST → Z3 AST 변환
│   └── verifier.rs     # 계약 검증 로직
├── verify/
│   ├── mod.rs          # 검증 모듈 진입점
│   ├── contract.rs     # pre/post 검증
│   └── counterexample.rs # 반례 추출 및 포매팅
```

## 구현 페이즈

### Phase 1: SMT 모듈 기반 (smt/)

1. `context.rs`: Z3 Context/Config 관리
2. `translator.rs`: BMB Expr → Z3 AST 변환
   - 산술 연산 변환
   - 비교 연산 변환
   - 논리 연산 변환
   - 조건문 변환

### Phase 2: Z3 바인딩 연동

1. Cargo.toml에 z3 의존성 추가
2. 기본 연결 테스트
3. 정수/불린 변수 생성
4. 제약조건 추가 및 검사

### Phase 3: 계약 검증기 (verify/)

1. `contract.rs`:
   - Pre-condition 검증: 입력 범위 확인
   - Post-condition 검증: 출력 보장 확인
   - 함수 본문을 `ret` 변수에 바인딩

### Phase 4: 반례 리포터

1. `counterexample.rs`:
   - Z3 Model에서 값 추출
   - 사용자 친화적 포맷 생성
   - ariadne와 통합하여 에러 출력

### Phase 5: CLI 통합

1. `bmb verify <file>` 명령어 추가
2. 검증 결과 출력 포맷
3. 에러 코드 정의

## 예상 출력

### 검증 성공
```
$ bmb verify examples/safe_divide.bmb
✓ safe_divide: pre verified
✓ safe_divide: post verified
All contracts verified successfully.
```

### 검증 실패 (반례)
```
$ bmb verify examples/bad_abs.bmb
✗ bad_abs: post verification failed

  │ fn bad_abs(x: i32) -> i32
  │   post ret >= 0
  │ = x;
  │   ─ returns x directly without abs
  │
  │ Counterexample:
  │   x = -1
  │   ret = -1 (violates: ret >= 0)
```

## 테스트 케이스 계획

### 검증 성공 케이스
- `verify_001_identity.bmb`: 항등 함수 (pre/post 없음)
- `verify_002_positive_guard.bmb`: `pre x > 0 post ret > 0`
- `verify_003_addition.bmb`: 덧셈 결과 검증
- `verify_004_if_else.bmb`: 조건문 검증

### 검증 실패 케이스 (반례 생성)
- `fail_001_missing_abs.bmb`: abs 없이 음수 반환
- `fail_002_division_by_zero.bmb`: 0으로 나눔 가능
- `fail_003_overflow.bmb`: 오버플로우 미검증

## 제한사항 (v0.2)

1. **지원 타입**: i32, i64, bool만 검증
2. **비선형 산술**: 곱셈은 상수와의 곱만 완전 지원
3. **함수 호출**: 인라인 또는 @trust 필요
4. **재귀**: 미지원 (v0.3+)
5. **배열/참조**: 미지원 (v0.3+)

## 의존성 변경

```toml
[dependencies]
z3 = "0.12"
```

## 마일스톤

- [ ] Phase 1: SMT 모듈 기반 구현
- [ ] Phase 2: Z3 연동 및 기본 테스트
- [ ] Phase 3: pre/post 검증 로직
- [ ] Phase 4: 반례 추출 및 포매팅
- [ ] Phase 5: CLI 통합
- [ ] Phase 6: 테스트 및 문서화
