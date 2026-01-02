---

# BMB Language Specification

**Version**: v0.10 (2026-01-03)
**Status**: Implementation in Progress

> This specification describes the BMB language as of v0.10 Sunrise.
> For detailed implementation roadmap, see [ROADMAP.md](ROADMAP.md).

---

## 1. 개요

BMB는 AI-Native 프로그래밍 언어이다.

- 계약 기반 정확성 보장
- 네이티브 최대 성능 (C/Rust 초월)
- AI 생성 최적화

---

## 2. 어휘 구조

### 2.1 키워드

```
fn        함수 정의
type      타입 정의
enum      합 타입 정의
struct    레코드 정의
pre       전제 조건
post      후제 조건
where     타입 제약
ret       반환값 참조
self      자기 참조
old       이전 값 참조
if        조건
then      조건 결과
else      조건 대안
match     패턴 매칭
let       불변 바인딩
var       가변 바인딩
mut       가변 참조
rec       재귀
for       반복
in        범위/포함
forall    전칭 양화
exists    존재 양화
low       저수준 블록
pure      순수 함수
decreases 종료 증명
invariant 루프 불변식
modifies  프레임 조건
aligned   메모리 정렬
disjoint  비중첩 증명
own       소유권
ref       참조
drop      해제
move      이동
copy      복사
async     비동기
await     대기
spawn     생성
chan      채널
send      송신
recv      수신
try       시도
extern    외부 연동
trust     신뢰 (검증 생략)
check     런타임 검사
contract  계약 정의
satisfies 계약 만족
```

### 2.2 기호

```
:         타입 선언
->        함수 반환 / 함축
=>        람다 / 분기 결과
=         정의 / 바인딩
?         Option 타입
!         Result 타입
|         분기 구분 / 합 타입
,         나열
;         문장 끝
< >       비교 / 제네릭
( )       그룹 / 호출
[ ]       배열 / 인덱싱
{ }       블록 / 레코드
&         참조
@         어노테이션
<:        계약 포함 (서브타입)
```

### 2.3 연산자

```
산술:     + - * / %
비교:     == != < > <= >=
논리:     and or not
비트:     band bor bxor bnot shl shr
```

---

## 3. 메모리 모델

### 3.1 소유권

```bmb
let a = [1, 2, 3];
let b = a;            -- a 이동됨

let x: i32 = 1;
let y = x;            -- x 복사됨 (Copy 타입)

let c = copy(a);      -- 명시적 깊은 복사
```

#### 3.1.1 소유권 타입

| 타입 | 의미 | 제약 |
|------|------|------|
| `T` | 소유 | 이동 후 무효 |
| `own T` | 명시적 소유 | 힙 할당, drop 필요 |
| `&T` | 불변 참조 | 여러 개 가능 |
| `&mut T` | 가변 참조 | 하나만 가능 |
| `*T` | 불변 포인터 | low 블록에서만 역참조 |
| `*mut T` | 가변 포인터 | low 블록에서만 역참조 |

### 3.2 참조

```bmb
let r: &[i32] = &arr;         -- 불변 참조
let m: &mut [i32] = &mut arr; -- 가변 참조
```

#### 3.2.1 빌림 규칙

컴파일러가 검증하는 빌림 규칙:

```
규칙 1: &T 여러 개 OR &mut T 하나 (동시 불가)
규칙 2: 참조는 원본보다 오래 살 수 없음
규칙 3: &mut T 존재 시 &T 생성 불가
```

```bmb
let arr = [1, 2, 3];
let r1 = &arr;        -- OK: 불변 참조
let r2 = &arr;        -- OK: 불변 참조 여러 개
let m = &mut arr;     -- ERROR: 불변 참조 존재 중

let arr2 = [1, 2, 3];
let m1 = &mut arr2;   -- OK: 가변 참조
let m2 = &mut arr2;   -- ERROR: 가변 참조 하나만
```

#### 3.2.2 에일리어싱 계약

```bmb
fn swap<T>(a: &mut T, b: &mut T) -> ()
  pre distinct(a, b)       -- 비중첩 증명
  post *a == old(*b)
  post *b == old(*a)
= { let tmp = *a; *a = *b; *b = tmp };

-- distinct 위반 시 컴파일 에러
let x = 5;
swap(&mut x, &mut x);  -- ERROR: distinct(a, b) 위반
```

### 3.3 수명

```bmb
fn first<T>(arr: &[T]) -> &T
  pre len(arr) > 0
= &arr[0];

fn longest<'a, T>(a: &'a [T], b: &'a [T]) -> &'a [T]
= if len(a) > len(b) then a else b;
```

#### 3.3.1 수명 표기

| 표기 | 의미 |
|------|------|
| `'a` | 명시적 수명 |
| `'static` | 프로그램 전체 수명 |
| `'_` | 추론된 수명 |

```bmb
fn select<'a, 'b, T>(x: &'a T, y: &'b T, first: bool) -> &'a T
  pre first              -- 'b 수명은 결과에 무관
= x;
```

### 3.4 할당

```bmb
let x: i32 = 1;                    -- 스택
let arr: [i32; 10] = [0; 10];      -- 스택
let vec: own [i32] = alloc([1,2,3]); -- 힙
```

### 3.5 리전 (v0.106.3)

계층적 메모리 영역:

```bmb
region 'outer {
  let a = alloc(100);

  region 'inner {
    let b = alloc(50);
    -- b는 'inner 종료 시 해제
  }

  -- a는 'outer 종료 시 해제
}
```

#### 3.5.1 리전 규칙

```
규칙 1: 내부 리전 참조는 외부로 탈출 불가
규칙 2: 리전 종료 시 모든 할당 해제
규칙 3: 리전 간 참조는 수명으로 검증
```

### 3.6 선형 타입 (v0.106.4)

정확히 한 번 사용해야 하는 값:

```bmb
let handle: linear File = open("data.txt");
-- handle 미사용 시 컴파일 에러
-- handle 중복 사용 시 컴파일 에러
close(handle);  -- OK: 정확히 한 번 사용
```

#### 3.6.1 선형 타입 규칙

| 규칙 | 설명 |
|------|------|
| 필수 사용 | 선형 값은 반드시 소비 |
| 단일 사용 | 두 번 이상 사용 불가 |
| 이동 전용 | 복사 불가 |

```bmb
fn transfer(src: linear Resource, dst: &mut Container) -> ()
  post src 소비됨
= dst.store(src);

-- 사용 예
let r: linear Resource = acquire();
transfer(r, &mut container);
-- r 재사용 불가
```

### 3.7 메모리 안전성 검증 (v0.106.5)

컴파일러는 세 가지 검사를 통합 수행:

```
┌─────────────────────────────────────┐
│     MemorySafetyContext             │
├─────────────────────────────────────┤
│  BorrowContext   (빌림 검사)        │
│  RegionContext   (리전 검사)        │
│  LinearContext   (선형 검사)        │
└─────────────────────────────────────┘
```

모든 검사 통과 시 메모리 안전성 보장.

---

## 4. 타입 시스템

### 4.1 기본 타입

```
i8 i16 i32 i64 i128     부호 있는 정수
u8 u16 u32 u64 u128     부호 없는 정수
f32 f64                 부동소수점
bool                    불린
char                    유니코드 문자
()                      유닛
never                   반환 안 함
```

### 4.2 복합 타입

```bmb
[T]                     슬라이스
[T; N]                  고정 배열
own [T]                 소유 배열
?T                      Option<T>
T ! E                   Result<T, E>
(T, U)                  튜플
T -> U                  함수
&T                      불변 참조
&mut T                  가변 참조
own T                   소유권
```

### 4.3 합 타입 (enum)

```bmb
enum Option<T> {
  Some(T),
  None
}

enum Result<T, E> {
  Ok(T),
  Err(E)
}

enum Shape {
  Circle(f64),
  Rect(f64, f64),
  Point
}

enum Ordering {
  Lt,
  Eq,
  Gt
}
```

### 4.4 레코드 (struct)

```bmb
struct Point {
  x: f64,
  y: f64
}

struct Error {
  message: String,
  code: i32,
  source: ?own Error
}

struct Person {
  name: String,
  age: u32
}
```

### 4.5 정제 타입

```bmb
type NonZero = i32
  where self != 0;

type Positive = i32
  where self > 0;

type Bounded(lo: i32, hi: i32) = i32
  where lo <= self and self <= hi;

type Probability = f64
  where 0.0 <= self and self <= 1.0;
```

### 4.6 계약 타입

```bmb
type Sorted<T: Ord> = [T]
  where forall(i in 0..len(self)-1): self[i] <= self[i+1];

type NonEmpty<T> = [T]
  where len(self) > 0;

type Unique<T: Eq> = [T]
  where forall(i in 0..len(self), j in 0..len(self)): 
    i != j -> self[i] != self[j];
```

### 4.7 계약 포함 (<:)

```bmb
-- 자동 추론
Positive <: NonZero        -- self > 0 -> self != 0
NonEmpty<T> <: [T]         -- 항상 성립
Sorted<T> <: [T]           -- 항상 성립

-- 사용
fn first<T>(arr: NonEmpty<[T]>) -> T = arr[0];

let sorted: Sorted<[i32]> = sort([3,1,2]);
let x = first(sorted);  -- Sorted <: NonEmpty 면 허용, 아니면 거부

-- 컴파일러가 Sorted <: NonEmpty 증명 시도
-- 실패 시: NonEmpty 요구하는 곳에 Sorted 사용 불가
```

---

## 5. 계약 시스템

### 5.1 기본 구조

```bmb
fn name(params) -> ReturnType
  pre precondition
  post postcondition
= implementation;
```

### 5.2 old() 의미론

```bmb
fn increment(x: &mut i32) -> ()
  post *x == old(*x) + 1
= *x = *x + 1;

fn sort(arr: &mut [i32]) -> ()
  post sorted(arr)
  post perm(old(arr), arr)
= quicksort(arr);
```

**old(expr) 규칙:**

| 규칙 | 설명 |
|------|------|
| 평가 시점 | 함수 진입 시 |
| 적용 대상 | &mut 파라미터, 전역 상태 |
| 복사 방식 | 논리적 스냅샷 (SMT 레벨) |
| 중첩 금지 | old(old(x)) 불허 |
| post 전용 | pre에서 사용 불가 |

**SMT 변환:**

```bmb
fn increment(x: &mut i32) -> ()
  post *x == old(*x) + 1
= *x = *x + 1;
```

```smt
(define-fun increment ((x_old Int) (x_new Int)) Bool
  (= x_new (+ x_old 1)))

(assert (forall ((x_old Int))
  (let ((x_new (+ x_old 1)))
    (increment x_old x_new))))
```

### 5.3 forall/exists 문법

```bmb
-- 단일 변수
forall(i in 0..n): P(i)
exists(i in 0..n): P(i)

-- 다중 변수
forall(i in 0..n, j in 0..m): P(i, j)
exists(i in 0..n, j in 0..m): P(i, j)

-- 조건부
forall(i in 0..n): i > 0 -> P(i)
exists(i in 0..n): i > 0 and P(i)

-- 컬렉션
forall(x in arr): P(x)
exists(x in arr): P(x)
```

**EBNF:**

```ebnf
quantifier := ('forall' | 'exists') '(' bindings ')' ':' expr ;
bindings   := binding (',' binding)* ;
binding    := IDENT 'in' range ;
range      := expr '..' expr | expr ;
```

### 5.4 Frame Conditions (modifies)

```bmb
fn swap(a: &mut i32, b: &mut i32) -> ()
  modifies a, b
  post *a == old(*b)
  post *b == old(*a)
= let tmp = *a; *a = *b; *b = tmp;

fn increment_first(arr: &mut [i32]) -> ()
  modifies arr[0]
  post arr[0] == old(arr[0]) + 1
  post forall(i in 1..len(arr)): arr[i] == old(arr[i])
= arr[0] = arr[0] + 1;
```

**modifies 규칙:**

| 규칙 | 설명 |
|------|------|
| 명시적 선언 | 변경되는 메모리 위치 나열 |
| 프레임 추론 | 선언되지 않은 위치는 변경 불가 |
| 배열 인덱싱 | `modifies arr[i]` 또는 `modifies arr[0..n]` |
| 필드 접근 | `modifies obj.field` |
| 전체 참조 | `modifies *ptr` |

**암시적 프레임:**
- `modifies` 없으면 아무것도 변경하지 않음 (순수)
- `modifies a` 선언 시, `b`는 자동으로 `old(b) == b` 보장

**SMT 변환:**

```smt
; modifies a, b implies frame condition:
; forall other locations: old(loc) == loc
(assert (= c_new c_old))  ; c not in modifies → unchanged
```

### 5.5 계약 표현식 제약

계약 내 표현식은 **순수**해야 함:

| 허용 | 금지 |
|------|------|
| 산술/논리 연산 | 부작용 함수 호출 |
| 순수 함수 호출 | I/O |
| 필드 접근 | 할당 |
| 인덱싱 | 변경 |
| forall/exists | 무한 루프 |
| old() (post에서) | old() (pre에서) |

### 5.6 계약 정의 및 합성

```bmb
-- 계약 정의
contract Sortable<T: Ord> for fn(&mut [T]) -> () {
  post sorted(self)
  post perm(old(self), self)
}

-- 계약 사용
fn quicksort<T: Ord>(arr: &mut [T]) -> ()
  satisfies Sortable<T>
= ...;

fn mergesort<T: Ord>(arr: &mut [T]) -> ()
  satisfies Sortable<T>
= ...;

-- 계약 합성
contract StableSortable<T: Ord> for fn(&mut [T]) -> () {
  satisfies Sortable<T>
  post stable(old(self), self)  -- 추가 조건
}
```

### 5.7 암시적 바인딩 규칙

```bmb
type Sorted<T: Ord> = [T]
  where forall(i in 0..len(self)-1): self[i] <= self[i+1];
```

| 암시적 | 의미 |
|--------|------|
| `self` | 정의 중인 타입의 값 |
| `len(self)` | self의 길이 (배열) |
| `ret` | 함수 반환값 (post에서) |

**명시 규칙:**

```bmb
-- 암시적 (허용)
type Sorted<T> = [T]
  where forall(i in 0..len(self)-1): self[i] <= self[i+1];

-- 명시적 (권장 복잡한 경우)
type Sorted<T> = arr: [T]
  where forall(i in 0..len(arr)-1): arr[i] <= arr[i+1];
```

---

## 6. 계약 기반 최적화

### 6.1 최적화 매핑

| 계약 | 최적화 | 조건 |
|------|--------|------|
| `pre i >= 0 and i < len(arr)` | 경계 검사 제거 | 인덱싱 시 |
| `pre some(opt)` | None 검사 제거 | unwrap 시 |
| `pre ok(result)` | Err 검사 제거 | unwrap 시 |
| `pre b != 0` | 0 나눗셈 검사 제거 | 나눗셈 시 |
| `pre disjoint(a, b)` | restrict 최적화 | 포인터 연산 |
| `pure` | CSE, 메모이제이션, 재배치 | 전역 |
| `Sorted<[T]>` | 이진 검색 선택 | 검색 시 |
| `NonEmpty<[T]>` | 빈 배열 검사 제거 | first/last |
| `decreases` | 재귀 인라인 힌트 | 종료 증명됨 |
| `post ret >= 0` | 부호 검사 제거 | 후속 사용 |
| `aligned(N)` | SIMD 정렬 로드 | 벡터화 |

### 6.2 예시: 경계 검사 제거

```bmb
fn sum(arr: &[i32]) -> i32 = {
  var total = 0;
  for i in 0..len(arr) {
    total = total + arr[i];  -- 경계 검사 제거됨
    -- 이유: i in 0..len(arr) 이므로 i < len(arr) 증명됨
  }
  total
};
```

### 6.3 예시: SIMD 활성화

```bmb
fn dot(a: &[f32] aligned(32), b: &[f32] aligned(32)) -> f32
  pre len(a) == len(b)
  pre disjoint(a, b)
= low {
    -- pre 조건으로 컴파일러가 알 수 있는 것:
    -- 1. aligned(32): AVX 정렬 로드 가능
    -- 2. len(a) == len(b): 길이 검사 불필요
    -- 3. disjoint(a, b): 벡터화 안전
    
    var sum = f32x8.zero();
    for i in 0..len(a) step 8 {
      let va = f32x8.load(a, i);   -- 정렬 로드
      let vb = f32x8.load(b, i);   -- 정렬 로드
      sum = f32x8.fma(va, vb, sum);
    }
    f32x8.hsum(sum)
  };
```

### 6.4 예시: 순수성 활용

```bmb
fn square(x: i32) -> i32
  pure
  post ret == x * x
= x * x;

fn sum_squares(arr: &[i32]) -> i32 = {
  var total = 0;
  for i in 0..len(arr) {
    total = total + square(arr[i]);
    -- 컴파일러 최적화 가능:
    -- 1. square 인라인
    -- 2. 루프 벡터화
    -- 3. 결과 재사용 (같은 값이면)
  }
  total
};
```

### 6.5 예시: 계약 전파

```bmb
fn binary_search(arr: &Sorted<[i32]>, target: i32) -> ?usize
  post match ret {
    Some(i) -> arr[i] == target,
    None -> forall(x in arr): x != target
  }
= ...;

fn contains(arr: &Sorted<[i32]>, target: i32) -> bool
= some(binary_search(arr, target));
-- 컴파일러: Sorted 타입이므로 O(log n) 검색 선택
-- [i32] 였다면 O(n) 선형 검색

fn index_of(arr: &[i32], target: i32) -> ?usize
= ...;  -- O(n) 선형 검색
```

---

## 7. 에러 처리

### 7.1 에러 처리 철학

| 접근법 | 용도 |
|--------|------|
| `pre` 위반 | 프로그래머 오류 → panic |
| `Result<T, E>` | 복구 가능한 실패 |
| `Option<T>` | 값의 부재 |
| `panic!` | 복구 불가능 → 종료 |

### 7.2 Result 타입

```bmb
enum Result<T, E> {
  Ok(T),
  Err(E)
}

type T ! E = Result<T, E>;
```

### 7.3 기본 연산

```bmb
fn ok<T, E>(v: T) -> T ! E = Ok(v);
fn err<T, E>(e: E) -> T ! E = Err(e);
fn is_ok<T, E>(r: &(T ! E)) -> bool;
fn is_err<T, E>(r: &(T ! E)) -> bool;

fn unwrap<T, E>(r: T ! E) -> T
  pre is_ok(&r);

fn unwrap_err<T, E>(r: T ! E) -> E
  pre is_err(&r);
```

### 7.4 전파 연산자 `?`

```bmb
fn read_config(path: &str) -> Config ! Error = {
  let content = read_file(path)?;       -- 실패 시 Err 반환
  let parsed = parse_json(content)?;    -- 실패 시 Err 반환
  let config = validate(parsed)?;       -- 실패 시 Err 반환
  Ok(config)
};
```

`try` 키워드와 동등:

```bmb
let x = try expr;   -- 명시적
let x = expr?;      -- 간결
```

### 7.5 @errorset

타입 안전 에러 그룹 정의:

```bmb
@errorset ConfigError
  @error FILE_NOT_FOUND = 1
  @error PARSE_FAILED = 2
  @error VALIDATION_FAILED = 3
  @error PERMISSION_DENIED = 4

fn read_config(path: &str) -> Config ! ConfigError = {
  let content = read_file(path)
    .map_err(_ => ConfigError.FILE_NOT_FOUND)?;
  let parsed = parse_json(content)
    .map_err(_ => ConfigError.PARSE_FAILED)?;
  Ok(validate(parsed)?)
};
```

#### 7.5.1 @errorset 특징

| 특징 | 설명 |
|------|------|
| 타입 안전 | 정의된 에러만 사용 가능 |
| 정수 코드 | FFI 호환 |
| 완전성 | match 시 모든 케이스 필수 |

### 7.6 에러 계약

```bmb
fn divide(a: i32, b: i32) -> i32 ! DivError
  post b != 0 ==> is_ok(ret)
  post b == 0 ==> ret == Err(DivError.DIVIDE_BY_ZERO)
  post is_ok(ret) ==> unwrap(ret) * b == a
= if b == 0 then err(DivError.DIVIDE_BY_ZERO) else ok(a / b);

-- pre vs Result 선택
fn safe_divide(a: i32, b: NonZero) -> i32
  post ret * b == a
= a / b;    -- b가 NonZero이므로 항상 성공
```

---

## 8. 함수

### 8.1 기본 구조

```bmb
fn name(params) -> ReturnType
  pre preconditions
  post postconditions
= implementation;
```

### 8.2 검증 수준

```bmb
-- 완전 검증 (기본)
fn abs(x: i32) -> i32
  post ret >= 0
= if x >= 0 then x else -x;

-- 신뢰 (검증 생략)
fn fast_abs(x: i32) -> i32
  post ret >= 0
  @trust
= if x >= 0 then x else -x;

-- 런타임 검사
fn checked_abs(x: i32) -> i32
  post ret >= 0
  @check
= if x >= 0 then x else -x;
```

### 8.3 검증 결과 처리

| 결과 | 의미 | 기본 동작 |
|------|------|-----------|
| Verified | 증명 완료 | 컴파일 진행 |
| Failed(반례) | 반례 발견 | 컴파일 거부 |
| Timeout | 시간 초과 | 컴파일 거부 |
| Unknown | 결정 불가 | 컴파일 거부 |

**어노테이션으로 동작 변경:**

```bmb
@trust    -- Timeout/Unknown 시 프로그래머 보장
@check    -- Timeout/Unknown 시 런타임 검사 삽입
@skip     -- Timeout/Unknown 시 경고만, 컴파일 진행
```

---

## 9. 동시성

### 9.1 스레드

```bmb
fn spawn<T: Send>(f: fn() -> T) -> Handle<T>;
fn join<T>(h: Handle<T>) -> T;
```

### 9.2 채널

```bmb
fn channel<T: Send>() -> (Sender<T>, Receiver<T>);
fn send<T>(tx: &Sender<T>, v: T) -> () ! ChanError;
fn recv<T>(rx: &Receiver<T>) -> T ! ChanError;
```

### 9.3 비동기

```bmb
fn fetch(url: &str) -> async String ! IoError;

fn process() -> async () ! IoError = {
  let data = await fetch("http://example.com");
  print(&data)
};
```

---

## 10. 저수준 블록

### 10.1 SIMD 타입

```bmb
i8x16 i8x32 i8x64
i16x8 i16x16 i16x32
i32x4 i32x8 i32x16
i64x2 i64x4 i64x8
f32x4 f32x8 f32x16
f64x2 f64x4 f64x8
```

### 10.2 SIMD 연산

```bmb
-- 생성
fn V.zero<V: Simd>() -> V;
fn V.splat<V: Simd>(x: elem(V)) -> V;

-- 로드/스토어
fn V.load<V: Simd>(ptr: &[elem(V)], offset: usize) -> V
  pre offset + lanes(V) <= len(ptr);

fn V.store<V: Simd>(ptr: &mut [elem(V)], offset: usize, v: V) -> ()
  pre offset + lanes(V) <= len(ptr);

-- 산술
fn V.add<V: SimdNum>(a: V, b: V) -> V;
fn V.sub<V: SimdNum>(a: V, b: V) -> V;
fn V.mul<V: SimdNum>(a: V, b: V) -> V;
fn V.fma<V: SimdFloat>(a: V, b: V, c: V) -> V;

-- 수평
fn V.hsum<V: SimdNum>(v: V) -> elem(V);
fn V.hmin<V: SimdOrd>(v: V) -> elem(V);
fn V.hmax<V: SimdOrd>(v: V) -> elem(V);
```

### 10.3 타겟 조건

```bmb
fn sort(arr: &mut [i32]) -> ()
  post sorted(arr)
= match target {
    | x86_64 { avx512 } -> avx512_sort(arr)
    | x86_64 { avx2 } -> avx2_sort(arr)
    | aarch64 { neon } -> neon_sort(arr)
    | _ -> scalar_sort(arr)
  };
```

---

## 11. FFI

### 11.1 외부 함수

```bmb
extern "C" {
  fn malloc(size: usize) -> *mut u8;
  fn free(ptr: *mut u8) -> ();
  fn strlen(s: *const u8) -> usize;
}
```

### 11.2 안전 래퍼

```bmb
fn safe_strlen(s: &str) -> usize
  post ret <= len(s.as_bytes())
= low { strlen(s.as_ptr()) };
```

---

## 12. 모듈

### 12.1 정의

```bmb
mod math {
  pub fn abs(x: i32) -> i32 = ...;
  fn helper(x: i32) -> i32 = ...;
}
```

### 12.2 인터페이스

```bmb
interface Sortable<T> {
  fn sort(xs: &mut [T]) -> ()
    satisfies contract {
      post sorted(xs)
      post perm(old(xs), xs)
    };
}
```

### 12.3 구현

```bmb
impl Sortable<i32> {
  fn sort(xs: &mut [i32]) -> () = radix_sort(xs);
}

impl<T: Ord> Sortable<T> {
  fn sort(xs: &mut [T]) -> () = merge_sort(xs);
}
```

---

## 13. 표준 라이브러리

### 13.1 Option

```bmb
enum Option<T> { Some(T), None }
type ?T = Option<T>;

fn some<T>(v: T) -> ?T = Some(v);
fn none<T>() -> ?T = None;
fn is_some<T>(o: &?T) -> bool;
fn is_none<T>(o: &?T) -> bool;
fn unwrap<T>(o: ?T) -> T pre is_some(&o);
fn map<T, U>(o: ?T, f: fn(T) -> U) -> ?U;
```

### 13.2 Result

```bmb
enum Result<T, E> { Ok(T), Err(E) }
type T ! E = Result<T, E>;

fn ok<T, E>(v: T) -> T ! E = Ok(v);
fn err<T, E>(e: E) -> T ! E = Err(e);
fn is_ok<T, E>(r: &(T ! E)) -> bool;
fn is_err<T, E>(r: &(T ! E)) -> bool;
fn unwrap<T, E>(r: T ! E) -> T pre is_ok(&r);
fn map<T, U, E>(r: T ! E, f: fn(T) -> U) -> U ! E;
```

### 13.3 배열

```bmb
fn len<T>(arr: &[T]) -> usize;
fn is_empty<T>(arr: &[T]) -> bool;
fn get<T>(arr: &[T], i: usize) -> ?&T;
fn first<T>(arr: &NonEmpty<[T]>) -> &T;
fn last<T>(arr: &NonEmpty<[T]>) -> &T;
```

### 13.4 고차 함수

```bmb
fn map<T, U>(arr: &[T], f: fn(&T) -> U) -> own [U]
  post len(ret) == len(arr);

fn filter<T>(arr: &[T], p: fn(&T) -> bool) -> own [T]
  post forall(x in ret): p(&x);

fn fold<T, U>(arr: &[T], init: U, f: fn(U, &T) -> U) -> U;
```

---

## 14. 예시

### 14.1 이진 검색

```bmb
fn binary_search<T: Ord>(arr: &Sorted<[T]>, target: &T) -> ?usize
  post match ret {
    Some(i) -> arr[i] == *target,
    None -> forall(x in arr): x != *target
  }
  decreases len(arr)
= {
    rec search(lo: usize, hi: usize) -> ?usize =
      if lo >= hi then none()
      else {
        let mid = lo + (hi - lo) / 2;
        match compare(&arr[mid], target)
          | Lt -> search(mid + 1, hi)
          | Gt -> search(lo, mid)
          | Eq -> some(mid)
      };
    search(0, len(arr))
  };
```

### 14.2 퀵소트

```bmb
contract Sortable<T: Ord> for fn(&mut [T]) -> () {
  post sorted(self)
  post perm(old(self), self)
}

fn quicksort<T: Ord>(arr: &mut [T]) -> ()
  satisfies Sortable<T>
= if len(arr) <= 1 then ()
  else {
    let pivot_idx = partition(arr);
    quicksort(&mut arr[0..pivot_idx]);
    quicksort(&mut arr[pivot_idx+1..]);
  };
```

### 14.3 벡터 덧셈 (SIMD)

```bmb
fn vadd(a: &[f32] aligned(32), b: &[f32] aligned(32)) -> own [f32]
  pre len(a) == len(b)
  pre disjoint(a, b)
  post len(ret) == len(a)
  post forall(i in 0..len(ret)): ret[i] == a[i] + b[i]
= low {
    let n = len(a);
    let mut result = alloc_aligned::<f32>(n, 32);
    
    for i in 0..n step 8 {
      let va = f32x8.load(a, i);
      let vb = f32x8.load(b, i);
      f32x8.store(&mut result, i, f32x8.add(va, vb));
    }
    
    result
  };
```

---

## 15. 문법 (EBNF)

```ebnf
program      := definition* ;

definition   := fn_def 
              | type_def 
              | enum_def
              | struct_def
              | contract_def
              | mod_def 
              | impl_def 
              | use_stmt 
              | extern_def ;

fn_def       := 'pub'? 'fn' IDENT generics? '(' params ')' '->' type
                contract_clause*
                annotation*
                '=' expr ';' ;

type_def     := 'pub'? 'type' IDENT generics? '=' type
                ('where' condition)? ';' ;

enum_def     := 'pub'? 'enum' IDENT generics? '{'
                variant (',' variant)* ','?
                '}' ;

variant      := IDENT ('(' type (',' type)* ')')? ;

struct_def   := 'pub'? 'struct' IDENT generics? '{'
                field (',' field)* ','?
                '}' ;

field        := IDENT ':' type ;

contract_def := 'contract' IDENT generics? 'for' fn_sig '{'
                contract_clause*
                '}' ;

contract_clause := 'pre' condition
                 | 'post' condition
                 | 'pure'
                 | 'decreases' expr
                 | 'satisfies' IDENT generics? ;

annotation   := '@' IDENT ('(' args ')')? ;

expr         := literal
              | IDENT
              | expr binop expr
              | unop expr
              | 'if' expr 'then' expr 'else' expr
              | 'match' expr ('|' pattern '->' expr)+
              | '{' stmt* expr '}'
              | 'rec' IDENT '(' params ')' '->' type '=' expr
              | 'fn' '(' params ')' '=>' expr
              | 'for' IDENT 'in' expr block
              | 'low' block
              | 'try' expr
              | 'await' expr
              | expr '(' args ')'
              | expr '[' expr ']'
              | expr '.' IDENT
              | '&' 'mut'? expr
              | 'old' '(' expr ')' ;

condition    := expr
              | quantifier ;

quantifier   := ('forall' | 'exists') '(' bindings ')' ':' expr ;

bindings     := binding (',' binding)* ;

binding      := IDENT 'in' range ;

range        := expr '..' expr 
              | expr ;

type         := base_type
              | '[' type ']'
              | '[' type ';' expr ']'
              | '?' type
              | type '!' type
              | '(' type (',' type)* ')'
              | type '->' type
              | '&' 'mut'? type
              | 'own' type
              | IDENT generics? ;

generics     := '<' type_param (',' type_param)* '>' ;

type_param   := IDENT (':' constraint)?
              | LIFETIME ;

pattern      := IDENT
              | literal
              | '(' pattern (',' pattern)* ')'
              | IDENT '(' pattern* ')'
              | '_' ;
```

---

## 16. 부록: 계약 포함 규칙

### 자동 추론

```
Positive <: NonZero
  증명: self > 0 -> self != 0

Bounded(1, 100) <: Positive
  증명: 1 <= self and self <= 100 -> self > 0

Sorted<T> <: [T]
  증명: 항상 성립 (추가 제약)

NonEmpty<T> <: [T]
  증명: 항상 성립 (추가 제약)
```

### 사용 규칙

```bmb
fn f(x: NonZero) -> i32;

let p: Positive = 5;
f(p);  -- 허용: Positive <: NonZero

let n: i32 = 5;
f(n);  -- 거부: i32 <: NonZero 아님
```

---

## 17. 부록: 검증 반례 형식

```
[검증 실패]
함수: divide
파일: src/math.bmb:15:1
계약: pre b != 0

반례:
  a = 10
  b = 0

호출 위치: src/main.bmb:42:5
  let result = divide(x, y);
  
  이 시점 상태:
    x = 10
    y = 0  -- 여기서 위반

제안:
  1. 호출 전 y != 0 검사 추가
  2. divide를 ?i32 반환으로 변경
```

---

## 18. 부록: 성능 목표

### 18.1 목표 (vs C -O3)

| 벤치마크 | Gold | Silver |
|----------|------|--------|
| 배열 합계 | ≥100% | 90-95% |
| 이진 검색 | ≥100% | 85-90% |
| 퀵정렬 | ≥95% | 85-90% |
| 행렬 곱셈 | ≥80% | 70-75% |
| JSON 파싱 | ≥70% | 50-60% |

**핵심 원칙**: BMB ≥ C -O3 (모든 케이스), BMB > C -O3 (계약 활용 케이스)

### 18.2 계약 기반 최적화

| 최적화 | 조건 | 예상 개선 |
|--------|------|-----------|
| 경계 검사 제거 | Gold 증명 | 5-15% |
| 에일리어싱 분석 | distinct 계약 | 10-20% |
| SIMD 벡터화 | vectorizable 계약 | 2-8x |
| 연산 융합 | fusable 계약 | 30-50% |
| 순수성 활용 | pure 계약 | 5-10% |

```bmb
-- 계약 → 최적화 예
fn sum(arr: &[i32]) -> i32
  pre len(arr) < 10000        -- 경계 검사 제거
  pure                         -- 부작용 없음 → 캐싱
= arr.iter().fold(0, add);

-- 컴파일러: SIMD 벡터화 + 언롤링 적용
```

### 18.3 최적화 해제 조건

| 조건 | 결과 |
|------|------|
| pre 증명 성공 | 런타임 검사 제거 |
| pre 증명 실패 | 런타임 검사 유지 |
| pure 선언 | 메모이제이션 허용 |
| distinct 증명 | restrict 포인터 적용 |

---

## 끝