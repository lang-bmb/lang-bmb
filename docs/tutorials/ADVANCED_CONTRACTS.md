# Advanced Contract Programming in BMB

> 고급 계약 프로그래밍 기법: `@trust`, 복합 조건, 정제 타입, Z3 활용

---

## Prerequisites

이 가이드를 읽기 전에 다음을 이해하고 있어야 합니다:
- 기본 `pre`/`post` 조건 (see: `CONTRACT_PROGRAMMING.md`)
- BMB 함수 정의 문법
- 기본 타입 시스템

---

## 1. @trust 어노테이션

### 개념

`@trust`는 정적 검증을 우회하고 런타임에만 검사하도록 지정합니다.
성능상 이유로 Z3 검증이 불가능하거나 외부 코드와 연동할 때 사용합니다.

### 사용 시나리오

```bmb
// 1. 외부 라이브러리 호출 (검증 불가)
@trust
fn read_file(path: String) -> String
  post ret.len() >= 0
= __builtin_read_file(path);

// 2. 성능 크리티컬 코드 (검증 시간 초과)
@trust
fn fast_sort(arr: Vec<i64>) -> Vec<i64>
  post ret.len() == arr.len()
= __intrinsic_sort(arr);

// 3. 수학적으로 증명되었지만 Z3가 처리 못하는 경우
@trust
fn gcd(a: i64, b: i64) -> i64
  pre a > 0 && b > 0
  post ret > 0 && a % ret == 0 && b % ret == 0
= if b == 0 { a } else { gcd(b, a % b) };
```

### @trust 사용 가이드라인

| 사용 OK | 사용 금지 |
|---------|----------|
| 외부 FFI 호출 | 비즈니스 로직 핵심 |
| 성능 최적화 핫스팟 | 보안 관련 코드 |
| 수학적 증명 완료 | 새로 작성한 알고리즘 |

### @trust 감사 (Audit)

```bash
# @trust 사용 현황 검색
bmb q fn --has-trust

# @trust 함수 목록 출력
grep -r "@trust" src/ --include="*.bmb"
```

---

## 2. 복합 Pre/Post 조건

### 다중 조건 결합

```bmb
fn binary_search(arr: Vec<i64>, target: i64) -> i64
  // 여러 사전 조건
  pre arr.len() > 0                    // 빈 배열 불가
  pre is_sorted(arr)                   // 정렬된 배열만
  pre target >= arr[0]                 // 범위 내
  pre target <= arr[arr.len() - 1]
  // 여러 사후 조건
  post ret >= -1                       // -1 또는 유효 인덱스
  post ret < arr.len() as i64
  post ret == -1 || arr[ret as usize] == target
= binary_search_impl(arr, target, 0, arr.len() - 1);
```

### 조건부 사후 조건

```bmb
fn safe_divide(a: i64, b: i64) -> i64?
  pre true  // 모든 입력 허용
  // 조건부 결과
  post b == 0 implies ret == None
  post b != 0 implies ret == Some(a / b)
= if b == 0 { None } else { Some(a / b) };
```

### 관계형 조건

```bmb
fn clamp(value: i64, min: i64, max: i64) -> i64
  pre min <= max
  // 결과가 범위 내에 있음을 보장
  post ret >= min
  post ret <= max
  // 원본 값과의 관계
  post value < min implies ret == min
  post value > max implies ret == max
  post value >= min && value <= max implies ret == value
= if value < min { min }
  else if value > max { max }
  else { value };
```

---

## 3. 정제 타입 (Refinement Types)

### 기본 정제 타입

```bmb
// 양수 타입
type Positive = i64 where self > 0;

// 퍼센트 (0-100)
type Percent = i64 where self >= 0 && self <= 100;

// 비어있지 않은 문자열
type NonEmptyString = String where self.len() > 0;

// 유효한 포트 번호
type Port = i64 where self >= 1 && self <= 65535;
```

### 정제 타입 사용

```bmb
fn calculate_discount(price: Positive, discount: Percent) -> Positive
  post ret <= price
= price - (price * discount / 100);

fn listen(port: Port) -> bool = {
    // port는 자동으로 1-65535 범위 보장
    __builtin_listen(port)
};
```

### 복합 정제 타입

```bmb
// 정렬된 배열
type SortedVec = Vec<i64> where is_sorted(self);

// 크기 제한 배열
type BoundedVec<const N: usize> = Vec<i64> where self.len() <= N;

// 유효한 이메일 (간소화)
type Email = String where
    self.len() > 0 &&
    contains(self, "@") &&
    contains(self, ".");
```

---

## 4. 불변식 (Invariants)

### 루프 불변식

```bmb
fn sum_array(arr: Vec<i64>) -> i64
  post ret == arr.iter().sum()  // 전체 합계
= {
    let mut total = 0;
    let mut i = 0;

    while i < arr.len()
      invariant total == arr[0..i].iter().sum()  // 부분 합계
      invariant i <= arr.len()
    {
        total = total + arr[i];
        i = i + 1;
    }

    total
};
```

### 구조체 불변식

```bmb
struct BankAccount {
    balance: i64,
    min_balance: i64,

    invariant balance >= min_balance
}

impl BankAccount {
    fn withdraw(self, amount: i64) -> BankAccount
      pre amount > 0
      pre self.balance - amount >= self.min_balance
      post ret.balance == self.balance - amount
    = BankAccount {
        balance: self.balance - amount,
        min_balance: self.min_balance
    };
}
```

---

## 5. 재귀 함수 종료 증명

### 감소 함수 (Decreasing Function)

```bmb
fn factorial(n: i64) -> i64
  pre n >= 0
  post ret >= 1
  decreases n  // 종료 증명: n이 매 호출마다 감소
= if n <= 1 { 1 } else { n * factorial(n - 1) };
```

### 복합 종료 조건

```bmb
fn ackermann(m: i64, n: i64) -> i64
  pre m >= 0 && n >= 0
  decreases (m, n)  // 사전순 감소
= if m == 0 { n + 1 }
  else if n == 0 { ackermann(m - 1, 1) }
  else { ackermann(m - 1, ackermann(m, n - 1)) };
```

---

## 6. Z3 검증기 활용

### 검증 실행

```bash
# 기본 검증
bmb verify file.bmb

# 상세 출력
bmb verify file.bmb --verbose

# 타임아웃 설정 (밀리초)
bmb verify file.bmb --timeout 30000

# 특정 함수만 검증
bmb verify file.bmb --function calculate_discount
```

### 검증 결과 해석

```
Verifying: calculate_discount
  Pre-condition: ✓ Satisfiable
  Post-condition: ✓ Proved

Verifying: complex_algorithm
  Pre-condition: ✓ Satisfiable
  Post-condition: ✗ Unknown (timeout after 30000ms)

  Hint: Consider adding @trust or simplifying conditions
```

### 검증 실패 디버깅

```bmb
// 문제가 있는 함수
fn problematic(x: i64) -> i64
  pre x > 0
  post ret > x  // Z3가 증명 실패할 수 있음
= x + 1;

// 해결 1: 조건 명확화
fn fixed_v1(x: i64) -> i64
  pre x > 0
  pre x < i64::MAX  // 오버플로우 방지
  post ret == x + 1
  post ret > x
= x + 1;

// 해결 2: @check로 런타임 검증
@check
fn fixed_v2(x: i64) -> i64
  pre x > 0
  post ret > x
= x + 1;
```

---

## 7. 성능 고려사항

### 검증 시간 최적화

```bmb
// 느린 버전: 복잡한 조건
fn slow_verify(arr: Vec<i64>) -> i64
  post forall i in 0..arr.len(): ret >= arr[i]  // O(n) 검증
= find_max(arr);

// 빠른 버전: 단순화된 조건
fn fast_verify(arr: Vec<i64>) -> i64
  post ret == arr.iter().max()  // 단일 비교
= find_max(arr);
```

### 계약 비용 분석

| 조건 유형 | 정적 검증 시간 | 런타임 비용 |
|-----------|---------------|-------------|
| 단순 비교 | ~1ms | 무시 가능 |
| 배열 순회 | ~100ms | O(n) |
| 중첩 루프 | ~10s | O(n²) |
| 재귀 조건 | 타임아웃 가능 | 가변 |

### 프로덕션 최적화

```bmb
// 개발 중: 전체 검증
#[cfg(debug)]
fn process(data: Vec<i64>) -> Vec<i64>
  pre is_valid(data)
  post is_sorted(ret)
= process_impl(data);

// 프로덕션: 핵심만 검증
#[cfg(release)]
@trust
fn process(data: Vec<i64>) -> Vec<i64>
  pre data.len() > 0  // 핵심 조건만
= process_impl(data);
```

---

## 8. 일반적인 패턴

### Null-Safety 패턴

```bmb
fn safe_get<T>(arr: Vec<T>, idx: i64) -> T?
  pre idx >= 0
  post idx >= arr.len() as i64 implies ret == None
  post idx < arr.len() as i64 implies ret == Some(arr[idx as usize])
= if idx < arr.len() as i64 { Some(arr[idx as usize]) } else { None };
```

### State Machine 패턴

```bmb
enum ConnectionState { Disconnected, Connecting, Connected }

fn connect(state: ConnectionState) -> ConnectionState
  pre state == ConnectionState::Disconnected
  post ret == ConnectionState::Connecting
= ConnectionState::Connecting;

fn on_connected(state: ConnectionState) -> ConnectionState
  pre state == ConnectionState::Connecting
  post ret == ConnectionState::Connected
= ConnectionState::Connected;
```

### Builder 패턴

```bmb
struct RequestBuilder {
    url: String?,
    method: String?,
    body: String?,

    invariant url != None implies url.unwrap().len() > 0
}

impl RequestBuilder {
    fn with_url(self, url: String) -> RequestBuilder
      pre url.len() > 0
      post ret.url == Some(url)
    = RequestBuilder { url: Some(url), ..self };

    fn build(self) -> Request
      pre self.url != None
      pre self.method != None
    = Request {
        url: self.url.unwrap(),
        method: self.method.unwrap(),
        body: self.body
    };
}
```

---

## 9. 트러블슈팅

### 일반적인 오류

#### 1. "Pre-condition unsatisfiable"
```
Error: Pre-condition for 'divide' is unsatisfiable
```
**원인**: 사전 조건이 모순됨
**해결**: 조건 검토, 모순되는 조건 제거

#### 2. "Post-condition not provable"
```
Error: Cannot prove post-condition for 'complex_fn'
```
**원인**: Z3가 증명 실패
**해결**: 조건 단순화, `@trust` 사용, 보조 조건 추가

#### 3. "Verification timeout"
```
Warning: Verification timeout for 'recursive_fn' (30000ms)
```
**원인**: 복잡한 재귀 또는 루프
**해결**: `--timeout` 증가, 함수 분리, `@trust` 사용

### 디버깅 팁

```bash
# SMT-LIB2 출력 확인
bmb verify file.bmb --emit-smt

# 단계별 검증
bmb verify file.bmb --step-by-step

# 반례 확인
bmb verify file.bmb --show-counterexample
```

---

## 10. Best Practices

### Do

1. **명확한 사전 조건** - 모든 가정을 명시
2. **테스트 가능한 사후 조건** - 검증 가능한 조건
3. **점진적 복잡성** - 단순한 조건부터 시작
4. **문서화** - 복잡한 조건에 주석 추가

### Don't

1. **과도한 @trust** - 핵심 로직에 사용 금지
2. **복잡한 중첩 조건** - 분리하여 단순화
3. **불필요한 런타임 비용** - 프로덕션에서 최적화
4. **타임아웃 무시** - 근본 원인 해결

---

## Further Reading

- `CONTRACT_PROGRAMMING.md` - 기본 계약 프로그래밍
- `PERFORMANCE.md` - 성능 최적화
- `docs/scenarios/CONTRACTS.md` - 계약 시나리오
- BMB Language Specification - 계약 문법 정의

