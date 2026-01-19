## 언어 X 목표 기준서: 이론적 최대 달성 수준

---

## Executive Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    언어 X 성능 목표                          │
├─────────────────────────────────────────────────────────────┤
│  이론적 최적 대비 목표: 97~99%                               │
│  (현재 Rust: 92~98%, C: 85~92%)                             │
├─────────────────────────────────────────────────────────────┤
│  제거 가능 비용:  5~8%  →  0~1%                              │
│  제거 불가 비용:  3~5%  →  3~5% (물리적/수학적 한계)          │
└─────────────────────────────────────────────────────────────┘
```

---

## 1. Bounds Checking

### 목표

| 항목 | 현재 (Rust) | 언어 X 목표 |
|-----|------------|------------|
| 내부 연산 | 0.5~3% | **0%** |
| 외부 입력 경계 | 1회 검사 | **1회 검사** (제거 불가) |
| 컴파일 타임 증명 | 단순 패턴만 | **모든 정적 패턴** |

### 요구 기능

```
1. 의존 타입 시스템
   - 길이가 타입에 인코딩: Vect n a
   - 인덱스 타입: Fin n (0 ≤ i < n 보장)

2. 자동 증명 생성
   - 단순 루프: for i in 0..arr.len()
   - 컴파일러가 자동으로 Fin 증명 생성

3. 증명 전파
   - 한번 검증된 인덱스는 함수 경계 넘어 전파
   - 재검사 없음

4. 폴백 메커니즘
   - 증명 불가 시 명시적 런타임 검사
   - 또는 컴파일 에러 (프로그래머 선택)
```

### 문법 예시

```
// 정적 길이 배열
let arr: Array[10, i32] = [...]
let i: Fin[10] = 5          // 컴파일 타임 검증
let x = arr[i]              // 검사 0

// 동적 인덱스
let user_input: Nat = read_input()
match user_input.to_fin(arr.len) {
    Some(i) => arr[i]       // 검사 0 (이미 검증됨)
    None    => default
}

// 루프 자동 추론
for i in arr.indices() {    // i: Fin[arr.len] 자동 추론
    sum += arr[i]           // 검사 0
}

// 슬라이스 분할
let (left, right) = arr.split_at(5)
// left: Array[5, i32], right: Array[arr.len - 5, i32]
// 타입 수준에서 비중첩 보장
```

### 달성 기준

```
✅ 통과 조건:
   - 정적 분석 가능한 모든 인덱싱에서 런타임 검사 0
   - 생성된 어셈블리에 cmp/jae 없음 (bounds check 흔적)
   - 외부 입력 경계에서만 검사 존재

❌ 실패 조건:
   - 단순 for 루프에서 bounds check 발견
   - 동일 인덱스 중복 검사
```

---

## 2. Integer Overflow

### 목표

| 항목 | 현재 (Rust) | 언어 X 목표 |
|-----|------------|------------|
| Debug 검사 | 10~100x 느림 | **0%** (타입 수준) |
| Release wrap | 검사 0 (안전성 희생) | **검사 0 + 안전** |
| 범위 추적 | 없음 | **완전 자동** |

### 요구 기능

```
1. 정제 타입 (Refinement Types)
   - 값 범위가 타입에 인코딩
   - int{0 ≤ x ≤ 100}

2. 자동 범위 추론
   - 산술 연산 결과 범위 자동 계산
   - a: int{0..100} + b: int{0..100} → int{0..200}

3. 타입 승격 규칙
   - 범위 초과 가능 시 자동 승격
   - i32 * i32 → i64 (필요시)

4. 오버플로우 불가능 증명
   - 컴파일 타임에 증명되면 검사 제거
   - 증명 불가 시 컴파일 에러 또는 명시적 처리
```

### 문법 예시

```
// 범위 타입 선언
type Percentage = int{0..100}
type Byte = int{0..255}

// 자동 범위 추론
fn add_percentages(a: Percentage, b: Percentage) -> int{0..200} {
    a + b  // 오버플로우 불가능, 검사 0
}

// 범위 축소 (검사 필요)
fn to_byte(x: int{0..200}) -> Option[Byte] {
    if x <= 255 { Some(x as Byte) }  // 1회 검사
    else { None }
}

// 안전한 곱셈 (자동 승격)
fn multiply(a: i32, b: i32) -> i64 {
    a.widen() * b.widen()  // i64로 승격, 오버플로우 불가
}

// 명시적 처리 강제
fn risky_add(a: i32, b: i32) -> i32 {
    a + b  // 컴파일 에러: 오버플로우 가능성 명시 필요
}

fn risky_add_handled(a: i32, b: i32) -> Overflow | i32 {
    a +? b  // 오버플로우 시 Overflow 반환
}
```

### 달성 기준

```
✅ 통과 조건:
   - 범위 추적 가능한 모든 연산에서 런타임 검사 0
   - 오버플로우 가능성이 타입 에러로 보고
   - SIMD 벡터화 정상 작동

❌ 실패 조건:
   - -ftrapv 수준의 성능 저하
   - 범위 정보 손실로 불필요한 검사 삽입
```

---

## 3. Null Safety

### 목표

| 항목 | 현재 (Rust) | 언어 X 목표 |
|-----|------------|------------|
| Null 검사 | 0% (Option) | **0%** |
| 메모리 오버헤드 | 0 (참조) | **0** |
| FFI 안전성 | 수동 검사 | **경계 자동화** |

### 요구 기능

```
1. Null 불가능 참조 (기본)
   - 모든 참조는 유효함을 타입이 보장
   - &T는 절대 null 아님

2. 명시적 부재 (Option)
   - Maybe[T] = None | Some(T)
   - 패턴 매칭 강제

3. FFI 자동 래핑
   - 외부 함수 반환 포인터 자동 Option 변환
   - null 검사 1회 후 내부 전파
```

### 문법 예시

```
// 기본: null 불가능
let x: &i32 = &value     // 항상 유효
let y: &i32 = null       // 컴파일 에러

// 명시적 부재
let maybe: Maybe[&i32] = None

// 패턴 매칭 강제
match maybe {
    Some(ref) => use(ref)   // ref: &i32, null 아님
    None => handle_absence()
}

// FFI 자동 래핑
extern fn c_function() -> *mut i32

// 언어 X에서 자동 변환
let result: Maybe[&mut i32] = c_function().as_ref()
// null 검사 자동 삽입, 이후 검사 0
```

### 달성 기준

```
✅ 통과 조건:
   - Safe 코드에서 null dereference 불가능
   - Option<&T> 크기 = &T 크기
   - FFI 경계에서 자동 null 처리

❌ 실패 조건:
   - 런타임 null 검사 존재 (내부 코드)
   - 메모리 오버헤드 발생
```

---

## 4. Aliasing Analysis

### 목표

| 항목 | 현재 (Rust) | 언어 X 목표 |
|-----|------------|------------|
| 가변 참조 유일성 | 언어 보장 | **언어 보장** |
| LLVM noalias 활용 | ~90% | **100%** (폐쇄 시스템) |
| 벡터화 성공률 | ~70% | **95%+** |

### 요구 기능

```
1. 선형 타입 시스템
   - 유일 참조: Unique[T]
   - 공유 참조: Shared[T]
   - 이동 의미론 기본

2. 영역 기반 메모리 (Region-based)
   - 참조의 유효 범위 타입 수준 추적
   - 교차 참조 불가능 증명

3. 전체 프로그램 분석 (WPA)
   - 모든 포인터 관계 정적 분석
   - 별도 컴파일 단위 없음 (폐쇄 시스템)

4. 효과 시스템
   - 함수의 메모리 효과 타입에 명시
   - reads[a], writes[b], allocates[c]
```

### 문법 예시

```
// 유일 참조
fn process(data: Unique[Array[i32]]) {
    // data는 이 함수에서만 접근 가능
    // 컴파일러가 noalias 보장
}

// 영역 타입
fn with_region['r](
    a: &'r mut Array[i32],
    b: &'r Array[i32],
    c: &'r Array[i32]
) {
    // 'r 영역 내에서 a는 b, c와 겹치지 않음 보장
    for i in a.indices() {
        a[i] = b[i] + c[i]  // 벡터화 가능!
    }
}

// 효과 명시
fn mutate(arr: &mut Array[i32]) 
    effects { writes[arr] }
{
    // 이 함수는 arr만 수정함을 타입이 보장
}

// 순수 함수
fn pure_compute(x: i32, y: i32) -> i32
    effects { pure }
{
    x + y  // 부작용 없음, 자유롭게 재배치 가능
}
```

### 달성 기준

```
✅ 통과 조건:
   - 모든 &mut가 LLVM noalias로 컴파일
   - restrict 수준의 벡터화 달성
   - 불필요한 메모리 재로드 없음

❌ 실패 조건:
   - LLVM이 aliasing 가정으로 최적화 포기
   - 순수 함수에서 메모리 접근 발생
```

### 한계 명시

```
⚠️ 제거 불가능:
   - FFI 경계: 외부 코드의 aliasing 보장 불가
   - 포인터 산술: 런타임 결정 주소
   
해결책:
   - FFI는 unsafe 영역으로 격리
   - 포인터 산술 금지 또는 증명 요구
```

---

## 5. Virtual Dispatch

### 목표

| 항목 | 현재 (Rust) | 언어 X 목표 |
|-----|------------|------------|
| 정적 디스패치 | 기본 | **기본** |
| 동적 디스패치 | dyn Trait | **필요시만** |
| Devirtualization | LTO 의존 | **전체 프로그램** |

### 요구 기능

```
1. Monomorphization (기본)
   - 제네릭은 컴파일 타임 구체화
   - 각 타입별 특수화 코드 생성

2. 전체 프로그램 Devirtualization
   - 모든 동적 타입 분석
   - 단일 구현이면 직접 호출로 변환

3. Sealed 타입 계층
   - 확장 불가능 계층은 완전 정적화
   - enum처럼 모든 변형 열거 가능

4. Defunctionalization
   - 함수 객체를 데이터로 변환
   - 간접 호출 제거
```

### 문법 예시

```
// 정적 디스패치 (기본)
trait Drawable {
    fn draw(&self)
}

fn render[T: Drawable](obj: &T) {
    obj.draw()  // 직접 호출, 인라이닝 가능
}

// Sealed 계층
sealed trait Shape {
    fn area(&self) -> f64
}

struct Circle { radius: f64 }
struct Square { side: f64 }

impl Shape for Circle { ... }
impl Shape for Square { ... }
// Shape의 구현은 Circle, Square만 존재
// 컴파일러가 완전 열거 가능

fn process_shape(s: &dyn Shape) {
    s.area()  // 정적 디스패치로 변환 가능!
    // match s.type {
    //     Circle => circle_area()
    //     Square => square_area()
    // }
}

// 명시적 동적 (열린 확장)
open trait Plugin {
    fn execute(&self)
}
// 런타임 로딩 가능, vtable 사용
```

### 달성 기준

```
✅ 통과 조건:
   - 제네릭 함수에서 간접 호출 0
   - Sealed 계층에서 vtable 없음
   - 열린 계층만 동적 디스패치

❌ 실패 조건:
   - 불필요한 vtable 생성
   - 단일 구현에서 간접 호출
```

---

## 6. Exception Handling

### 목표

| 항목 | 현재 (Rust) | 언어 X 목표 |
|-----|------------|------------|
| 정상 경로 비용 | ~0% | **0%** |
| 에러 경로 비용 | 분기 | **분기** |
| 스택 되감기 | panic 시 | **없음** |

### 요구 기능

```
1. 대수적 효과 (Algebraic Effects)
   - 에러를 효과로 모델링
   - 컴파일 타임에 핸들러 결정

2. Result 기반 (기본)
   - 모든 에러는 반환값으로 처리
   - 스택 되감기 없음

3. 효과 타입 추론
   - 함수의 가능한 에러 자동 추론
   - 명시적 선언 불필요

4. 자동 전파
   - ? 연산자 수준의 편의성
   - 제로 오버헤드
```

### 문법 예시

```
// 대수적 효과
effect Error[E] {
    fn raise(e: E) -> Never
}

fn may_fail() -> i32 with Error[String] {
    if condition {
        raise("error")
    }
    42
}

fn handle_it() -> i32 {
    with handler {
        raise(e) => 0  // 에러 시 0 반환
    } in {
        may_fail()
    }
}
// 컴파일 타임에 핸들러 인라이닝
// 스택 되감기 없음

// Result 기반 (간단한 경우)
fn read_file(path: &str) -> Result[String, IoError] {
    let content = fs.read(path)?  // 자동 전파
    Ok(content)
}

// 효과 조합
fn complex() -> i32 
    with Error[ParseError], Error[IoError] 
{
    let data = read_file("input.txt")?
    let parsed = parse(data)?
    parsed.value
}
```

### 달성 기준

```
✅ 통과 조건:
   - 정상 경로에서 예외 관련 코드 0
   - 에러 전파가 단순 분기로 컴파일
   - .eh_frame 섹션 없음

❌ 실패 조건:
   - 스택 되감기 코드 존재
   - 예외 테이블 생성
```

---

## 7. Floating Point Precision

### 목표

| 항목 | 현재 (Rust) | 언어 X 목표 |
|-----|------------|------------|
| IEEE 754 준수 | 기본 | **타입별 선택** |
| fast-math | nightly만 | **타입 수준** |
| 영역별 제어 | 없음 | **완전 지원** |

### 요구 기능

```
1. 정밀도 타입 시스템
   - IEEE754[f32]: 엄격한 준수
   - Fast[f32]: 재배열 허용
   - Approx[f32]: 근사 허용

2. 정밀도 전파
   - 혼합 연산 시 더 엄격한 쪽으로
   - 명시적 변환 필요

3. 영역별 정밀도
   - 블록 수준에서 정밀도 선택
   - 함수 수준 속성
```

### 문법 예시

```
// 타입 수준 정밀도
type Precise = IEEE754[f64]
type Fast = Relaxed[f64]

fn scientific_compute(x: Precise, y: Precise) -> Precise {
    x + y  // IEEE 754 준수, 재배열 금지
}

fn graphics_compute(x: Fast, y: Fast) -> Fast {
    x + y  // 재배열 허용, FMA 자동 생성, SIMD 최적화
}

// 변환
let precise: Precise = 1.0
let fast: Fast = precise.relax()  // 명시적 변환
let back: Precise = fast.strict() // 명시적 복귀

// 블록 수준 정밀도
fn mixed() -> f64 {
    let a = strict {
        // IEEE 754 준수 영역
        kahan_sum(data)
    }
    
    let b = relaxed {
        // fast-math 영역
        approximate_sqrt(x)
    }
    
    a + b  // 기본 정밀도로 복귀
}

// 함수 속성
@precision(relaxed)
fn fast_function(x: f64) -> f64 {
    // 전체 함수에 fast-math 적용
}
```

### 달성 기준

```
✅ 통과 조건:
   - Relaxed 타입에서 -ffast-math 수준 최적화
   - IEEE754 타입에서 엄격한 준수
   - 혼합 시 컴파일 에러 또는 명시적 변환

❌ 실패 조건:
   - 정밀도 요구사항 무시
   - 전역 fast-math만 지원
```

---

## 8. Branch Prediction

### 목표

| 항목 | 현재 (Rust) | 언어 X 목표 |
|-----|------------|------------|
| 힌트 제공 | nightly | **안정적 지원** |
| 자동 추론 | 제한적 | **PGO 통합** |
| Branchless 변환 | 수동 | **자동 제안** |

### 요구 기능 (본질적 한계 내에서)

```
1. 분기 힌트 (안정적)
   - likely, unlikely 키워드
   - 컴파일러 코드 배치 최적화

2. PGO 통합
   - 프로파일 데이터 자동 수집
   - 재컴파일 시 자동 적용

3. Branchless 자동 변환
   - 조건부 이동 자동 선택
   - 프로그래머에게 제안

4. 컴파일 타임 분기 극대화
   - constexpr if 확장
   - 타입 수준 조건
```

### 문법 예시

```
// 분기 힌트
if unlikely(error_condition) {
    handle_error()
}

if likely(normal_case) {
    fast_path()
}

// 컴파일 타임 분기
fn process[T]() {
    if const (T == i32) {
        // i32 특수화 (런타임 분기 0)
    } else if const (T == f64) {
        // f64 특수화
    }
}

// Branchless 힌트
let max = branchless { if a > b { a } else { b } }
// 컴파일러가 CMOV 또는 조건부 연산 사용

// PGO 자동 통합
@profile
fn hot_function() {
    // 프로파일 데이터 자동 수집
    // 재컴파일 시 분기 예측 최적화
}
```

### 달성 기준

```
✅ 통과 조건:
   - 힌트가 코드 배치에 반영
   - PGO로 핫 패스 최적화
   - Branchless 변환 제안

⚠️ 본질적 한계:
   - 런타임 데이터 의존 분기는 예측 불가
   - CPU 예측기에 의존 (하드웨어 한계)
```

---

## 9. Register Allocation / Instruction Scheduling

### 목표

| 항목 | 현재 (LLVM) | 언어 X 목표 |
|-----|------------|------------|
| 할당 품질 | 휴리스틱 ~95% | **휴리스틱 ~97%** |
| 작은 함수 | 최적 가능 | **최적 보장** |
| 힌트 제공 | 없음 | **레지스터 클래스** |

### 요구 기능 (NP-Hard 한계 내에서)

```
1. 개선된 휴리스틱
   - ML 기반 레지스터 할당
   - 더 나은 스필 결정

2. 작은 함수 최적 보장
   - 함수 크기 < 50 명령어
   - ILP/전수탐색으로 최적해

3. 레지스터 힌트
   - 프로그래머가 레지스터 클래스 지정
   - 핫 변수 우선 할당

4. Superoptimizer 통합
   - 크리티컬 루프 자동 최적화
   - 빌드 시간 투자 옵션
```

### 문법 예시

```
// 레지스터 힌트
fn hot_loop() {
    @register_hint(priority: high)
    let accumulator = 0
    
    for item in data {
        accumulator += item
    }
}

// 크리티컬 섹션 최적화 요청
@superoptimize
fn critical_kernel(a: &[f32], b: &[f32]) -> f32 {
    // 빌드 시간 투자해서 최적 코드 탐색
}

// 인라인 어셈블리 (최후 수단)
fn manual_optimization() {
    asm {
        "vfmadd231ps {a}, {b}, {c}"
        a = inout(ymm) result,
        b = in(ymm) x,
        c = in(ymm) y,
    }
}
```

### 달성 기준

```
✅ 통과 조건:
   - 휴리스틱 품질 향상 (측정 가능)
   - 작은 함수 최적해 보장
   - Superoptimizer 옵션 제공

⚠️ 본질적 한계:
   - 일반 함수에서 최적해 불가 (NP-Complete)
   - 휴리스틱 개선이 최선
```

---

## 10. Auto-Vectorization

### 목표

| 항목 | 현재 (LLVM) | 언어 X 목표 |
|-----|------------|------------|
| 자동 벡터화 성공률 | ~70% | **95%+** |
| 의존성 분석 | 제한적 | **프로그래머 증명** |
| 진단 | 컴파일러 리포트 | **IDE 통합** |

### 요구 기능

```
1. 의존성 증명 시스템
   - 루프 의존성 없음을 증명 제공
   - 컴파일러가 안전하게 벡터화

2. Polyhedral 모델 통합
   - 아핀 루프 자동 분석
   - 타일링, 교환 자동 적용

3. SIMD 타입 시스템
   - 명시적 벡터 타입
   - portable-simd 수준의 추상화

4. 실패 진단
   - 벡터화 실패 이유 명확히
   - 수정 제안
```

### 문법 예시

```
// 자동 벡터화 (힌트)
@vectorize
fn add_arrays(a: &mut [f32], b: &[f32], c: &[f32]) {
    for i in 0..a.len() {
        a[i] = b[i] + c[i]
    }
}
// 컴파일러가 &mut 보장으로 벡터화

// 의존성 증명 제공
@vectorize(independent: true)  // 프로그래머 보장
fn complex_loop(a: &mut [f32], indices: &[usize]) {
    for &i in indices {
        a[i] += 1.0
    }
}

// 명시적 SIMD
fn explicit_simd(a: &mut [f32], b: &[f32]) {
    for (a_chunk, b_chunk) in a.chunks_mut(8).zip(b.chunks(8)) {
        let va: f32x8 = f32x8::from_slice(a_chunk)
        let vb: f32x8 = f32x8::from_slice(b_chunk)
        (va + vb).write_to_slice(a_chunk)
    }
}

// Polyhedral 변환
@polyhedral
fn matrix_multiply(A: &Matrix, B: &Matrix, C: &mut Matrix) {
    for i in 0..N {
        for j in 0..N {
            for k in 0..N {
                C[i][j] += A[i][k] * B[k][j]
            }
        }
    }
}
// 컴파일러가 자동으로 타일링, 루프 교환 적용
```

### 달성 기준

```
✅ 통과 조건:
   - 단순 루프 100% 벡터화
   - 아핀 루프 95%+ 벡터화
   - 실패 시 명확한 이유 제공

⚠️ 본질적 한계:
   - 간접 참조 (a[b[i]]) 분석 제한
   - 복잡한 제어 흐름
```

---

## 11. 컴파일 모델

### 목표

```
전체 프로그램 컴파일 + 증분 빌드 조화
```

### 요구 기능

```
1. 전체 프로그램 분석 (WPA)
   - 모든 코드 동시 분석
   - 크로스 모듈 최적화

2. 증분 컴파일
   - 변경된 부분만 재분석
   - 의존성 그래프 활용

3. LTO 기본 적용
   - 링크 타임 최적화 항상 활성
   - ThinLTO로 빌드 시간 절충

4. 캐싱
   - 분석 결과 캐싱
   - 증명 결과 캐싱
```

### 달성 기준

```
✅ 통과 조건:
   - 전체 프로그램 최적화 품질
   - 개발 시 빠른 증분 빌드
   - 릴리스 빌드 최대 최적화
```

---

## 종합 목표 기준표

| 영역 | 현재 최선 | 언어 X 목표 | 달성 방법 | 한계 |
|-----|----------|------------|----------|------|
| **Bounds Check** | 0.5~3% | **0%** | 의존 타입 | 외부 입력 1회 |
| **Overflow Check** | Debug 느림 | **0%** | 정제 타입 | 범위 불확실 시 승격 |
| **Null Check** | 0% (Rust) | **0%** | ADT | ✅ 달성 |
| **Aliasing** | ~90% 활용 | **100%** | 선형 타입 + WPA | FFI 경계 |
| **Virtual Dispatch** | 정적 기본 | **100% 정적** | Sealed + Defunc | 열린 확장 |
| **Exception** | ~0% | **0%** | 대수적 효과 | ✅ 달성 가능 |
| **FP Precision** | 전역 선택 | **타입 수준** | 정밀도 타입 | ✅ 달성 가능 |
| **Branch Prediction** | 힌트 | **힌트 + PGO** | 통합 PGO | 런타임 한계 |
| **Register Alloc** | 휴리스틱 | **개선 휴리스틱** | ML + ILP | NP-Complete |
| **Instruction Sched** | 휴리스틱 | **개선 휴리스틱** | Superopt | NP-Hard |
| **Vectorization** | ~70% | **95%+** | 증명 + Polyhedral | 간접 참조 |

---

## 최종 성능 목표

```
┌─────────────────────────────────────────────────────────────┐
│                    언어 X 성능 분석                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  제거된 비용 (vs Rust):                                      │
│    - 안전성 검사:     0.5~3%  →  0%     (−2%)               │
│    - Aliasing 손실:   1~2%    →  0%     (−1.5%)             │
│    - 디스패치 손실:   ~0.5%   →  0%     (−0.5%)             │
│    - 벡터화 실패:     ~2%     →  ~0.5%  (−1.5%)             │
│                                                             │
│  제거 불가 비용:                                             │
│    - 레지스터/스케줄링: ~2~3% (NP-Hard)                       │
│    - 분기 예측 미스:    ~1~2% (물리적 한계)                   │
│    - 캐시 미스:         ~1~2% (하드웨어 한계)                 │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  이론적 최적 대비:                                           │
│    - Rust:    92~98%                                        │
│    - 언어 X:  97~99%                                        │
│    - 개선:    ~2~5%p                                        │
│                                                             │
│  절대 한계:                                                  │
│    - 최적 대비 ~1~3% 손실 불가피                             │
│    - NP-Hard 문제 + 물리적 한계                              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 구현 우선순위

```
Phase 1: 안전성 검사 제거 (영향: ~2%)
  ├── 의존 타입 시스템
  ├── 정제 타입 시스템
  └── 자동 증명 생성

Phase 2: Aliasing 최적화 (영향: ~1.5%)
  ├── 선형 타입 강화
  ├── 효과 시스템
  └── 전체 프로그램 분석

Phase 3: 컴파일러 품질 (영향: ~1.5%)
  ├── ML 기반 휴리스틱
  ├── Polyhedral 모델
  └── Superoptimizer 통합

Phase 4: 도구 체인 (영향: 간접적)
  ├── PGO 자동화
  ├── IDE 통합 진단
  └── 성능 프로파일러
```