## 가정 재정의

```
전제 조건:
- 프로그래머는 무한한 수학적 능력을 가짐
- 증명 작성 시간/비용은 고려하지 않음
- 언어 학습 곡선은 무시
- 오직 "런타임 비용 0이 기술적으로 가능한가?"만 분석
```

---

## 영역별 재분석

### 1. Bounds Checking → ✅ 비용 0 가능

**의존 타입으로 완전 제거 가능**

```idris
-- 모든 인덱스가 컴파일 타임에 검증됨
safeHead : Vect (S n) a -> a  -- 길이 1 이상만 허용
safeHead (x :: xs) = x
-- 빈 벡터에 safeHead 호출 = 컴파일 에러

-- 동적 인덱스도 처리 가능
indexDyn : (v : Vect n a) -> (i : Nat) -> Maybe a
indexDyn v i = case decLT i n of
    Yes prf => Just (index (natToFin i prf) v)  -- 증명 있음 → 검사 없음
    No _    => Nothing                           -- 증명 불가 → 실패 반환
```

**단, 외부 입력의 경계**:
```
사용자 입력 i → 최초 1회 검증 필수 (제거 불가)
이후 내부 전파 → 타입 시스템이 추적 → 추가 검사 0
```

**결론**: 내부 연산은 비용 0, 외부 경계에서 최소 1회는 물리적으로 필요

---

### 2. Integer Overflow → ✅ 비용 0 가능

**정제 타입(Refinement Types)으로 해결**

```fstar
// F*: 범위가 타입에 인코딩
type nat_lt (n:nat) = x:nat{x < n}
type bounded_int (lo:int) (hi:int) = x:int{lo <= x && x <= hi}

// 오버플로우가 불가능함을 타입이 보장
let safe_add (x:bounded_int 0 1000) (y:bounded_int 0 1000) 
    : bounded_int 0 2000 = x + y
// 런타임 검사 0, 컴파일러가 범위 추적
```

**한계**: 범위를 알 수 없는 연산
```fstar
// 사용자 입력 두 개의 곱 → 범위 무한대 가능
// 해결책 1: 결과 타입을 BigInt로 승격
// 해결책 2: 오버플로우 시 명시적 처리 강제
let mul_or_fail (x y : int32) : Either Overflow int64 = ...
```

**결론**: ✅ 비용 0 가능 (단, 타입 시스템이 복잡해짐)

---

### 3. Null/Option Checking → ✅ 비용 0 가능 (이미 달성)

```rust
// Rust의 Option<T>는 이미 비용 0
// 컴파일러가 None 케이스를 강제 처리하게 함
// 런타임 null 검사 = 0
```

```haskell
-- Haskell Maybe도 동일
-- 패턴 매칭이 컴파일 타임에 완전성 검사
```

**결론**: ✅ 이미 해결됨

---

### 4. Aliasing Analysis → ⚠️ 부분적 가능

**선형 타입(Linear Types)으로 상당 부분 해결**

```rust
// Rust의 &mut T는 이미 선형성 보장
// 문제: LLVM이 이 정보를 완전히 활용 못함
```

**더 강력한 접근: Uniqueness Types**

```
Clean 언어의 Uniqueness Types:
- *World → 유일한 참조임을 타입이 보장
- 컴파일러가 in-place mutation 보장
- 런타임 aliasing 검사 = 0
```

**그러나 본질적 한계 존재**:

```c
// 이 함수의 호출자가 제공하는 포인터들의 관계는?
void external_lib_call(int* a, int* b);

// 해결 불가:
// - FFI 경계
// - 런타임에 결정되는 포인터 산술
// - void* 캐스팅
```

**결론**: ⚠️ 순수 함수형 + 폐쇄 시스템에서만 비용 0

---

### 5. 분기 예측 / 투기적 실행 → ❌ 불가능

```
if (runtime_condition) {    // CPU가 예측해야 함
    path_a();
} else {
    path_b();
}
```

**이론적 한계**:
```
- 조건이 런타임 데이터에 의존
- 컴파일 타임에 결정 불가능
- Halting Problem의 변형

증명:
  만약 모든 분기를 컴파일 타임에 결정할 수 있다면
  → 프로그램의 모든 실행 경로를 알 수 있음
  → 프로그램이 종료하는지 알 수 있음
  → Halting Problem 해결
  → 모순
```

**가능한 완화**:
```
1. 분기 없는 코드 (branchless programming)
   - select 명령어 사용
   - 비용: 항상 양쪽 계산

2. 컴파일 타임 분기 (constexpr if)
   - 런타임 조건에는 적용 불가
```

**결론**: ❌ 런타임 조건 분기는 물리적으로 제거 불가

---

### 6. 레지스터 할당 / 명령어 스케줄링 → ❌ 불가능

**NP-Complete/NP-Hard 문제**

```
최적 레지스터 할당 = Graph Coloring Problem
최적 명령어 스케줄링 = Job Shop Scheduling

다항 시간 해결 불가 (P ≠ NP 가정)
```

**현실적 접근**:
```
1. 휴리스틱으로 "충분히 좋은" 해 탐색
2. 작은 함수에서는 최적해 가능
3. 슈퍼옵티마이저 (STOKE): 확률적 탐색으로 최적해 근사
```

**결론**: ❌ 최적해는 불가능, 근사해만 가능

---

### 7. 자동 벡터화 → ⚠️ 부분적 가능

**의존성 분석의 한계**

```c
for (int i = 0; i < n; i++) {
    a[f(i)] = b[g(i)];  // f, g의 결과가 겹치는가?
}
```

**해결 가능한 경우**:
```
- f, g가 순수 함수이고 정적 분석 가능
- 의존 타입으로 비중첩 증명 제공
```

```idris
-- 프로그래머가 비중첩 증명 제공
vectorize : (f g : Nat -> Nat) 
         -> Disjoint f g n  -- 증명: f와 g의 결과가 겹치지 않음
         -> Vect n a -> Vect n a -> Vect n a
```

**해결 불가능한 경우**:
```c
// 런타임에만 알 수 있는 의존성
a[user_input_1] = b[user_input_2];
```

**결론**: ⚠️ 정적으로 분석 가능한 경우만 비용 0

---

### 8. 가상 함수 / 동적 디스패치 → ⚠️ 부분적 가능

**Whole Program Analysis + Defunctionalization**

```haskell
-- 모든 함수를 데이터로 변환
data Func = Add Int | Mul Int | Compose Func Func

apply : Func -> Int -> Int
apply (Add n) x = x + n
apply (Mul n) x = x * n
apply (Compose f g) x = apply f (apply g x)

-- 컴파일 타임에 모든 케이스 정적 디스패치로 변환 가능
```

**한계**:
```
- 열린 시스템 (플러그인, 동적 로딩) 불가
- 재귀적 타입에서 무한 확장 문제
- FFI 경계에서 불가
```

**결론**: ⚠️ 폐쇄형 전체 프로그램 분석 시에만 가능

---

## 종합: 이론적 최대 달성 가능 수준

| 영역 | 비용 0 가능? | 조건 |
|-----|------------|------|
| Bounds check | ✅ | 의존 타입 + 내부 연산 한정 |
| Overflow check | ✅ | 정제 타입 |
| Null check | ✅ | 이미 달성 (Option/Maybe) |
| Aliasing | ⚠️ | 순수 함수형 + 선형 타입 + 폐쇄 시스템 |
| 분기 예측 | ❌ | **물리적 불가능** |
| 레지스터 할당 | ❌ | **NP-Complete** |
| 명령어 스케줄링 | ❌ | **NP-Hard** |
| 자동 벡터화 | ⚠️ | 정적 분석 가능 루프 한정 |
| 동적 디스패치 | ⚠️ | 전체 프로그램 분석 + 폐쇄 시스템 |

---

## 이상적인 언어 X의 스펙

```
언어 X 설계:

1. 타입 시스템
   - 의존 타입 (Dependent Types)
   - 정제 타입 (Refinement Types)  
   - 선형/어파인 타입 (Linear/Affine Types)
   - 효과 시스템 (Effect System)

2. 컴파일 모델
   - 전체 프로그램 컴파일 (Whole Program)
   - Defunctionalization 기본 적용
   - 슈퍼옵티마이저 통합

3. 제약 조건
   - FFI 없음 (순수 폐쇄 시스템)
   - 동적 로딩 없음
   - 모든 재귀는 구조적 재귀로 제한
```

**이 언어로 달성 가능한 수준**:

```
이론적 최적 대비:

안전성 검사 오버헤드: ~0%      (vs 현재 0.5~3%)
Aliasing 오버헤드:    ~0%      (vs 현재 1~5%)
디스패치 오버헤드:    ~0%      (vs 현재 ~1%)
-----------------------------------------
컴파일러 휴리스틱:    ~2~5%    (vs 현재 3~8%)
분기 예측 미스:       ~1~3%    (vs 현재 1~3%)
-----------------------------------------
총합:                 ~3~8%    (vs 현재 5~15%)
```

---

## 결론

### 프로그래머 난이도를 무시하면:

| 질문 | 답변 |
|-----|------|
| **안전성 검사 비용 0** | ✅ 가능 |
| **Aliasing 비용 0** | ⚠️ 폐쇄 시스템에서만 가능 |
| **전체 비용 0** | ❌ 불가능 |

### 제거 불가능한 본질적 비용 (~3~5%)

```
1. NP-Hard 최적화 문제의 근사 해 사용  : ~2~3%
2. 런타임 조건 분기의 예측 미스        : ~1~2%
3. 캐시 미스 (하드웨어 한계)           : ~1~2%
```

### 최종 답변

> **프로그래머 난이도를 완전히 무시해도, 이론적 최적 대비 ~3~5% 손실은 컴퓨터 과학과 물리학의 근본적 한계로 인해 제거 불가능합니다.**

현재 기술 대비 개선 가능한 영역은 **안전성 검사 오버헤드(0.5~3% → 0%)** 정도이며, 이는 **Idris, F\*, ATS** 같은 언어가 이미 부분적으로 달성하고 있습니다. 다만 이 언어들이 주류가 되지 못하는 이유가 바로 "프로그래머 난이도"입니다.

======

## 런타임 비용 0을 위한 이론적 문제와 해결 방안 총정리

---

## 1. Bounds Checking (경계 검사)

### 문제 정의
```
배열/슬라이스 접근 시 인덱스가 유효 범위 내인지 검사
arr[i] → if (i >= len) panic!() → 분기 + 비교 연산
```

### 현재 비용
- **0.5~3%** 일반적 오버헤드
- 핫 루프에서 **최대 10%**

### 비용 발생 원인
```c
// 매 접근마다 검사
for (int i = 0; i < n; i++) {
    sum += arr[i];  // cmp + jae 명령어 삽입
}
```
- 컴파일러가 `i < n`과 `i < arr.len`의 관계를 증명 못함
- 루프 불변 조건 추출 실패
- 인덱스가 복잡한 표현식일 때 분석 포기

### 이론적 해결: 의존 타입 (Dependent Types)

```idris
-- 타입에 길이 정보 인코딩
data Vect : (n : Nat) -> Type -> Type where
    Nil  : Vect 0 a
    (::) : a -> Vect n a -> Vect (S n) a

-- 인덱스 타입이 범위를 보장
data Fin : Nat -> Type where
    FZ : Fin (S n)          -- 0은 항상 유효 (길이 1 이상)
    FS : Fin n -> Fin (S n) -- 후속 인덱스

-- 안전한 인덱싱: 런타임 검사 0
index : Fin n -> Vect n a -> a
index FZ     (x :: xs) = x
index (FS k) (x :: xs) = index k xs
-- 잘못된 인덱스 = 타입 에러 = 컴파일 불가
```

### 동적 인덱스 처리

```idris
-- 런타임 값을 Fin으로 변환 (1회 검사)
natToFin : (k : Nat) -> (n : Nat) -> Maybe (Fin n)
natToFin k n = case decLT k n of
    Yes prf => Just (mkFin k prf)   -- 증명 생성
    No  _   => Nothing

-- 이후 내부 전파는 검사 0
processAll : (input : Nat) -> Vect n a -> Maybe a
processAll input vec = do
    idx <- natToFin input n    -- 여기서 1회만 검사
    let result1 = index idx vec        -- 검사 0
    let result2 = transform idx vec    -- 검사 0 (idx 재사용)
    pure result2
```

### 루프에서의 해결

```idris
-- 루프 인덱스가 타입 수준에서 추적됨
mapVect : (a -> b) -> Vect n a -> Vect n b
mapVect f Nil       = Nil
mapVect f (x :: xs) = f x :: mapVect f xs
-- 구조적 재귀 → 인덱스 범위 자동 증명 → 검사 0

-- fold도 마찬가지
foldVect : (a -> b -> b) -> b -> Vect n a -> b
foldVect f acc Nil       = acc
foldVect f acc (x :: xs) = foldVect f (f x acc) xs
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 정적 인덱스 `arr[3]` | ✅ | 컴파일 타임 검증 |
| 루프 인덱스 `arr[i]` | ✅ | 구조적 재귀 / Fin 타입 |
| 계산된 인덱스 `arr[f(x)]` | ✅ | f의 치역 증명 제공 |
| 외부 입력 `arr[user_input]` | ⚠️ | 경계에서 1회 검사 필수 |

### 결론
```
✅ 비용 0 달성 가능
   - 내부 연산: 완전히 제거
   - 외부 경계: 최소 1회 (물리적 필수)
```

---

## 2. Integer Overflow (정수 오버플로우)

### 문제 정의
```
산술 연산 결과가 타입 범위 초과 시 검출/처리
a + b → if (a > MAX - b) trap → 조건 분기 삽입
```

### 현재 비용
- Debug 모드: **10~500배** 느림
- SIMD 벡터화 완전 차단

### 비용 발생 원인
```asm
; 오버플로우 검사가 포함된 덧셈
addl    %esi, %edi
jo      .overflow_handler   ; 오버플로우 시 점프
; → 벡터화 불가, 파이프라인 스톨
```

### 이론적 해결: 정제 타입 (Refinement Types)

```fstar
// F*: 값의 범위가 타입에 인코딩
type int_range (lo hi : int) = x:int{lo <= x /\ x <= hi}

// 덧셈 결과 범위가 자동 계산됨
val safe_add : int_range 0 100 -> int_range 0 100 -> int_range 0 200
let safe_add x y = x + y  // 오버플로우 불가능 → 검사 0

// 곱셈도 마찬가지
val safe_mul : int_range 0 1000 -> int_range 0 1000 -> int_range 0 1000000
let safe_mul x y = x * y  // i32 범위 내 → 검사 0
```

### 타입 수준 산술

```agda
-- Agda: 컴파일 타임 범위 연산
data InRange : (lo hi : ℕ) → Set where
  mkInRange : (x : ℕ) → lo ≤ x → x ≤ hi → InRange lo hi

-- 덧셈의 타입 서명이 범위를 추적
_+ᵣ_ : InRange lo₁ hi₁ → InRange lo₂ hi₂ → InRange (lo₁ + lo₂) (hi₁ + hi₂)

-- 사용 예
example : InRange 0 200
example = (mkInRange 50 ...) +ᵣ (mkInRange 30 ...)
-- 50 + 30 = 80, 0 ≤ 80 ≤ 200 자동 증명
```

### 동적 값 처리

```fstar
// 외부 입력 → 범위 타입으로 승격
val fromInt : x:int -> option (int_range 0 255)
let fromInt x = 
    if 0 <= x && x <= 255 
    then Some x   // 여기서 1회 검사
    else None

// 이후 연산은 검사 0
val process : int_range 0 255 -> int_range 0 255 -> int_range 0 510
let process a b = a + b  // 증명됨: 255 + 255 = 510 ≤ i32 max
```

### 비트 연산 활용

```fstar
// 비트마스크로 범위 제한 (검사 대신 연산)
type uint8 = x:int{0 <= x /\ x < 256}

val mask_to_uint8 : int -> uint8
let mask_to_uint8 x = x land 0xFF  // 항상 범위 내 → 검사 0
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 상수 연산 | ✅ | 컴파일 타임 계산 |
| 범위 추적 가능 | ✅ | 정제 타입 |
| 범위 불확실 | ⚠️ | 결과 타입 확장 (i32→i64) |
| 임의 입력 | ⚠️ | 경계 검사 1회 또는 BigInt |

### 결론
```
✅ 비용 0 달성 가능
   - 정적 범위 추적으로 대부분 제거
   - 타입 승격 (i32→i64→i128→BigInt)으로 우회
```

---

## 3. Null/None Checking

### 문제 정의
```
포인터/참조가 유효한 값을 가리키는지 검사
ptr->field → if (ptr == null) trap
```

### 현재 비용
- 명시적 검사: 분기 오버헤드
- 암시적 검사: 페이지 폴트 트랩 (더 비쌈)

### 이론적 해결: 대수적 데이터 타입 (ADT)

```haskell
-- null 자체가 불가능한 타입 시스템
data Maybe a = Nothing | Just a

-- 패턴 매칭 강제 → null 검사가 아닌 분기 선택
process :: Maybe Int -> Int
process Nothing  = 0        -- 명시적 처리
process (Just x) = x + 1    -- x는 절대 null 아님

-- Just 내부에서는 검사 0
transform :: Int -> Int
transform x = x * 2  -- null 가능성 자체가 없음
```

### Rust의 구현

```rust
// Option<T>의 메모리 레이아웃 최적화
enum Option<T> {
    None,
    Some(T),
}

// Option<&T>는 실제로 포인터 1개 크기
// None = 0x0, Some = 실제 주소
// → 메모리 오버헤드 0

// 패턴 매칭 후 내부는 검사 0
fn process(opt: Option<i32>) -> i32 {
    match opt {
        None => 0,
        Some(x) => {
            let a = x + 1;  // 검사 0
            let b = a * 2;  // 검사 0
            b
        }
    }
}
```

### Non-nullable 참조

```kotlin
// Kotlin의 접근
val x: String = "hello"   // null 불가
val y: String? = null     // null 가능

fun process(s: String) {  // 파라미터가 non-null
    s.length              // 검사 0
}
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 타입 시스템 내부 | ✅ | Option/Maybe ADT |
| FFI 경계 | ⚠️ | 경계에서 1회 변환 |
| 레거시 코드 | ❌ | 언어 수준 재설계 필요 |

### 결론
```
✅ 비용 0 이미 달성됨
   - Rust, Haskell, Kotlin 등에서 구현
   - 핵심: null을 타입으로 표현, 런타임 검사 불필요
```

---

## 4. Aliasing Analysis (앨리어싱 분석)

### 문제 정의
```
두 포인터가 같은 메모리를 가리키는지 판별
→ 판별 실패 시 최적화 포기
```

### 현재 비용
- 루프 벡터화 실패: **2~5x** 성능 손실
- 레지스터 캐싱 실패: 불필요한 메모리 로드
- 코드 이동 제한

### 비용 발생 원인
```c
void compute(int* a, int* b, int* c, int n) {
    for (int i = 0; i < n; i++) {
        a[i] = b[i] + c[i];  // b, c가 a와 겹칠 수 있음
    }
}
// 매 반복마다 b[i], c[i] 재로드 필요
// SIMD 적용 불가 (의존성 불확실)
```

### 이론적 한계: 결정 불가능성

```
정리: 두 포인터가 같은 메모리를 가리키는지는 일반적으로 결정 불가능

증명 (귀류법):
  가정: 모든 경우에 aliasing을 결정하는 알고리즘 A 존재
  
  구성:
    p = some_complex_function(input)
    q = another_complex_function(input)
    if (p == q) { ... }
  
  A가 p == q를 결정하려면:
  → some_complex_function의 결과를 알아야 함
  → 함수의 종료 여부를 알아야 함
  → Halting Problem
  → 모순
```

### 이론적 해결: 선형 타입 (Linear Types)

```rust
// Rust의 소유권: 하나의 가변 참조만 허용
fn compute(a: &mut [i32], b: &[i32], c: &[i32]) {
    // 타입 시스템이 보장:
    // - a는 b, c와 겹치지 않음 (빌림 규칙)
    // - b, c는 불변이므로 서로 겹쳐도 무관
    for i in 0..a.len() {
        a[i] = b[i] + c[i];  // SIMD 가능!
    }
}
```

### 더 강력한 해결: Uniqueness Types

```clean
// Clean 언어: 유일성 타입
:: *World  // *는 유일성 표시

// 유일한 참조만 존재 → 절대 aliasing 없음
writeFile :: *File -> String -> *File
writeFile file content = ...
// file은 이 함수에서만 접근 가능
```

### 순수 함수형 접근

```haskell
-- 순수 함수: 부작용 없음 → aliasing 무관
compute :: Vector Int -> Vector Int -> Vector Int
compute b c = V.zipWith (+) b c
-- 새 벡터 생성 → 원본과 절대 alias 아님

-- ST 모나드: 제한된 가변성
runST $ do
    arr <- newArray n 0
    -- arr은 이 블록에서만 존재
    -- 외부 aliasing 불가능
    forM_ [0..n-1] $ \i ->
        writeArray arr i (b!i + c!i)
    freeze arr
```

### 전체 프로그램 분석

```
Whole Program Compilation:
1. 모든 포인터 생성 지점 수집
2. 데이터플로우 분석으로 aliasing 관계 계산
3. 증명 가능한 경우 noalias 마킹

한계:
- 별도 컴파일 불가
- 동적 로딩 불가
- 분석 시간 O(n²) ~ O(n³)
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 순수 함수 | ✅ | 부작용 없음 → aliasing 무관 |
| 선형 타입 | ✅ | 유일 참조 보장 |
| 전체 프로그램 | ✅ | 정적 분석 (폐쇄 시스템) |
| FFI/동적 로딩 | ❌ | 분석 불가능 |
| 포인터 산술 | ❌ | 결정 불가능 |

### 결론
```
⚠️ 부분적 비용 0 가능
   - 순수 함수형 + 선형 타입 + 폐쇄 시스템에서 달성
   - 열린 시스템/FFI에서는 본질적 한계
```

---

## 5. Virtual Dispatch (가상 함수 호출)

### 문제 정의
```
런타임에 실제 호출할 함수 결정
obj->method() → vtable 조회 → 간접 점프
```

### 현재 비용
- 직접 호출 대비 **2~6배** 느림
- 인라이닝 불가
- 분기 예측 실패 가능

### 비용 발생 원인
```cpp
class Base { virtual void f() = 0; };
class A : Base { void f() override; };
class B : Base { void f() override; };

void call(Base* obj) {
    obj->f();  // 어떤 f인지 런타임에 결정
}
// mov rax, [obj]        ; vtable 로드
// mov rax, [rax + 8]    ; 함수 포인터 로드  
// call rax              ; 간접 호출
```

### 이론적 해결 1: Defunctionalization

```haskell
-- 고차 함수를 1차 데이터로 변환
-- 원본
map :: (a -> b) -> [a] -> [b]
map f []     = []
map f (x:xs) = f x : map f xs

-- Defunctionalized
data FuncRep a b where
    AddOne :: FuncRep Int Int
    Double :: FuncRep Int Int
    Custom :: (a -> b) -> FuncRep a b

apply :: FuncRep a b -> a -> b
apply AddOne x = x + 1
apply Double x = x * 2
apply (Custom f) x = f x

mapDefunc :: FuncRep a b -> [a] -> [b]
mapDefunc f []     = []
mapDefunc f (x:xs) = apply f x : mapDefunc f xs
-- 컴파일 타임에 어떤 apply인지 결정 가능
```

### 이론적 해결 2: 전체 프로그램 분석

```
Whole Program Devirtualization:

1. 모든 클래스 상속 관계 수집
2. 각 호출 지점에서 가능한 타입 분석
3. 단일 타입이면 직접 호출로 변환

class Shape { virtual draw(); }
class Circle : Shape { draw(); }
class Square : Shape { draw(); }

void render(Shape* s) {
    s->draw();  // Shape, Circle, Square 가능
}

void renderCircle(Circle* c) {
    c->draw();  // Circle만 가능 → 직접 호출!
}
```

### 이론적 해결 3: 정적 다형성

```cpp
// CRTP: Curiously Recurring Template Pattern
template<typename Derived>
class Base {
    void interface() {
        static_cast<Derived*>(this)->implementation();
    }
};

class Concrete : public Base<Concrete> {
    void implementation() { /* ... */ }
};
// 컴파일 타임에 타입 결정 → 인라이닝 가능
```

```rust
// Rust의 monomorphization
fn process<T: Drawable>(item: &T) {
    item.draw();  // T가 컴파일 타임에 결정
}
// process::<Circle>, process::<Square> 별도 생성
// 각각 직접 호출
```

### 타입 수준 다형성

```idris
-- 인터페이스가 타입 파라미터
interface Drawable a where
    draw : a -> IO ()

-- 구현
Drawable Circle where
    draw c = putStrLn "Circle"

Drawable Square where
    draw s = putStrLn "Square"

-- 사용: 컴파일 타임에 구체 타입 결정
render : Drawable a => a -> IO ()
render x = draw x  // a가 고정되면 직접 호출
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 제네릭/템플릿 | ✅ | Monomorphization |
| 폐쇄 상속 계층 | ✅ | 전체 프로그램 분석 |
| Sealed class | ✅ | 컴파일 타임 열거 |
| 열린 상속 | ❌ | 런타임 결정 필수 |
| 플러그인/동적 로딩 | ❌ | 컴파일 타임 정보 없음 |

### 결론
```
⚠️ 부분적 비용 0 가능
   - 폐쇄 시스템 + 전체 프로그램 분석 시 달성
   - 열린 확장성과 상충 (trade-off)
```

---

## 6. Exception Handling (예외 처리)

### 문제 정의
```
에러 발생 시 스택 되감기 (unwinding)
throw → catch까지 스택 프레임 순회 → 소멸자 호출
```

### 현재 비용
- 정상 경로: ~0% (zero-cost exception)
- 예외 발생 시: **100배** 이상 느림
- 바이너리 크기: +1~2% (.eh_frame)

### 비용 발생 원인
```cpp
void f() {
    SomeObject obj;  // 예외 시 소멸자 호출 필요
    may_throw();     // 예외 발생 가능
}
// .eh_frame에 unwind 정보 저장
// 예외 시: 테이블 검색 → 스택 되감기 → 소멸자 호출
```

### 이론적 해결: 대수적 효과 (Algebraic Effects)

```koka
// Koka: 효과가 타입에 인코딩
effect raise<a>
    ctl raise(x: a): b

fun may_fail(): raise<string> int
    if condition then raise("error")
    else 42

fun handle_it(): int
    with handler
        ctl raise(msg) -> 0  // 에러 시 0 반환
    may_fail()
// 컴파일 타임에 모든 효과 처리 결정
// 런타임 스택 되감기 불필요
```

### 이론적 해결: Result/Either 타입

```rust
// Rust: 예외 대신 Result
enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn may_fail() -> Result<i32, String> {
    if condition {
        Err("error".to_string())
    } else {
        Ok(42)
    }
}

fn caller() -> Result<i32, String> {
    let x = may_fail()?;  // 에러 전파 (점프 없음)
    Ok(x + 1)
}
// ? 연산자 = 단순 분기문
// 스택 되감기 없음
```

### 컴파일 타임 예외 분석

```
전체 프로그램 분석:
1. 모든 throw 지점 수집
2. 각 함수의 예외 가능성 계산
3. 예외 불가능 함수는 unwind 정보 제거

noexcept 전파:
f() noexcept → g() 호출 → g()도 noexcept여야 함
→ 컴파일러가 자동 추론
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| Result/Either | ✅ | 분기로 변환 |
| 대수적 효과 | ✅ | 컴파일 타임 효과 처리 |
| noexcept 함수 | ✅ | unwind 정보 제거 |
| 동적 예외 | ⚠️ | 발생 시 비용 불가피 |

### 결론
```
✅ 비용 0 달성 가능
   - Result/Either로 예외 모델링
   - 대수적 효과로 컴파일 타임 처리
   - 스택 되감기 자체를 제거
```

---

## 7. Floating Point Precision (부동소수점 정밀도)

### 문제 정의
```
IEEE 754 준수 시 연산 재배열/융합 제한
(a + b) + c ≠ a + (b + c)  // 부동소수점에서
```

### 현재 비용
- SIMD 벡터화 제한: **10~40%** 손실
- FMA 자동 생성 불가
- 루프 재배열 제한

### 비용 발생 원인
```c
// IEEE 754 준수
for (int i = 0; i < n; i++) {
    sum += arr[i];  // 순서 보장 필요
}
// 병렬화 불가 (결합 순서 변경 금지)

// -ffast-math 사용 시
// 4개씩 병렬 누적 → SIMD → 4배 빠름
```

### 이론적 해결: 정밀도 타입 시스템

```
// 정밀도가 타입에 인코딩
type Float<P> where P : Precision

trait Precision {
    const ALLOW_REASSOC: bool;
    const ALLOW_FMA: bool;
    const ALLOW_RECIPROCAL: bool;
}

struct IEEE754;    // 엄격한 IEEE 754
struct FastMath;   // 재배열 허용
struct Balanced;   // 선택적 최적화

impl Precision for IEEE754 {
    const ALLOW_REASSOC: bool = false;
    const ALLOW_FMA: bool = false;
    const ALLOW_RECIPROCAL: bool = false;
}

impl Precision for FastMath {
    const ALLOW_REASSOC: bool = true;
    const ALLOW_FMA: bool = true;
    const ALLOW_RECIPROCAL: bool = true;
}

fn sum<P: Precision>(arr: &[Float<P>]) -> Float<P> {
    // P에 따라 다른 코드 생성
}
```

### 효과 시스템 접근

```koka
// 정밀도를 효과로 모델링
effect precision
    fun get_mode(): PrecisionMode

fun compute(): precision float
    match get_mode()
        IEEE754  -> ieee_compute()
        FastMath -> fast_compute()

// 사용 시점에 모드 선택
with handler
    fun get_mode() -> FastMath
compute()  // fast_compute 선택
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 정밀도 불필요 영역 | ✅ | FastMath 타입 사용 |
| 정밀도 필수 영역 | ❌ | IEEE 754 준수 필수 |
| 혼합 사용 | ✅ | 타입 수준 분리 |

### 결론
```
✅ 비용 0 달성 가능 (정밀도 포기 시)
   - 타입 시스템으로 정밀도 요구사항 명시
   - 영역별 다른 최적화 적용
   - 선택은 프로그래머에게
```

---

## 8. Branch Prediction (분기 예측)

### 문제 정의
```
조건 분기의 방향을 CPU가 예측
예측 실패 시 파이프라인 플러시 (15~20 사이클 손실)
```

### 현재 비용
- 예측 가능 분기: ~0%
- 예측 불가 분기: **10~30%** 손실

### 비용 발생 원인
```c
for (int i = 0; i < n; i++) {
    if (data[i] > threshold) {  // 데이터 의존적
        sum += data[i];
    }
}
// data가 랜덤이면 예측률 ~50%
// 매 2회 중 1회 파이프라인 플러시
```

### 본질적 한계: 물리적 불가능

```
정리: 런타임 데이터 의존적 분기는 컴파일 타임에 결정 불가능

증명:
  if (f(user_input) > 0) { A } else { B }
  
  user_input은 런타임에만 알 수 있음
  → f(user_input)의 결과도 런타임에만 결정
  → 분기 방향은 런타임에만 결정
  → 컴파일러가 할 수 있는 것 없음
```

### 부분적 해결: Branchless Programming

```c
// 분기 버전
int max(int a, int b) {
    if (a > b) return a;
    else return b;
}

// Branchless 버전
int max_branchless(int a, int b) {
    int diff = a - b;
    int mask = diff >> 31;  // 음수면 0xFFFFFFFF
    return a - (diff & mask);
}
// 분기 없음 → 예측 실패 없음

// 조건부 이동 (CMOV)
int max_cmov(int a, int b) {
    return (a > b) ? a : b;
    // 컴파일러가 CMOV 명령어로 변환
}
```

### 컴파일 타임 분기

```cpp
// constexpr if: 컴파일 타임 결정
template<typename T>
auto process(T value) {
    if constexpr (std::is_integral_v<T>) {
        return value * 2;
    } else {
        return value + 0.5;
    }
}
// 런타임 분기 0

// 타입 수준 조건
template<bool Cond, typename Then, typename Else>
using If = std::conditional_t<Cond, Then, Else>;
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 컴파일 타임 상수 | ✅ | constexpr if |
| 타입 기반 분기 | ✅ | 템플릿 특수화 |
| Branchless 가능 | ✅ | 조건부 연산으로 변환 |
| 데이터 의존적 | ❌ | **물리적 불가능** |

### 결론
```
❌ 비용 0 불가능 (런타임 데이터 의존 분기)
   - 분기 자체를 제거하거나 (branchless)
   - 컴파일 타임으로 이동하거나 (constexpr)
   - 런타임 분기는 본질적으로 예측 의존
```

---

## 9. Register Allocation (레지스터 할당)

### 문제 정의
```
변수를 CPU 레지스터에 배치
레지스터 부족 시 메모리 스필 발생
```

### 현재 비용
- 최적 대비: **2~5%** 손실 (휴리스틱 사용)

### 본질적 한계: NP-Complete

```
정리: 최적 레지스터 할당 = Graph Coloring = NP-Complete

환원:
  변수 = 노드
  동시 활성 = 간선
  레지스터 수 = 색상 수
  
  k-coloring 문제와 동치
  → NP-Complete
  → 다항 시간 최적해 불가능 (P ≠ NP 가정)
```

### 부분적 해결: 제한된 범위에서 최적해

```
작은 함수 (변수 < 50개):
  - ILP (Integer Linear Programming)로 최적해 계산 가능
  - 시간: O(2^n) but n이 작으면 실용적

SSA 형태:
  - Chordal graph로 변환 가능
  - Chordal graph coloring = O(n)
  - 최적해 가능!
```

### Superoptimizer 접근

```
STOKE (Stanford):
  - 확률적 탐색으로 최적 코드 시퀀스 발견
  - 작은 함수에서 인간/컴파일러보다 나은 결과
  - 시간: 수 분 ~ 수 시간

한계:
  - 큰 함수에서 탐색 공간 폭발
  - 실용적 범위: ~20 명령어
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 작은 함수 | ✅ | ILP/전수 탐색 |
| SSA 형태 | ✅ | Chordal coloring |
| 일반 함수 | ❌ | **NP-Complete** |

### 결론
```
❌ 일반적으로 비용 0 불가능
   - NP-Complete 문제
   - 작은 범위에서만 최적해 가능
   - 휴리스틱이 "충분히 좋은" 해 제공
```

---

## 10. Instruction Scheduling (명령어 스케줄링)

### 문제 정의
```
명령어 순서 최적화 (파이프라인 활용 극대화)
의존성 유지하면서 지연 최소화
```

### 현재 비용
- 최적 대비: **1~3%** 손실

### 본질적 한계: NP-Hard

```
정리: 최적 명령어 스케줄링 = Job Shop Scheduling = NP-Hard

환원:
  명령어 = 작업
  실행 유닛 = 기계
  의존성 = 선행 제약
  
  → NP-Hard
  → 다항 시간 최적해 불가능
```

### 현실적 접근

```
List Scheduling (휴리스틱):
  1. 의존성 그래프 구성
  2. 준비된 명령어 중 우선순위 높은 것 선택
  3. 반복
  
  복잡도: O(n²)
  품질: 최적의 ~95~98%
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 작은 기본 블록 | ✅ | 전수 탐색 |
| 일반 코드 | ❌ | **NP-Hard** |

### 결론
```
❌ 일반적으로 비용 0 불가능
   - NP-Hard 문제
   - 휴리스틱이 실용적
```

---

## 11. Auto-Vectorization (자동 벡터화)

### 문제 정의
```
스칼라 루프를 SIMD 명령어로 변환
4/8/16개 요소 동시 처리
```

### 현재 비용
- 벡터화 실패 시: **2~8배** 성능 손실

### 비용 발생 원인
```c
// 벡터화 실패 케이스
for (int i = 0; i < n; i++) {
    a[i] = a[i-1] + b[i];  // 루프 전달 의존성
}
// i번째 결과가 i-1에 의존 → 병렬화 불가
```

### 이론적 한계

```
루프 의존성 분석: 결정 불가능한 경우 존재

a[f(i)] = b[g(i)]

f, g가 복잡한 함수면:
  - f(i) == f(j) for some i ≠ j 인지 결정 불가
  - 보수적으로 의존성 가정 → 벡터화 포기
```

### 이론적 해결: 의존성 증명 제공

```idris
-- 프로그래머가 비의존성 증명 제공
vectorizable : (f : Nat -> Nat) 
            -> Injective f            -- f가 단사 함수임을 증명
            -> (arr : Vect n a)
            -> Vect n a
-- 증명이 있으면 컴파일러가 안전하게 벡터화
```

### Polyhedral Model

```
다면체 모델:
  - 루프를 수학적 다면체로 표현
  - 의존성을 벡터 공간에서 분석
  - 자동으로 타일링/벡터화 변환 도출

한계:
  - 아핀 인덱스만 처리 가능 (a[i*2+1])
  - 비선형 인덱스 불가 (a[i*i])
  - 간접 참조 불가 (a[b[i]])
```

### 해결 가능 범위

| 케이스 | 비용 0 가능? | 방법 |
|-------|------------|------|
| 단순 루프 | ✅ | 자동 분석 |
| 아핀 인덱스 | ✅ | Polyhedral model |
| 의존성 증명 가능 | ✅ | 프로그래머 증명 |
| 간접 참조 | ⚠️ | 런타임 검사 또는 불가 |
| 복잡한 제어 흐름 | ❌ | 분석 한계 |

### 결론
```
⚠️ 부분적 비용 0 가능
   - 단순/아핀 루프: 자동 벡터화
   - 복잡한 경우: 프로그래머 증명 또는 수동 SIMD
```

---

## 종합 요약표

| 영역 | 비용 0 가능? | 필요 조건 | 본질적 한계 |
|-----|------------|----------|-----------|
| **Bounds Check** | ✅ | 의존 타입 | 외부 입력 1회 검사 |
| **Overflow Check** | ✅ | 정제 타입 | 범위 불확실 시 타입 승격 |
| **Null Check** | ✅ | ADT (Option) | 이미 달성 |
| **Aliasing** | ⚠️ | 선형 타입 + 폐쇄 시스템 | FFI, 포인터 산술 |
| **Virtual Dispatch** | ⚠️ | 전체 프로그램 분석 | 열린 확장성 |
| **Exception** | ✅ | Result + 대수적 효과 | - |
| **FP Precision** | ✅ | 타입 수준 정밀도 | 정밀도 필요 시 비용 존재 |
| **Branch Prediction** | ❌ | - | **물리적 불가능** |
| **Register Alloc** | ❌ | - | **NP-Complete** |
| **Instruction Sched** | ❌ | - | **NP-Hard** |
| **Auto-Vectorization** | ⚠️ | 의존성 증명 | 복잡한 제어 흐름 |

---

## 이론적 최소 비용

```
프로그래머 난이도 무시 + 이상적 언어 X 가정:

제거 가능한 비용:
  - 안전성 검사: 0.5~3%  → 0%
  - Aliasing:    1~5%    → 0% (폐쇄 시스템)
  - 디스패치:    ~1%     → 0% (폐쇄 시스템)
  - 예외:        ~0%     → 0% (Result 사용)
  
제거 불가능한 비용:
  - 레지스터/스케줄링: ~2~3% (NP-Hard)
  - 분기 예측:         ~1~2% (물리적 한계)
  - 캐시 미스:         ~1~2% (하드웨어 한계)
  
========================================
이론적 최소 손실: ~3~7%
========================================
```
