# Contract-Based Analysis: Unique Value Beyond Traditional Linters

> Research document exploring capabilities unique to contract-based languages like BMB

---

## Background

Traditional linters analyze code syntactically and structurally. BMB, with explicit contracts (preconditions, postconditions, invariants), has access to **semantic information** that enables analysis impossible in conventional languages.

This document explores unique value propositions that contract-based programming enables.

---

## Research Findings

### 1. Semantic Equivalence via Contracts

**Key Insight**: Functions with identical contracts are semantically equivalent, regardless of implementation details.

```bmb
fn max_a(x: i64, y: i64) -> i64
  post ret >= x and ret >= y
  post ret == x or ret == y
= if x > y then x else y;

fn max_b(x: i64, y: i64) -> i64
  post ret >= x and ret >= y
  post ret == x or ret == y
= if y <= x then x else y;
```

These functions have identical contracts. The compiler can prove they are behaviorally equivalent without comparing implementations.

**Validation**: Dafny demonstrates this capability. Academic work on contract subsumption confirms feasibility.

### 2. Contract Subsumption

Contracts can be compared for "stronger than" relationships:

```bmb
-- Strong contract
fn positive_square(x: i64) -> i64
  pre x > 0
  post ret > 0
= x * x;

-- Weaker contract (subsumes the above)
fn non_negative_square(x: i64) -> i64
  pre x >= 0
  post ret >= 0
= x * x;
```

`non_negative_square` can replace `positive_square` in any context (wider precondition, narrower postcondition).

### 3. Contract Conflict Detection

Impossible call chains can be detected at compile time:

```bmb
fn requires_positive(x: i64) -> i64
  pre x > 0
= x + 1;

fn returns_negative() -> i64
  post ret < 0
= -1;

-- ERROR: Provably impossible call
let result = requires_positive(returns_negative());
```

### 4. Specification Mining (Inverse Direction)

Traditional tools (Daikon) infer specifications from execution traces. BMB can invert this:
- Given explicit contracts, infer properties about code structure
- Detect when implementation doesn't fully utilize contract guarantees
- Suggest stronger contracts based on actual implementation

---

## Unique Capabilities Matrix

| Capability | Description | Why Traditional Linters Can't |
|------------|-------------|-------------------------------|
| **Semantic Deduplication** | Find functions with equivalent contracts | Need explicit behavioral specifications |
| **Contract Conflict Detection** | Detect impossible call chains | Can't infer behavioral intent |
| **Specification Coverage** | Measure contract completeness | No contract concept exists |
| **Contract-based Dead Code** | If precondition is unsatisfiable, function unreachable | Only syntactic analysis |
| **API Evolution Safety** | Verify contract subsumption across versions | No semantic comparison available |
| **Auto-generated Tests** | Contracts become property-based tests | Need specification source |
| **Contract Propagation** | Infer contracts for helper functions | No specification to propagate |
| **Performance Hints** | Use postconditions for optimization | Missing semantic guarantees |

---

## Architectural Placement

기능별 적정 위치 구분:

### 1. Type Checker (컴파일 타임, 동일 패키지)

| Feature | 이유 | SMT 필요 |
|---------|------|----------|
| Contract Completeness | AST에서 postcondition 유무만 확인 | No |
| Semantic Duplication | 동일 모듈 내 contract AST 비교 | No (구조적 비교) |

**장점**: 빠름, 항상 실행됨, 즉각 피드백

### 2. Verify Command (SMT 기반, 동일 패키지)

| Feature | 이유 | SMT 필요 |
|---------|------|----------|
| Trivial Contract | `post true` 같은 tautology 검증 필요 | Yes |
| Contract Conflict | call site에서 pre/post 충돌 검사 | Yes |
| Contract Dead Code | precondition satisfiability 검사 | Yes |

**장점**: 이미 Z3 통합됨, `bmb verify` 시에만 실행

### 3. Lint / 별도 도구 (크로스 패키지, 별도 실행)

| Feature | 이유 | SMT 필요 |
|---------|------|----------|
| Contract Subsumption | 두 버전 비교 필요 (old vs new API) | Yes |

**장점**: 버전 간 비교, CI/CD 통합 가능

### 4. Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    bmb check (Type Checker)                  │
│  ┌─────────────────────┐  ┌─────────────────────────────┐   │
│  │ Contract Completeness│  │ Semantic Duplication (AST) │   │
│  └─────────────────────┘  └─────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    bmb verify (SMT/Z3)                       │
│  ┌──────────────────┐ ┌──────────────┐ ┌─────────────────┐  │
│  │ Trivial Contract │ │Contract Conf.│ │Contract DeadCode│  │
│  └──────────────────┘ └──────────────┘ └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    bmb lint (별도 도구)                       │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Contract Subsumption (cross-version API compatibility) │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Roadmap

### Phase 81: Contract Completeness Warning (Implemented v0.81.0)

**Location**: `bmb check` (Type Checker)
**Scope**: 동일 패키지, 컴파일 타임
**SMT**: 불필요
**Status**: ✅ Complete

**Goal**: Warn when functions lack postconditions

```bmb
fn add(a: i64, b: i64) -> i64 = a + b;
-- Warning: function 'add' has no postcondition
```

**Implementation**:
- `check_fn()`에서 `f.post.is_none()` 확인
- `CompileWarning::MissingPostcondition` variant
- AST 검사만으로 충분, SMT 불필요

**Exclusions**:
- `@trust` annotation
- `main` function
- Underscore-prefixed (`_helper`)
- Unit return type (`-> ()`)

### Phase 82: Trivial Contract Warning (Implemented v0.85.0)

**Location**: `bmb verify` (SMT/Z3)
**Scope**: 동일 패키지
**SMT**: 필요
**Status**: ✅ Complete

**Goal**: Warn when contracts are tautologies

```bmb
fn foo(x: i64) -> i64
  post ret == ret  -- Warning: tautology
  post true        -- Warning: tautology
= x;
```

**Implementation**:
- `CompileWarning::TrivialContract` variant in error module
- `ContractVerifier::detect_trivial_contracts()` method
- `is_tautology()` helper: SMT query `NOT(contract)` - if UNSAT, contract is tautology
- Checks preconditions, postconditions, and named contracts
- Integrated into `bmb verify` command

### Phase 83: Contract Conflict Detection (Implemented v0.86.0)

**Location**: `bmb verify` (SMT/Z3)
**Scope**: 동일 패키지, call site 분석
**SMT**: 필요
**Status**: ✅ Complete

**Goal**: Detect provably impossible call chains

```bmb
fn needs_positive(x: i64) pre x > 0 = ...;
fn gives_negative() -> i64 post ret < 0 = ...;

needs_positive(gives_negative());  -- Error: contracts conflict
```

**Implementation**:
- `ContractVerifier::detect_contract_conflicts()` method
- `check_expr_for_conflicts()` recursive AST traversal
- `check_call_for_conflicts()` for call site analysis
- `check_conflict()` SMT verification: `postcondition AND param=ret AND precondition` UNSAT → conflict
- Integrated into `bmb verify` command

### Phase 84: Semantic Duplication Warning (Implemented v0.84.0)

**Location**: `bmb check` (Type Checker)
**Scope**: 동일 모듈 내
**SMT**: 불필요 (구조적 비교)
**Status**: ✅ Complete

**Goal**: Detect functions with equivalent contracts

```bmb
fn max_v1(a: i64, b: i64) -> i64
  post ret >= a and ret >= b
= if a > b { a } else { b };

fn max_v2(a: i64, b: i64) -> i64
  post ret >= a and ret >= b  -- Warning: equivalent to max_v1
= if b <= a { a } else { b };
```

**Implementation**:
- `CompileWarning::SemanticDuplication` variant
- `output::format_expr()` for span-agnostic S-expression comparison
- `output::format_type()` for type signature comparison
- `TypeChecker::contract_signatures` tracks (signature, postcondition) -> function
- 동일 시그니처 + 동일 postcondition = 경고

**Note**: 파라미터 이름이 다르면 다른 contract로 취급 (예: `x, y` vs `a, b`)

### Phase 85: Contract Subsumption Check

**Location**: `bmb lint` (별도 도구)
**Scope**: 크로스 패키지/버전
**SMT**: 필요

**Goal**: Verify API compatibility through contract comparison

```bmb
-- Old API (v1.0)
fn parse(s: &str) -> i64
  pre len(s) > 0
  post ret >= 0;

-- New API (v2.0) - Breaking change!
fn parse(s: &str) -> i64
  pre len(s) > 5  -- Stronger precondition = breaking
  post ret >= 0;
```

**Implementation**:
- 두 버전의 API contract 로드
- Precondition: `new_pre => old_pre` (weaker or equal)
- Postcondition: `old_post => new_post` (stronger or equal)
- gotgan 패키지 버전 비교와 통합

### Phase 86: Contract-Based Dead Code (Implemented v0.86.0)

**Location**: `bmb verify` (SMT/Z3)
**Scope**: 동일 패키지
**SMT**: 필요
**Status**: ✅ Complete

**Goal**: Detect unreachable code via unsatisfiable preconditions

```bmb
fn impossible(x: i64) -> i64
  pre x > 0 and x < 0  -- Unsatisfiable!
= x;
```

**Implementation**:
- `ContractVerifier::detect_unsatisfiable_precondition()` method
- SMT query: precondition SAT check
- If UNSAT, function is unreachable → warning
- Integrated into `bmb verify` command

---

## Priority Matrix

| Phase | Feature | Location | SMT | Impact | Complexity | Priority |
|-------|---------|----------|-----|--------|------------|----------|
| 81 | Contract Completeness | `check` | No | High | Low | **P0** |
| 84 | Semantic Duplication | `check` | No | Medium | Medium | **P1** |
| 82 | Trivial Contract | `verify` | Yes | High | Medium | **P0** |
| 83 | Contract Conflict | `verify` | Yes | High | Medium | **P1** |
| 86 | Contract Dead Code | `verify` | Yes | Medium | Medium | **P1** |
| 85 | Contract Subsumption | `lint` | Yes | Medium | High | **P2** |

### 구현 순서 권장

1. **Phase 81** (check): SMT 없이 즉시 구현 가능
2. **Phase 84** (check): AST 비교로 구현 가능
3. **Phase 82, 83, 86** (verify): 기존 SMT 인프라 활용
4. **Phase 85** (lint): 별도 도구로 분리, 나중에 구현

---

## Related Work

### Design by Contract Languages
- **Eiffel**: Pioneer of DbC (Bertrand Meyer, 1986)
- **JML**: Java Modeling Language with Hoare-style specs
- **Dafny**: Microsoft's verification-aware language
- **Spec#**: C# with contracts (Microsoft Research)

### Academic Research
- Daikon: Dynamic invariant detection
- ESC/Java: Extended Static Checking
- Code Contracts (.NET): Runtime contract checking

### Key Papers
- "Design by Contract" - Bertrand Meyer (1992)
- "Behavioral Subtyping" - Liskov & Wing (1994)
- "Automatic Verification of Pointer Programs" - Dafny team

---

## Conclusion

BMB's explicit contracts provide semantic information enabling:

1. **Behavioral equivalence detection** without implementation comparison
2. **Contract completeness tracking** as a code quality metric
3. **Impossible call chain detection** at compile time
4. **API evolution verification** through subsumption checks

These capabilities are **impossible** in languages without explicit contracts, representing unique value for BMB.

---

**Created**: 2026-01-11
**Status**: Research Complete, Implementation Planned (Phase 81-86)
