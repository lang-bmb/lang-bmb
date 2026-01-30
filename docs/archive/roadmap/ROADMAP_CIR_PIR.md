# CIR/PIR Implementation Roadmap

## Executive Summary

COMPILER.md에서 정의한 BMB 고유의 IR(CIR, PIR)과 증명 기반 최적화 파이프라인을 구현하기 위한 단계별 로드맵입니다.

**목표**: 증명을 최적화 데이터로 활용하여 런타임 오버헤드 제로 달성

**검증 원칙**: 모든 페이즈는 벤치마크 회귀 테스트를 통과해야 함

---

## Phase Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│  Phase 0: Foundation (v0.52.x)                                      │
│  "현재 암묵적인 계약 정보를 명시적 자료구조로 표현"                 │
│  ├─ CIR 자료구조 정의                                               │
│  ├─ TAST → CIR 변환기                                               │
│  └─ 벤치마크 기준선 확립                                            │
├─────────────────────────────────────────────────────────────────────┤
│  Phase 1: Verification Infrastructure (v0.53.x)                     │
│  "증명 결과를 저장하고 재사용하는 인프라 구축"                      │
│  ├─ Proof Database 설계                                             │
│  ├─ Function Summary 추출                                           │
│  └─ 증분 컴파일 기초                                                │
├─────────────────────────────────────────────────────────────────────┤
│  Phase 2: Proof-Indexed IR (v0.54.x)                                │
│  "모든 표현식에 증명된 사실을 부착"                                 │
│  ├─ PIR 자료구조 정의                                               │
│  ├─ CIR → PIR 변환 (증명 전파)                                      │
│  └─ PIR → MIR 변환                                                  │
├─────────────────────────────────────────────────────────────────────┤
│  Phase 3: Proof-Guided Optimization (v0.55.x)                       │
│  "증명 기반 최적화 패스 구현"                                       │
│  ├─ Bounds Check Elimination (BCE)                                  │
│  ├─ Null Check Elimination (NCE)                                    │
│  ├─ Division Check Elimination (DCE)                                │
│  └─ Unreachable Code Elimination (UCE)                              │
├─────────────────────────────────────────────────────────────────────┤
│  Phase 4: Integration & Polish (v0.56.x → v1.0)                     │
│  "전체 파이프라인 통합 및 최적화"                                   │
│  ├─ End-to-end 파이프라인 통합                                      │
│  ├─ 성능 튜닝                                                       │
│  └─ 문서화 및 안정화                                                │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0: Foundation (v0.52.x)

### Goal
현재 TypeChecker와 MIR에 암묵적으로 존재하는 계약 정보를 **명시적 CIR 자료구조**로 분리

### Tasks

#### 0.1 CIR 자료구조 정의 (v0.52.1)

**파일**: `bmb/src/cir/mod.rs` (신규)

```rust
/// Contract IR - 계약의 논리적 표현
pub struct CirProgram {
    pub functions: Vec<CirFunction>,
    pub type_invariants: HashMap<TypeId, Vec<Proposition>>,
}

pub struct CirFunction {
    pub name: String,
    pub signature: FunctionSignature,

    /// 전제조건 (caller가 보장)
    pub preconditions: Vec<Proposition>,

    /// 후제조건 (callee가 보장)
    pub postconditions: Vec<Proposition>,

    /// 불변식 (항상 참)
    pub invariants: Vec<Proposition>,

    /// 효과 분류
    pub effects: EffectSet,

    /// 함수 본문 (typed expressions)
    pub body: CirExpr,
}

/// 논리 명제
pub enum Proposition {
    /// 상수 true/false
    Const(bool),

    /// 비교: e1 op e2
    Compare(CirExpr, CompareOp, CirExpr),

    /// 논리 연산
    Not(Box<Proposition>),
    And(Vec<Proposition>),
    Or(Vec<Proposition>),
    Implies(Box<Proposition>, Box<Proposition>),

    /// 양화사
    Forall(String, CirType, Box<Proposition>),
    Exists(String, CirType, Box<Proposition>),

    /// 함수 호출 (pure 함수만)
    Call(String, Vec<CirExpr>),
}

pub enum EffectSet {
    Pure,                    // 부작용 없음
    Read(Vec<MemoryRegion>), // 읽기만
    Write(Vec<MemoryRegion>),// 쓰기 포함
    Io,                      // I/O 수행
    Diverge,                 // 무한 루프 가능
}
```

**완료 기준**:
- [ ] `CirProgram`, `CirFunction`, `Proposition` 타입 정의
- [ ] 단위 테스트 10개 이상
- [ ] `cargo test cir` 통과

#### 0.2 TAST → CIR 변환기 (v0.52.2)

**파일**: `bmb/src/cir/lower.rs` (신규)

```rust
/// Typed AST를 CIR로 변환
pub fn lower_to_cir(tast: &TypedProgram) -> CirProgram {
    // 1. 각 함수의 계약 추출
    // 2. 타입 불변식 수집
    // 3. 효과 분석 수행
}
```

**완료 기준**:
- [ ] 모든 계약 어노테이션(@pre, @post, @invariant) 변환
- [ ] 효과 분석 구현 (pure, read, write, io)
- [ ] 기존 테스트 케이스 100% 통과

#### 0.3 CIR 직렬화 및 출력 (v0.52.3)

**CLI 명령어**: `bmb build --emit-cir`

```bash
$ bmb build example.bmb --emit-cir -o example.cir
$ cat example.cir
CirFunction {
  name: "binary_search"
  preconditions: [
    Compare(Call("len", [arr]), Gt, Const(0)),
    Call("is_sorted", [arr])
  ]
  postconditions: [
    Compare(ret, Gte, Const(-1)),
    Compare(ret, Lt, Call("len", [arr]))
  ]
  effects: Pure
}
```

**완료 기준**:
- [ ] `--emit-cir` 플래그 구현
- [ ] JSON 및 텍스트 출력 형식 지원
- [ ] 왕복 테스트 (CIR → 텍스트 → CIR 파싱)

#### 0.4 벤치마크 기준선 확립 (v0.52.4)

**목표**: CIR 도입 후 성능 회귀 없음 확인

```bash
# 기준선 기록
$ ./benchmark_run/run_compare_all3.ps1 > baseline_v0.51.txt

# CIR 도입 후 비교
$ ./benchmark_run/run_compare_all3.ps1 > after_cir_v0.52.txt
$ diff baseline_v0.51.txt after_cir_v0.52.txt
```

**완료 기준**:
- [ ] 16개 벤치마크 모두 ≤102% (2% 허용 오차)
- [ ] 컴파일 시간 ≤110%
- [ ] 벤치마크 결과 기록 (`ecosystem/benchmark-bmb/results/v0.52.txt`)

---

## Phase 1: Verification Infrastructure (v0.53.x)

### Goal
증명 결과를 **저장, 캐싱, 재사용**하는 인프라 구축

### Tasks

#### 1.1 Proof Database 설계 (v0.53.1)

**파일**: `bmb/src/verify/proof_db.rs` (신규)

```rust
/// 증명 결과 데이터베이스
pub struct ProofDatabase {
    /// 함수별 검증 결과
    function_proofs: HashMap<FunctionId, FunctionProofResult>,

    /// 증명 캐시 (LRU)
    cache: LruCache<ProofQuery, ProofResult>,

    /// 파일 해시 (증분 컴파일용)
    file_hashes: HashMap<PathBuf, u64>,
}

pub struct FunctionProofResult {
    /// 검증 상태
    status: VerificationStatus,

    /// 증명된 사실들
    proven_facts: Vec<ProofFact>,

    /// 검증 시간
    verification_time: Duration,

    /// SMT 쿼리 수
    smt_queries: usize,
}

pub struct ProofFact {
    /// 증명된 명제
    proposition: Proposition,

    /// 유효 범위
    scope: ProofScope,

    /// 증거 (디버깅/감사용)
    evidence: ProofEvidence,
}

pub enum ProofScope {
    /// 함수 전체
    Function(FunctionId),

    /// 특정 기본 블록
    Block(BlockId),

    /// 특정 표현식
    Expr(ExprId),

    /// 조건부 (분기 조건 하에서만)
    Conditional(Proposition, Box<ProofScope>),
}
```

**완료 기준**:
- [ ] ProofDatabase CRUD 연산 구현
- [ ] LRU 캐시 동작 검증
- [ ] 직렬화/역직렬화 (증분 컴파일 준비)

#### 1.2 Function Summary 추출 (v0.53.2)

**파일**: `bmb/src/verify/summary.rs` (신규)

```rust
/// 함수 요약 - 호출자 관점의 계약
pub struct FunctionSummary {
    /// 호출 시 필요한 전제조건
    pub requires: Vec<Proposition>,

    /// 반환 시 보장하는 후제조건
    pub ensures: Vec<Proposition>,

    /// 효과 요약
    pub effects: EffectSet,

    /// 종료성 (증명됨/미증명/발산)
    pub termination: TerminationStatus,
}

/// 프로그램 전체의 함수 요약 추출
pub fn extract_summaries(cir: &CirProgram) -> HashMap<FunctionId, FunctionSummary>;
```

**용도**:
- 모듈 간 증명 전파
- 인라이닝 결정
- 증분 컴파일 (요약이 변경되지 않으면 재검증 불필요)

**완료 기준**:
- [ ] 모든 함수에서 요약 추출
- [ ] 요약 기반 모듈 간 검증 테스트

#### 1.3 증분 검증 프로토타입 (v0.53.3)

**파일**: `bmb/src/verify/incremental.rs` (신규)

```rust
/// 증분 검증 관리자
pub struct IncrementalVerifier {
    db: ProofDatabase,
    summaries: HashMap<FunctionId, FunctionSummary>,
}

impl IncrementalVerifier {
    /// 변경된 함수만 재검증
    pub fn verify_incremental(
        &mut self,
        old_cir: &CirProgram,
        new_cir: &CirProgram,
    ) -> VerificationResult {
        // 1. 변경된 함수 식별
        // 2. 영향받는 함수 계산 (역방향 의존성)
        // 3. 필요한 함수만 재검증
        // 4. 캐시 업데이트
    }
}
```

**완료 기준**:
- [ ] 변경 감지 로직 구현
- [ ] 의존성 그래프 구축
- [ ] 증분 검증 30% 이상 시간 단축 (대규모 프로젝트 기준)

#### 1.4 벤치마크 검증: 검증 성능 (v0.53.4)

**새 벤치마크**: `ecosystem/benchmark-bmb/benches/verify/`

| 벤치마크 | 측정 대상 | 목표 |
|---------|----------|------|
| `verify_small.bmb` | 단일 함수 검증 | < 100ms |
| `verify_medium.bmb` | 10개 함수 검증 | < 1s |
| `verify_large.bmb` | 100개 함수 검증 | < 10s |
| `verify_incremental.bmb` | 1개 함수 변경 후 재검증 | < 500ms |

**완료 기준**:
- [ ] 검증 벤치마크 스위트 구축
- [ ] 목표 시간 달성
- [ ] 기존 16개 벤치마크 회귀 없음

---

## Phase 2: Proof-Indexed IR (v0.54.x)

### Goal
**모든 표현식에 증명된 사실을 부착**하여 최적화 패스가 활용할 수 있게 함

### Tasks

#### 2.1 PIR 자료구조 정의 (v0.54.1)

**파일**: `bmb/src/pir/mod.rs` (신규)

```rust
/// Proof-Indexed IR - 증명이 부착된 IR
pub struct PirProgram {
    pub functions: Vec<PirFunction>,
    pub proof_db: ProofDatabase,
}

pub struct PirFunction {
    pub name: String,
    pub params: Vec<(String, PirType)>,
    pub body: PirExpr,

    /// 함수 진입 시점의 증명된 사실들
    pub entry_facts: Vec<ProofFact>,
}

/// 증명이 부착된 표현식
pub struct PirExpr {
    /// 표현식 종류
    pub kind: PirExprKind,

    /// 이 표현식에서 증명된 사실들
    pub proven: Vec<ProofFact>,

    /// 이 표현식의 결과에 대해 알려진 사실들
    pub result_facts: Vec<ProofFact>,

    /// 소스 위치
    pub span: Span,
}

pub enum PirExprKind {
    Literal(Literal),
    Var(VarId),
    Binary(BinaryOp, Box<PirExpr>, Box<PirExpr>),

    /// 배열 인덱싱 - 증명 정보 포함
    Index {
        array: Box<PirExpr>,
        index: Box<PirExpr>,
        /// None이면 bounds check 필요, Some이면 증명됨
        bounds_proof: Option<ProofFact>,
    },

    /// 필드 접근 - null 증명 포함
    Field {
        base: Box<PirExpr>,
        field: String,
        /// None이면 null check 필요, Some이면 증명됨
        null_proof: Option<ProofFact>,
    },

    /// 나눗셈 - 0 검사 증명 포함
    Div {
        lhs: Box<PirExpr>,
        rhs: Box<PirExpr>,
        /// None이면 zero check 필요, Some이면 증명됨
        nonzero_proof: Option<ProofFact>,
    },

    If {
        cond: Box<PirExpr>,
        then_branch: Box<PirExpr>,
        else_branch: Box<PirExpr>,
        /// then 분기에서 추가되는 사실
        then_facts: Vec<ProofFact>,
        /// else 분기에서 추가되는 사실
        else_facts: Vec<ProofFact>,
    },

    // ... 기타 표현식
}
```

**완료 기준**:
- [ ] 모든 표현식 종류에 증명 필드 추가
- [ ] 타입 시스템과 통합

#### 2.2 CIR → PIR 변환 (증명 전파) (v0.54.2)

**파일**: `bmb/src/pir/propagate.rs` (신규)

```rust
/// CIR을 PIR로 변환하며 증명을 전파
pub fn propagate_proofs(cir: &CirProgram, db: &ProofDatabase) -> PirProgram {
    // 1. 함수 진입점에 전제조건을 사실로 등록
    // 2. 각 표현식 순회하며 증명 전파
    // 3. 분기점에서 조건을 사실로 추가
    // 4. 루프에서 불변식 전파
}

/// 증명 전파 규칙
enum PropagationRule {
    /// 전제조건은 함수 전체에서 유효
    PreconditionToFact,

    /// if (cond) 분기에서 cond는 참
    BranchCondition,

    /// while (cond) { body }에서 body 내 cond는 참
    LoopCondition,

    /// let x = e 이후 x에 대한 사실은 e에 대한 사실과 동일
    LetBinding,

    /// 함수 호출 후 후제조건은 사실
    PostconditionAfterCall,
}
```

**예시**:

```bmb
fn example(arr: &[i64], idx: i64)
    pre idx >= 0
    pre idx < arr.len()
= {
    // PIR에서:
    // proven: [idx >= 0, idx < arr.len()]

    let val = arr[idx];
    // Index { bounds_proof: Some(ProofFact { idx < arr.len() }) }
    // → bounds check 제거 가능

    if idx + 1 < arr.len() {
        // then_facts: [idx + 1 < arr.len()]
        // → 여기서 arr[idx + 1] 접근 시 bounds check 제거
        arr[idx + 1]
    } else {
        val
    }
};
```

**완료 기준**:
- [ ] 5가지 전파 규칙 구현
- [ ] 중첩 분기/루프 처리
- [ ] 전파 정확성 테스트 (증명 누락 없음)

#### 2.3 PIR → MIR 변환 (v0.54.3)

**파일**: `bmb/src/pir/lower_to_mir.rs` (신규)

```rust
/// PIR을 MIR로 변환 (증명 정보 보존)
pub fn lower_pir_to_mir(pir: &PirProgram) -> MirProgram {
    // PIR의 증명 정보를 MIR의 어노테이션으로 변환
}
```

MIR 확장:

```rust
// 기존 MIR
pub struct MirStmt {
    pub kind: MirStmtKind,
    pub span: Span,
}

// 확장된 MIR
pub struct MirStmt {
    pub kind: MirStmtKind,
    pub span: Span,
    /// v0.54: 이 문장에서 활용 가능한 증명
    pub available_proofs: Vec<ProofFact>,
}
```

**완료 기준**:
- [ ] PIR → MIR 변환 구현
- [ ] 증명 정보가 MIR까지 전달됨 확인
- [ ] 기존 MIR 최적화 패스와 호환

#### 2.4 벤치마크 검증: PIR 오버헤드 (v0.54.4)

**측정 항목**:

| 메트릭 | 허용 범위 |
|--------|----------|
| 컴파일 시간 증가 | ≤ 20% |
| 메모리 사용량 증가 | ≤ 30% |
| 실행 시간 (16개 벤치마크) | ≤ 100% (회귀 없음) |

**완료 기준**:
- [ ] 오버헤드 측정 스크립트 작성
- [ ] 허용 범위 내 확인
- [ ] 병목 식별 및 문서화

---

## Phase 3: Proof-Guided Optimization (v0.55.x)

### Goal
PIR의 증명 정보를 활용하여 **런타임 검사 제거**

### Tasks

#### 3.1 Bounds Check Elimination (BCE) (v0.55.1)

**파일**: `bmb/src/mir/optimize/bce.rs` (신규)

```rust
/// 경계 검사 제거 패스
pub struct BoundsCheckElimination;

impl OptimizationPass for BoundsCheckElimination {
    fn run(&self, mir: &mut MirFunction) -> bool {
        let mut changed = false;

        for block in &mut mir.blocks {
            for stmt in &mut block.stmts {
                if let MirStmtKind::IndexCheck { array, index, .. } = &stmt.kind {
                    // 증명 검색: index < array.len()
                    if has_proof(&stmt.available_proofs, |p| {
                        matches!(p, Proposition::Compare(
                            idx, Lt, Call("len", arr)
                        ) if idx == index && arr == array)
                    }) {
                        // 검사 제거
                        stmt.kind = MirStmtKind::Nop;
                        changed = true;
                    }
                }
            }
        }

        changed
    }
}
```

**벤치마크 목표**:

| 벤치마크 | 현재 | BCE 적용 후 | 개선 |
|---------|------|------------|------|
| `array_sum` | 100% | ≤ 95% | 5%+ |
| `binary_search` | 100% | ≤ 90% | 10%+ |
| `sorting` | 100% | ≤ 85% | 15%+ |

**완료 기준**:
- [ ] BCE 패스 구현
- [ ] 배열 집약 벤치마크에서 측정 가능한 개선
- [ ] 오탐지 없음 (잘못된 검사 제거 없음)

#### 3.2 Null Check Elimination (NCE) (v0.55.2)

**파일**: `bmb/src/mir/optimize/nce.rs` (신규)

```rust
/// Null 검사 제거 패스
pub struct NullCheckElimination;

impl OptimizationPass for NullCheckElimination {
    fn run(&self, mir: &mut MirFunction) -> bool {
        // pre ptr != null 이면 null 검사 제거
    }
}
```

**벤치마크 목표**:

| 벤치마크 | 현재 | NCE 적용 후 |
|---------|------|------------|
| `linked_list` | 100% | ≤ 90% |
| `tree_traverse` | 100% | ≤ 85% |

**완료 기준**:
- [ ] NCE 패스 구현
- [ ] 포인터 집약 벤치마크에서 개선
- [ ] null 역참조 버그 없음 (soundness 유지)

#### 3.3 Division Check Elimination (DCE) (v0.55.3)

**파일**: `bmb/src/mir/optimize/dce.rs` (신규)

```rust
/// 0 나눗셈 검사 제거 패스
pub struct DivisionCheckElimination;

impl OptimizationPass for DivisionCheckElimination {
    fn run(&self, mir: &mut MirFunction) -> bool {
        // pre divisor != 0 이면 0 검사 제거
    }
}
```

**완료 기준**:
- [ ] DCE 패스 구현
- [ ] 나눗셈 포함 벤치마크에서 확인

#### 3.4 Contract-Based Unreachable Elimination (CUE) (v0.55.4)

**파일**: `bmb/src/mir/optimize/cue.rs` (신규)

```rust
/// 계약 기반 도달 불가 코드 제거
pub struct ContractUnreachableElimination;

impl OptimizationPass for ContractUnreachableElimination {
    fn run(&self, mir: &mut MirFunction) -> bool {
        // pre x > 0 이고 if x <= 0 { ... } 이면
        // x <= 0 분기는 도달 불가 → 제거
    }
}
```

**예시**:

```bmb
fn process(x: i64)
    pre x > 0
= {
    if x <= 0 {
        // 이 분기 전체 제거됨
        // pre x > 0 과 x <= 0 은 모순
        panic("negative")
    }
    compute(x)
};
```

**완료 기준**:
- [ ] CUE 패스 구현
- [ ] 모순 감지 로직 (SMT 또는 간단한 규칙 기반)
- [ ] 코드 크기 감소 측정

#### 3.5 통합 벤치마크 검증 (v0.55.5)

**전체 최적화 효과 측정**:

```bash
# 최적화 전/후 비교 스크립트
$ ./benchmark_run/compare_optimization_impact.ps1

┌─────────────────────────────────────────────────────────────┐
│  Optimization Impact Report (v0.55.x)                       │
├─────────────────────────────────────────────────────────────┤
│  Benchmark         │ Before │ After  │ Improvement │ Checks │
├────────────────────┼────────┼────────┼─────────────┼────────┤
│  array_sum         │ 100ms  │  92ms  │    8%       │ BCE    │
│  binary_search     │  50ms  │  42ms  │   16%       │ BCE    │
│  linked_list       │  80ms  │  68ms  │   15%       │ NCE    │
│  sorting           │ 150ms  │ 125ms  │   17%       │ BCE    │
│  ...               │        │        │             │        │
├────────────────────┼────────┼────────┼─────────────┼────────┤
│  TOTAL             │ 760ms  │ 680ms  │   11%       │        │
└─────────────────────────────────────────────────────────────┘
```

**목표**:
- 배열 집약 벤치마크: 10%+ 개선
- 포인터 집약 벤치마크: 10%+ 개선
- 전체 평균: 5%+ 개선

**완료 기준**:
- [ ] 모든 증명 기반 최적화 패스 통합
- [ ] 목표 성능 달성
- [ ] 회귀 없음 (어떤 벤치마크도 느려지지 않음)

---

## Phase 4: Integration & Polish (v0.56.x → v1.0)

### Goal
전체 파이프라인 통합, 안정화, 문서화

### Tasks

#### 4.1 End-to-End 파이프라인 통합 (v0.56.1)

**수정 파일**: `bmb/src/build/mod.rs`

```rust
/// 새로운 컴파일 파이프라인
pub fn build(config: &BuildConfig) -> BuildResult<()> {
    // Phase 1-2: Lexical + Syntactic
    let tokens = tokenize(&source)?;
    let ast = parse(&filename, &source, tokens)?;

    // Phase 3: Semantic → TAST + CIR
    let tast = type_check(&ast)?;
    let cir = lower_to_cir(&tast);

    // Phase 4: Verification → PIR
    let proof_db = verify(&cir)?;
    let pir = propagate_proofs(&cir, &proof_db);

    // Phase 5: Optimization
    let mir = lower_pir_to_mir(&pir);
    let optimized_mir = optimize(mir, config.opt_level);

    // Phase 6: Emission
    emit(&optimized_mir, config.target)
}
```

**완료 기준**:
- [ ] 전체 파이프라인 동작
- [ ] 이전 버전과 출력 호환성
- [ ] 에러 메시지 품질 유지

#### 4.2 CLI 옵션 확장 (v0.56.2)

```bash
# 새로운 CLI 옵션들
bmb build file.bmb --emit-cir     # CIR 출력
bmb build file.bmb --emit-pir     # PIR 출력
bmb build file.bmb --show-proofs  # 적용된 증명 표시
bmb build file.bmb --opt-report   # 최적화 보고서
```

**완료 기준**:
- [ ] 모든 새 옵션 구현
- [ ] 도움말 문서 업데이트

#### 4.3 성능 튜닝 (v0.56.3)

**최적화 대상**:
- SMT 쿼리 최소화
- 증명 캐시 히트율 개선
- 메모리 사용량 최적화

**목표**:
- 대규모 프로젝트 (10K LOC) 컴파일: < 30초
- 증분 컴파일: 변경 파일당 < 1초

**완료 기준**:
- [ ] 프로파일링 기반 병목 해소
- [ ] 목표 시간 달성

#### 4.4 문서화 및 안정화 (v0.56.4 → v1.0)

**문서**:
- [ ] `docs/CIR.md` - CIR 명세
- [ ] `docs/PIR.md` - PIR 명세
- [ ] `docs/PROOF_OPTIMIZATION.md` - 증명 기반 최적화 가이드
- [ ] `docs/WRITING_CONTRACTS.md` - 최적화 친화적 계약 작성법

**안정화**:
- [ ] API 프리징 (v1.0 호환성 보장)
- [ ] 1000개 이상의 테스트 케이스
- [ ] 실제 프로젝트 적용 테스트

---

## Benchmark Verification Matrix

모든 페이즈에서 실행해야 하는 벤치마크 체크리스트:

| Phase | 실행 벤치마크 | 통과 기준 |
|-------|--------------|----------|
| 0.52.x | 기존 16개 | ≤ 102% |
| 0.53.x | 기존 16개 + 검증 성능 | 회귀 없음 + 검증 목표 달성 |
| 0.54.x | 기존 16개 | ≤ 100% (PIR 오버헤드 상쇄) |
| 0.55.x | 기존 16개 + 최적화 측정 | 전체 평균 5%+ 개선 |
| 0.56.x | 기존 16개 + 대규모 프로젝트 | 안정성 + 성능 목표 |

---

## Risk Mitigation

### Risk 1: SMT 성능 병목

**문제**: Z3 쿼리가 컴파일 시간을 지배
**대응**:
- 타임아웃 설정 (함수당 5초)
- 캐시 적극 활용
- 단순 규칙 기반 증명 우선 시도

### Risk 2: 증명 불완전성

**문제**: SMT가 증명하지 못하는 참인 명제
**대응**:
- 보수적 접근 (증명 못하면 검사 유지)
- 사용자 힌트 어노테이션 (`@assume`)
- 증명 실패 보고서 제공

### Risk 3: 구현 복잡도

**문제**: 새 IR 2개 추가로 유지보수 부담
**대응**:
- 단계적 구현 (각 페이즈 완결성)
- 철저한 테스트
- 기존 코드 리팩토링 병행

---

## Success Criteria for v1.0

| 카테고리 | 메트릭 | 목표 |
|---------|--------|------|
| **성능** | 벤치마크 vs C | ≤ 100% (동등 이상) |
| **최적화** | 증명 기반 검사 제거 | 80%+ 제거율 |
| **컴파일** | 10K LOC 프로젝트 | < 30초 |
| **검증** | Z3 타임아웃 | < 5% 함수 |
| **안정성** | 테스트 커버리지 | > 80% |
| **문서** | API 문서화 | 100% public API |

---

## Timeline Estimate

| Phase | 예상 기간 | 버전 |
|-------|----------|------|
| Phase 0: Foundation | 2-3 주 | v0.52.1 ~ v0.52.4 |
| Phase 1: Verification Infra | 3-4 주 | v0.53.1 ~ v0.53.4 |
| Phase 2: PIR | 4-5 주 | v0.54.1 ~ v0.54.4 |
| Phase 3: Optimization | 4-5 주 | v0.55.1 ~ v0.55.5 |
| Phase 4: Integration | 3-4 주 | v0.56.1 ~ v1.0 |
| **Total** | **16-21 주** | |

---

## Appendix: Task Checklist

### Phase 0 Tasks
- [ ] 0.1: CIR 자료구조 정의
- [ ] 0.2: TAST → CIR 변환기
- [ ] 0.3: CIR 직렬화 및 출력
- [ ] 0.4: 벤치마크 기준선 확립

### Phase 1 Tasks
- [ ] 1.1: Proof Database 설계
- [ ] 1.2: Function Summary 추출
- [ ] 1.3: 증분 검증 프로토타입
- [ ] 1.4: 벤치마크 검증: 검증 성능

### Phase 2 Tasks
- [ ] 2.1: PIR 자료구조 정의
- [ ] 2.2: CIR → PIR 변환 (증명 전파)
- [ ] 2.3: PIR → MIR 변환
- [ ] 2.4: 벤치마크 검증: PIR 오버헤드

### Phase 3 Tasks
- [ ] 3.1: Bounds Check Elimination (BCE)
- [ ] 3.2: Null Check Elimination (NCE)
- [ ] 3.3: Division Check Elimination (DCE)
- [ ] 3.4: Contract-Based Unreachable Elimination (CUE)
- [ ] 3.5: 통합 벤치마크 검증

### Phase 4 Tasks
- [ ] 4.1: End-to-End 파이프라인 통합
- [ ] 4.2: CLI 옵션 확장
- [ ] 4.3: 성능 튜닝
- [ ] 4.4: 문서화 및 안정화
