# BMB 언어 스펙 / 컴파일러 개선 계획

> 작성일: 2026-01-19
> 버전: v1.0
> 상태: Draft

---

## 목표

```
┌─────────────────────────────────────────────────────────────┐
│                    BMB 성능 목표                             │
├─────────────────────────────────────────────────────────────┤
│  이론적 최적 대비: 97~99%                                    │
│  현재 Rust 대비:  동등 또는 우위                             │
│  C 대비:          10% 이내 (안전성 포함)                     │
├─────────────────────────────────────────────────────────────┤
│  핵심 차별화: "Contract으로 증명된 Zero-Cost Safety"         │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: 안전성 검사 제거 (Q1)

### 목표
- Bounds check 런타임 비용: 0.5~3% → 0%
- Overflow check 비용: ~1% → 0%
- 예상 성능 개선: +3~5%

---

### 1.1 의존 타입: Fin[N]

#### 언어 스펙

```bmb
// Fin[N]: 0 이상 N 미만의 정수
type Fin[N: usize] = usize where 0 <= self && self < N;

// 배열 타입과 연동
type Array[N: usize, T] = [T; N];

// 안전한 인덱싱 (bounds check 없음)
fn get[N, T](arr: Array[N, T], i: Fin[N]) -> T = arr[i];
```

#### 자동 추론 규칙

```bmb
// 규칙 1: 범위 루프 인덱스
for i in 0..N { ... }
// i: Fin[N] 자동 추론

// 규칙 2: 배열 인덱스 루프
for i in arr.indices() { ... }
// i: Fin[arr.len] 자동 추론

// 규칙 3: 검증된 변환
fn to_fin[N](x: usize) -> Option[Fin[N]] =
    if x < N { Some(x as Fin[N]) }  // 1회 검사
    else { None };
```

#### 컴파일러 구현

```
AST 단계:
  - IndexExpr 노드에 인덱스 타입 정보 추가

Types 단계:
  - Fin[N] 타입 추가
  - 범위 루프 → Fin 자동 추론
  - 정제 타입 → Fin 변환 규칙

MIR 단계:
  - bounds_check 플래그 추가
  - Fin 인덱스 → bounds_check = false

Codegen 단계:
  - bounds_check = false → 검사 코드 생략
```

#### 테스트 케이스

```bmb
// TC-1.1.1: 정적 인덱스
fn test_static() = {
    let arr: [i64; 5] = [1, 2, 3, 4, 5];
    let i: Fin[5] = 2;
    assert(arr[i] == 3);  // 검사 없어야 함
};

// TC-1.1.2: 루프 인덱스
fn test_loop() = {
    let arr: [i64; 10] = [...];
    let mut sum = 0;
    for i in 0..10 {
        sum = sum + arr[i];  // 검사 없어야 함
    }
};

// TC-1.1.3: 동적 인덱스
fn test_dynamic(x: usize) = {
    let arr: [i64; 100] = [...];
    match x.to_fin[100]() {
        Some(i) => arr[i],  // 검사 없어야 함
        None => 0,
    }
};
```

---

### 1.2 범위 산술 추론

#### 언어 스펙

```bmb
// 범위 타입 선언
type Percentage = i64 where 0 <= self && self <= 100;
type Byte = i64 where 0 <= self && self <= 255;

// 범위 산술 규칙 (컴파일러 내장)
// Percentage + Percentage → i64{0..200}
// Byte * Byte → i64{0..65025}
```

#### 추론 규칙

```
덧셈: T{a..b} + T{c..d} → T{a+c..b+d}
뺄셈: T{a..b} - T{c..d} → T{a-d..b-c}
곱셈: T{a..b} * T{c..d} → T{min(ac,ad,bc,bd)..max(ac,ad,bc,bd)}
나눗셈: T{a..b} / T{c..d} where c > 0 || d < 0
        → T{min(a/c,a/d,b/c,b/d)..max(a/c,a/d,b/c,b/d)}
```

#### 타입 승격 규칙

```bmb
// 자동 승격 (오버플로우 방지)
i8 * i8   → i16
i16 * i16 → i32
i32 * i32 → i64
i64 * i64 → i128 또는 컴파일 에러

// 명시적 승격 함수
fn widen[From, To](x: From) -> To
  where sizeof(To) > sizeof(From);
```

#### 컴파일러 구현

```
Types 단계:
  - RefinementType에 range 필드 추가
  - 산술 연산자 타입 규칙 확장
  - 범위 연산 로직 구현

SMT 단계:
  - 범위 정보 → Z3 assertion 생성
  - 오버플로우 불가능 증명

Codegen 단계:
  - 오버플로우 증명됨 → 검사 코드 생략
```

---

### 1.3 증명 캐싱 시스템

#### 설계

```
.bmb/cache/
├── proofs/
│   ├── <hash>.proof    # 증명 결과 캐시
│   └── manifest.json   # 증명 메타데이터
└── smt/
    └── <hash>.smt2     # SMT 쿼리 캐시
```

#### 구현

```rust
// 증명 캐시 키
struct ProofKey {
    function_hash: u64,      // 함수 시그니처 해시
    contract_hash: u64,      // contract 해시
    dependency_hash: u64,    // 의존 함수 해시
}

// 캐시 조회
fn lookup_proof(key: &ProofKey) -> Option<ProofResult>;

// 증분 검증
fn verify_incremental(changed_functions: &[FnId]) -> Vec<ProofResult>;
```

---

## Phase 2: Aliasing 최적화 (Q2)

### 목표
- LLVM noalias 활용률: ~0% → 100%
- 벡터화 성공률: ~70% → 95%+
- 예상 성능 개선: +2~5%

---

### 2.1 disjoint Predicate

#### 언어 스펙

```bmb
// disjoint: 두 참조가 겹치지 않음을 표현
fn compute(a: &mut [i64], b: &[i64], c: &[i64])
  pre disjoint(a, b)
  pre disjoint(a, c)
= {
    for i in a.indices() {
        a[i] = b[i] + c[i];  // SIMD 벡터화 가능
    }
};

// disjoint 정의
predicate disjoint[T](a: &[T], b: &[T]) =
    a.ptr + a.len <= b.ptr || b.ptr + b.len <= a.ptr;
```

#### 컴파일러 구현

```
AST 단계:
  - disjoint predicate 파싱

Types 단계:
  - disjoint 타입 검사
  - 참조 분리 분석

MIR 단계:
  - disjoint 정보 보존
  - noalias 마커 추가

Codegen (LLVM):
  - disjoint 참조 → noalias attribute
  - 함수 파라미터 → noalias, nonnull
```

---

### 2.2 효과 시스템

#### 언어 스펙

```bmb
// 효과 선언
effect reads[T];
effect writes[T];
effect allocates;
effect pure;  // 부작용 없음

// 함수 효과 명시
fn sum(arr: &[i64]) -> i64
  effects { reads[arr] }
= { ... };

fn mutate(arr: &mut [i64])
  effects { writes[arr] }
= { ... };

fn pure_add(a: i64, b: i64) -> i64
  effects { pure }
= a + b;

// 효과 추론 (명시하지 않으면 자동 추론)
fn auto_inferred(x: &mut [i64], y: &[i64]) = {
    // effects { reads[y], writes[x] } 자동 추론
};
```

#### 컴파일러 구현

```
AST 단계:
  - EffectClause 노드 추가
  - 효과 파싱

Types 단계:
  - 효과 시스템 타입 검사
  - 효과 추론 알고리즘
  - pure 함수 검증

MIR 단계:
  - 효과 정보 보존
  - 최적화 힌트 생성

Codegen 단계:
  - pure → readonly, nounwind
  - reads[x] → x에 대해 readonly
  - writes[x] → x에 대해서만 수정
```

---

### 2.3 Unique 타입 (선택적)

#### 언어 스펙

```bmb
// Unique[T]: 유일한 소유권
type Unique[T];

// 생성
fn create() -> Unique[[i64]] = Unique::new([1, 2, 3]);

// 소비 (이동)
fn consume(data: Unique[[i64]]) = {
    // data 소유권 이동
};

// 빌림
fn borrow(data: &Unique[[i64]]) = {
    // 읽기만 가능
};

fn borrow_mut(data: &mut Unique[[i64]]) = {
    // 유일한 가변 참조
};
```

---

## Phase 3: 컴파일러 체인 최적화 (Q3)

### 목표
- LTO 기본 활성화
- PGO 자동 통합
- 예상 성능 개선: +10~20%

---

### 3.1 LTO 지원

#### CLI 인터페이스

```bash
# 기본: ThinLTO 활성화
bmb build main.bmb -o out

# Full LTO (최대 최적화, 느린 빌드)
bmb build main.bmb -o out --lto=fat

# LTO 비활성화 (빠른 빌드)
bmb build main.bmb -o out --lto=off
```

#### 프로젝트 설정

```toml
# bmb.toml
[build]
lto = "thin"  # "off", "thin", "fat"

[build.release]
lto = "fat"
opt-level = 3
```

#### 컴파일러 구현

```
Codegen 단계:
  - LLVM Module에 LTO 메타데이터 추가
  - ThinLTO: 요약 정보 생성
  - Fat LTO: 전체 비트코드 보존

Linking 단계:
  - lld 또는 LLVM linker 호출
  - LTO 패스 실행
```

---

### 3.2 PGO 지원

#### CLI 인터페이스

```bash
# 1단계: 프로파일 생성 빌드
bmb build main.bmb -o out --pgo=generate

# 2단계: 프로그램 실행 (프로파일 수집)
./out  # default.profraw 생성

# 3단계: 프로파일 적용 빌드
bmb build main.bmb -o out --pgo=use --pgo-data=default.profdata
```

#### 자동화 스크립트

```bash
# bmb pgo 명령어
bmb pgo main.bmb --run="./out benchmark_input" -o out
# 내부적으로 3단계 자동 실행
```

#### 컴파일러 구현

```
Codegen 단계 (generate):
  - LLVM 프로파일 instrumentation 추가
  - -fprofile-generate 플래그

Codegen 단계 (use):
  - 프로파일 데이터 로드
  - -fprofile-use 플래그
  - 핫 패스 최적화
```

---

### 3.3 벡터화 진단

#### CLI 인터페이스

```bash
# 벡터화 리포트
bmb build main.bmb --report=vectorization

# 출력 예시:
# ✅ sum (line 10): vectorized (AVX2, 4x unroll)
# ❌ compute (line 25): not vectorized
#    reason: potential aliasing between 'a' and 'b'
#    suggestion: add `pre disjoint(a, b)`
```

#### 어트리뷰트

```bmb
@vectorize  // 벡터화 요청 (실패 시 경고)
fn must_vectorize(a: &mut [f64], b: &[f64]) = { ... };

@vectorize(require)  // 벡터화 필수 (실패 시 에러)
fn critical_loop(a: &mut [f64], b: &[f64]) = { ... };

@novectorize  // 벡터화 금지
fn small_loop(a: &mut [i64; 4]) = { ... };
```

---

## Phase 4: 고급 기능 (Q4)

### 목표
- 추가 최적화 기회 확보
- 예상 성능 개선: +1~2%

---

### 4.1 Sealed Trait

#### 언어 스펙

```bmb
// sealed: 이 파일/모듈 외부에서 구현 불가
sealed trait Shape {
    fn area(&self) -> f64;
}

struct Circle { radius: f64 }
struct Square { side: f64 }

impl Shape for Circle {
    fn area(&self) -> f64 = 3.14159 * self.radius * self.radius;
}

impl Shape for Square {
    fn area(&self) -> f64 = self.side * self.side;
}

// 컴파일러가 Shape = Circle | Square 임을 알고 최적화
fn process(s: &dyn Shape) -> f64 = s.area();
// → match로 변환 → 인라이닝 가능
```

---

### 4.2 FP 정밀도 타입

#### 언어 스펙

```bmb
// 정밀도 타입
type Strict[T] = T;      // IEEE 754 엄격 준수
type Relaxed[T] = T;     // fast-math 허용

// 타입 별칭
type f64_strict = Strict[f64];
type f64_fast = Relaxed[f64];

// 사용
fn scientific(x: f64_strict) -> f64_strict = x * x;
fn graphics(x: f64_fast) -> f64_fast = x * x;

// 블록 수준 제어
fn mixed() -> f64 = {
    let a = @strict { kahan_sum(data) };
    let b = @relaxed { approx_sqrt(x) };
    a + b
};
```

---

### 4.3 분기 힌트

#### 언어 스펙

```bmb
// likely/unlikely
if likely(common_case) {
    fast_path();
} else {
    slow_path();
}

if unlikely(error) {
    handle_error();
}

// 컴파일 타임 분기
fn process[T]() = {
    if const (T == i32) {
        i32_path();
    } else if const (T == f64) {
        f64_path();
    }
};
```

---

## 구현 일정

```
2026 Q1: Phase 1 (안전성 검사 제거)
├── Week 1-4:  Fin[N] 타입 설계 및 구현
├── Week 5-8:  범위 산술 추론
├── Week 9-10: 증명 캐싱
└── Week 11-12: 테스트 및 벤치마크

2026 Q2: Phase 2 (Aliasing 최적화)
├── Week 1-4:  disjoint predicate
├── Week 5-8:  효과 시스템
├── Week 9-10: LLVM noalias 생성
└── Week 11-12: 벡터화 검증

2026 Q3: Phase 3 (컴파일러 체인)
├── Week 1-4:  LTO 지원
├── Week 5-8:  PGO 지원
├── Week 9-10: 벡터화 진단
└── Week 11-12: 통합 테스트

2026 Q4: Phase 4 (고급 기능)
├── Week 1-4:  Sealed trait
├── Week 5-8:  FP 정밀도 타입
├── Week 9-10: 분기 힌트
└── Week 11-12: 최종 벤치마크
```

---

## 성공 기준

### Gate 정의

| Gate | 기준 | 측정 방법 |
|------|------|----------|
| **G1** | Bounds check 제거율 95%+ | ASM에서 cmp/jae 카운트 |
| **G2** | Overflow check 제거율 90%+ | 정제 타입 함수에서 |
| **G3** | 벡터화 성공률 95%+ | LLVM 리포트 |
| **G4** | 이론적 최적 대비 97%+ | 벤치마크 스위트 |
| **G5** | Rust 대비 동등 또는 우위 | 동일 알고리즘 비교 |

### 벤치마크 스위트

```
compute/
├── sum_array         # Bounds check 검증
├── matrix_multiply   # Aliasing + Vectorization
├── overflow_safe     # Overflow check 검증
└── mixed_precision   # FP 정밀도 검증

contract/
├── verified_sort     # Contract 오버헤드
├── refinement_math   # 정제 타입 성능
└── disjoint_compute  # disjoint 최적화

real-world/
├── json_parser       # 실제 워크로드
├── http_server       # I/O + CPU 혼합
└── compiler_self     # 자기 컴파일
```

---

## 리스크 및 완화

| 리스크 | 영향 | 완화 방안 |
|--------|------|----------|
| Fin[N] 타입 복잡성 | 사용성 저하 | 자동 추론 극대화 |
| SMT 증명 시간 | 빌드 시간 증가 | 증명 캐싱, 증분 검증 |
| LLVM 버전 의존성 | noalias 버그 | LLVM 버전 고정, 테스트 |
| 효과 시스템 복잡성 | 학습 곡선 | 자동 추론 기본, 명시적 선택 |

---

## 부록: 참조 언어

| 기능 | 참조 언어 | BMB 적용 |
|------|----------|---------|
| 의존 타입 | Idris, Agda | Fin[N], Vect[N, T] |
| 정제 타입 | F*, Liquid Haskell | `where` 절 확장 |
| 선형 타입 | Rust, Clean | Unique[T], 효과 시스템 |
| 효과 시스템 | Koka, Eff | `effects` 절 |
| Sealed | Kotlin, Scala 3 | `sealed` 키워드 |
