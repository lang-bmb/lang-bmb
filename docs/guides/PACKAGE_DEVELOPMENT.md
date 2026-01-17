# BMB Package Development Guide

> gotgan 패키지 매니저를 위한 패키지 개발 가이드

---

## Overview

이 가이드는 BMB 생태계를 위한 gotgan 패키지를 개발하고 배포하는 방법을 설명합니다.

### 패키지 개발 워크플로우

```
1. 계획 → 2. 구현 → 3. 테스트 → 4. 계약 추가 → 5. 문서화 → 6. 배포
```

---

## 1. 프로젝트 구조

### 표준 레이아웃

```
my-package/
├── gotgan.toml          # 패키지 메타데이터
├── src/
│   └── lib.bmb          # 메인 라이브러리
├── tests/
│   └── test_lib.bmb     # 테스트 파일
├── examples/
│   └── basic.bmb        # 사용 예제
└── README.md            # 문서
```

### gotgan.toml 구조

```toml
[package]
name = "my-package"
version = "0.1.0"
description = "A useful BMB package"
authors = ["Your Name <email@example.com>"]
license = "MIT"
repository = "https://github.com/user/my-package"
keywords = ["utility", "data-structures"]

[dependencies]
bmb-testing = "0.1.0"

[dev-dependencies]
bmb-benchmark = "0.1.0"
```

---

## 2. Rust에서 포팅하기

### 워크플로우

```bash
# 1. Rust 크레이트 소스 준비
git clone https://github.com/rust-lang/example-crate
cd example-crate

# 2. BMB로 변환
node tools/rust_to_bmb.mjs src/*.rs --stats   # 미리보기
node tools/rust_to_bmb.mjs src/*.rs --apply   # 변환

# 3. 수동 조정
# - BMB에 없는 기능 제거/대체
# - 계약 추가
# - 테스트 작성
```

### 변환 도구 지원

| Rust 문법 | BMB 변환 | 지원 여부 |
|-----------|----------|----------|
| `fn name()` | `fn name()` | ✅ |
| `struct` | `struct` | ✅ |
| `enum` | `enum` | ✅ |
| `impl` | `impl` | ✅ |
| `match` | `match` | ✅ |
| `Option<T>` | `T?` | ✅ |
| `Result<T, E>` | `Result<T, E>` | ✅ |
| `&mut` | 값 복사 | ⚠️ 수동 조정 |
| `lifetime` | 불필요 | ⚠️ 제거 |
| `async/await` | 미지원 | ❌ |

### 예시: HashMap 포팅

#### Rust 원본
```rust
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    len: usize,
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: vec![Vec::new(); 16],
            len: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let idx = self.hash(&key) % self.buckets.len();
        // ...
    }
}
```

#### BMB 변환
```bmb
// 간소화된 HashMap (i64 키 전용)
struct HashMap {
    handle: i64  // 내부 핸들
}

fn hashmap_create() -> HashMap
  post ret.handle > 0
= HashMap { handle: __builtin_hashmap_new() };

fn hashmap_put(map: HashMap, key: i64, value: i64) -> i64
  pre map.handle > 0
= __builtin_hashmap_put(map.handle, key, value);

fn hashmap_get(map: HashMap, key: i64) -> i64?
  pre map.handle > 0
= {
    let result = __builtin_hashmap_get(map.handle, key);
    if result == -9999 { None } else { Some(result) }
};
```

---

## 3. 계약 추가하기

### 단계별 가이드

```bmb
// 1단계: 기본 구현
fn divide(a: i64, b: i64) -> i64 = a / b;

// 2단계: 사전 조건 추가
fn divide(a: i64, b: i64) -> i64
  pre b != 0  // 0으로 나누기 방지
= a / b;

// 3단계: 사후 조건 추가
fn divide(a: i64, b: i64) -> i64
  pre b != 0
  post b * ret <= a  // 정수 나눗셈 특성
  post b * (ret + 1) > a
= a / b;

// 4단계: 정제 타입 사용
type NonZero = i64 where self != 0;

fn divide(a: i64, b: NonZero) -> i64
  post b * ret <= a
= a / b;
```

### 계약 패턴

#### 컬렉션 계약
```bmb
fn push(vec: Vec<i64>, item: i64) -> Vec<i64>
  post ret.len() == vec.len() + 1
  post ret[ret.len() - 1] == item
= __builtin_vec_push(vec, item);

fn pop(vec: Vec<i64>) -> (Vec<i64>, i64?)
  post vec.len() == 0 implies ret.1 == None
  post vec.len() > 0 implies ret.0.len() == vec.len() - 1
= __builtin_vec_pop(vec);
```

#### 변환 계약
```bmb
fn parse_int(s: String) -> i64?
  post s.len() == 0 implies ret == None
  post ret != None implies int_to_string(ret.unwrap()) == s
= parse_int_impl(s);
```

---

## 4. 테스트 작성

### 테스트 구조

```bmb
// tests/test_lib.bmb

fn test_basic_operations() -> i64 = {
    let map = hashmap_create();
    let _ = hashmap_put(map, 1, 100);
    let _ = hashmap_put(map, 2, 200);

    let t1 = if hashmap_get(map, 1) == Some(100) { 1 } else { 0 };
    let t2 = if hashmap_get(map, 2) == Some(200) { 1 } else { 0 };
    let t3 = if hashmap_get(map, 3) == None { 1 } else { 0 };

    if t1 + t2 + t3 == 3 { println(777); 1 } else { 0 }
};

fn test_edge_cases() -> i64 = {
    let map = hashmap_create();

    // 빈 맵 테스트
    let t1 = if hashmap_size(map) == 0 { 1 } else { 0 };

    // 중복 키 테스트
    let _ = hashmap_put(map, 1, 100);
    let _ = hashmap_put(map, 1, 200);  // 덮어쓰기
    let t2 = if hashmap_get(map, 1) == Some(200) { 1 } else { 0 };

    if t1 + t2 == 2 { println(888); 1 } else { 0 }
};

fn run_tests() -> i64 = {
    let t1 = test_basic_operations();
    let t2 = test_edge_cases();
    t1 + t2
};

fn main() -> i64 = run_tests();
```

### 테스트 실행

```bash
# 단일 테스트 파일
bmb run tests/test_lib.bmb

# 전체 테스트
bmb test my-package/

# 계약 검증
bmb verify src/lib.bmb
```

---

## 5. 문서화

### README.md 템플릿

```markdown
# my-package

A brief description of what this package does.

## Installation

\`\`\`toml
[dependencies]
my-package = "0.1.0"
\`\`\`

## Usage

\`\`\`bmb
import my_package::*;

fn main() -> i64 = {
    let result = my_function(42);
    println(result)
};
\`\`\`

## API Reference

### Functions

#### `my_function(x: i64) -> i64`
Description of what the function does.

**Contracts:**
- Pre: `x >= 0`
- Post: `ret >= x`

## Examples

See `examples/` directory.

## License

MIT
\`\`\`

### 인라인 문서화

```bmb
/// 두 숫자를 더합니다.
///
/// # Arguments
/// * `a` - 첫 번째 숫자
/// * `b` - 두 번째 숫자
///
/// # Returns
/// 두 숫자의 합
///
/// # Example
/// ```bmb
/// let sum = add(2, 3);  // 5
/// ```
fn add(a: i64, b: i64) -> i64
  post ret == a + b
= a + b;
```

---

## 6. 배포

### 로컬 테스트

```bash
# 패키지 구조 검증
gotgan check

# 로컬 빌드
gotgan build

# 로컬 테스트
gotgan test
```

### gotgan-packages 등록

```bash
# 1. 포크 및 클론
git clone https://github.com/YOUR_USERNAME/gotgan-packages
cd gotgan-packages

# 2. 패키지 추가
mkdir -p packages/my-package
cp -r /path/to/my-package/* packages/my-package/

# 3. 커밋 및 PR
git add packages/my-package
git commit -m "Add my-package v0.1.0"
git push origin main
# GitHub에서 PR 생성
```

### 버전 관리

```toml
# gotgan.toml
[package]
version = "0.1.0"  # 시맨틱 버저닝

# 변경 유형별 버전 증가:
# - 패치 (0.1.0 → 0.1.1): 버그 수정
# - 마이너 (0.1.0 → 0.2.0): 새 기능 (하위 호환)
# - 메이저 (0.1.0 → 1.0.0): 호환성 파괴 변경
```

---

## 7. 기존 패키지 예시

### bmb-collections (377 LOC)

```bmb
// HashMap wrapper
fn hashmap_create() -> HashMap
  post ret.handle > 0
= HashMap { handle: __builtin_hashmap_new() };

fn hashmap_put(map: HashMap, key: i64, value: i64) -> i64
  pre map.handle > 0
= __builtin_hashmap_put(map.handle, key, value);

// VecDeque 구현
fn deque_new() -> VecDeque
  post ret.head == 0 && ret.tail == 0
= VecDeque { head: 0, tail: 0, data: vec_new() };

fn deque_push_back(d: VecDeque, val: i64) -> VecDeque
  post ret.size() == d.size() + 1
= VecDeque {
    head: d.head,
    tail: d.tail + 1,
    data: vec_push(d.data, val)
};
```

### bmb-json (479 LOC)

```bmb
// JSON 타입 감지
fn json_type(json: String, pos: i64) -> i64
  pre pos >= 0 && pos < json.len()
= {
    let c = json.byte_at(pos);
    if c == 123 { type_object() }      // {
    else if c == 91 { type_array() }   // [
    else if c == 34 { type_string() }  // "
    else if is_digit(c) || c == 45 { type_number() }
    else if c == 116 || c == 102 { type_bool() }  // t, f
    else if c == 110 { type_null() }   // n
    else { type_invalid() }
};

// 키로 값 찾기
fn json_get_key(json: String, pos: i64, key: String) -> i64
  pre json_type(json, pos) == type_object()
  post ret == -1 || json_type(json, ret) != type_invalid()
= find_key_impl(json, pos, key);
```

---

## 8. Best Practices

### Do

1. **작은 단위로 시작** - 핵심 기능부터 구현
2. **계약 우선** - 구현 전 계약 설계
3. **테스트 커버리지** - 모든 공개 함수 테스트
4. **명확한 API** - 직관적인 함수명과 파라미터

### Don't

1. **과도한 의존성** - 필요한 것만 의존
2. **복잡한 제네릭** - BMB 타입 시스템에 맞게 단순화
3. **불필요한 @trust** - 검증 가능하면 검증
4. **문서 누락** - 모든 공개 API 문서화

---

## 9. 트러블슈팅

### 빌드 오류

```
Error: Unknown builtin function '__custom_fn'
```
**해결**: 빌트인 함수는 컴파일러 지원 필요. 일반 함수로 구현하거나 FFI 사용.

### 테스트 실패

```
Test failed: expected 777, got 0
```
**해결**: 테스트 로직 검토, 디버그 출력 추가.

### 계약 검증 실패

```
Cannot verify post-condition
```
**해결**: 조건 단순화, 보조 조건 추가, 또는 `@trust` 사용.

---

## Further Reading

- `MIGRATION_v0.32.md` - 문법 마이그레이션
- `tutorials/ADVANCED_CONTRACTS.md` - 고급 계약
- `ECOSYSTEM.md` - 생태계 개요
- gotgan 공식 문서 - 패키지 매니저 사용법

