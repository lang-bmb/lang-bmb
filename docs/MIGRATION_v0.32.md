# BMB v0.32 Syntax Migration Guide

> Pre-v0.32 문법에서 v0.32 문법으로의 마이그레이션 가이드

---

## Overview

v0.32에서 BMB 문법이 Rust와 더 유사하게 변경되었습니다. 이 가이드는 기존 코드를 새로운 문법으로 마이그레이션하는 방법을 설명합니다.

### 주요 변경사항

| 변경 | Pre-v0.32 | v0.32 |
|------|-----------|-------|
| 주석 | `-- comment` | `// comment` |
| 조건문 | `if cond then a else b` | `if cond { a } else { b }` |
| 옵션 타입 | `Option<T>` | `T?` |
| 논리 연산자 | `and`, `or`, `not` | `&&`, `\|\|`, `!` (선택적) |
| 시프트 연산자 | 미지원 | `<<`, `>>` |

---

## 1. 주석 마이그레이션

### Before (Pre-v0.32)
```bmb
-- This is a comment
fn add(a: i64, b: i64) -> i64 = a + b;  -- inline comment
```

### After (v0.32)
```bmb
// This is a comment
fn add(a: i64, b: i64) -> i64 = a + b;  // inline comment
```

### 자동 변환
```bash
# 단일 파일
sed -i 's/--/\/\//g' file.bmb

# 전체 프로젝트
node tools/migrate_syntax.mjs **/*.bmb --apply
```

---

## 2. 조건문 마이그레이션

### Before (Pre-v0.32)
```bmb
fn max(a: i64, b: i64) -> i64 =
    if a > b then a else b;

fn abs(x: i64) -> i64 =
    if x < 0 then 0 - x else x;

fn classify(n: i64) -> String =
    if n < 0 then "negative"
    else if n == 0 then "zero"
    else "positive";
```

### After (v0.32)
```bmb
fn max(a: i64, b: i64) -> i64 =
    if a > b { a } else { b };

fn abs(x: i64) -> i64 =
    if x < 0 { 0 - x } else { x };

fn classify(n: i64) -> String =
    if n < 0 { "negative" }
    else if n == 0 { "zero" }
    else { "positive" };
```

### 복잡한 조건문

#### Before
```bmb
fn complex(a: i64, b: i64, c: i64) -> i64 =
    if a > 0 then
        if b > 0 then a + b
        else a - b
    else
        if c > 0 then c
        else 0;
```

#### After
```bmb
fn complex(a: i64, b: i64, c: i64) -> i64 =
    if a > 0 {
        if b > 0 { a + b }
        else { a - b }
    }
    else {
        if c > 0 { c }
        else { 0 }
    };
```

### 블록 내 여러 표현식

#### Before
```bmb
fn with_side_effects(x: i64) -> i64 =
    if x > 0 then
        let _ = println(x);
        x * 2
    else
        let _ = println(0);
        0;
```

#### After
```bmb
fn with_side_effects(x: i64) -> i64 =
    if x > 0 {
        let _ = println(x);
        x * 2
    }
    else {
        let _ = println(0);
        0
    };
```

---

## 3. 옵션 타입 마이그레이션

### Before (Pre-v0.32)
```bmb
fn find(arr: Vec<i64>, target: i64) -> Option<i64> =
    if contains(arr, target) then Some(target) else None;

fn safe_div(a: i64, b: i64) -> Option<i64> =
    if b == 0 then None else Some(a / b);
```

### After (v0.32)
```bmb
fn find(arr: Vec<i64>, target: i64) -> i64? =
    if contains(arr, target) { Some(target) } else { None };

fn safe_div(a: i64, b: i64) -> i64? =
    if b == 0 { None } else { Some(a / b) };
```

### 제네릭 옵션

#### Before
```bmb
fn map<T, U>(opt: Option<T>, f: fn(T) -> U) -> Option<U> =
    match opt {
        Some(x) => Some(f(x)),
        None => None
    };
```

#### After
```bmb
fn map<T, U>(opt: T?, f: fn(T) -> U) -> U? =
    match opt {
        Some(x) => Some(f(x)),
        None => None
    };
```

---

## 4. 논리 연산자 (선택적)

v0.32에서는 `and`, `or`, `not`과 `&&`, `||`, `!` 모두 지원됩니다.
통일성을 위해 새 문법 사용을 권장합니다.

### Before (Pre-v0.32)
```bmb
fn is_valid(x: i64) -> bool =
    x > 0 and x < 100 and not (x == 50);

fn check(a: bool, b: bool) -> bool =
    a or b and not a;
```

### After (v0.32 - 권장)
```bmb
fn is_valid(x: i64) -> bool =
    x > 0 && x < 100 && !(x == 50);

fn check(a: bool, b: bool) -> bool =
    a || b && !a;
```

---

## 5. 시프트 연산자 (신규)

v0.32에서 비트 시프트 연산자가 추가되었습니다.

```bmb
fn shift_left(x: i64, n: i64) -> i64 = x << n;
fn shift_right(x: i64, n: i64) -> i64 = x >> n;

fn power_of_two(n: i64) -> i64 = 1 << n;
fn divide_by_two(x: i64) -> i64 = x >> 1;
```

---

## 자동 마이그레이션 도구

### migrate_syntax.mjs 사용법

```bash
# 변경 사항 미리보기 (적용하지 않음)
node tools/migrate_syntax.mjs path/to/*.bmb --stats

# 변경 사항 적용
node tools/migrate_syntax.mjs path/to/*.bmb --apply

# 단일 파일
node tools/migrate_syntax.mjs myfile.bmb --apply

# 전체 프로젝트
node tools/migrate_syntax.mjs bootstrap/**/*.bmb --stats
node tools/migrate_syntax.mjs bootstrap/**/*.bmb --apply
```

### 도구 기능

| 기능 | 설명 |
|------|------|
| `--stats` | 변경될 라인 수 표시 (dry-run) |
| `--apply` | 실제 파일 수정 |
| 주석 변환 | `--` → `//` |
| 조건문 변환 | `if then else` → `if { } else { }` |
| 옵션 변환 | `Option<T>` → `T?` |

### 예시 출력

```
$ node tools/migrate_syntax.mjs bootstrap/lexer.bmb --stats

Migration Statistics for bootstrap/lexer.bmb:
  Comments (-- → //): 45 changes
  If-then-else: 23 changes
  Option types: 8 changes
  Total: 76 changes

Run with --apply to make changes.
```

---

## Bootstrap 컴파일러 마이그레이션

Bootstrap 컴파일러 (~30K LOC)는 2026-01-12에 v0.32로 마이그레이션 완료되었습니다.

### 마이그레이션된 파일

| 파일 | LOC | 변경 수 |
|------|-----|--------|
| `bootstrap/lexer.bmb` | 2,500 | 120 |
| `bootstrap/parser.bmb` | 4,200 | 280 |
| `bootstrap/types.bmb` | 5,800 | 350 |
| `bootstrap/mir.bmb` | 3,100 | 190 |
| `bootstrap/llvm_ir.bmb` | 6,200 | 410 |
| `bootstrap/compiler.bmb` | 8,500 | 520 |
| **Total** | ~30,000 | ~1,870 |

### 검증

```bash
# 마이그레이션 후 테스트 실행
bmb run bootstrap/lexer.bmb   # 777...888...999 출력 확인
bmb run bootstrap/types.bmb   # 530+ 테스트 통과
bmb run bootstrap/compiler.bmb  # 전체 컴파일러 테스트
```

---

## 수동 마이그레이션 체크리스트

자동 도구가 처리하지 못하는 경우를 위한 수동 체크리스트:

### 1. 주석
- [ ] 모든 `--` 주석을 `//`로 변경
- [ ] 문자열 내의 `--`는 변경하지 않음

### 2. 조건문
- [ ] `if ... then ... else ...` 패턴 찾기
- [ ] 각 분기에 중괄호 추가
- [ ] 세미콜론 위치 확인 (마지막 분기 뒤)

### 3. 옵션 타입
- [ ] 함수 반환 타입의 `Option<T>` → `T?`
- [ ] 파라미터 타입의 `Option<T>` → `T?`
- [ ] 제네릭 내의 `Option<T>` → `T?`

### 4. 검증
- [ ] `bmb check` 실행하여 구문 오류 확인
- [ ] 모든 테스트 통과 확인

---

## 호환성 참고사항

### 지원 중단 예정 (Deprecated)

| 문법 | 상태 | 제거 예정 |
|------|------|----------|
| `--` 주석 | 경고 없이 지원 | v2.0 |
| `if then else` | 경고 없이 지원 | v2.0 |
| `and`/`or`/`not` | 영구 지원 | 해당 없음 |

### 권장 사항

1. **새 코드**: 항상 v0.32 문법 사용
2. **기존 코드**: 점진적 마이그레이션 권장
3. **혼합 사용**: 파일 단위로 일관성 유지

---

## 문제 해결

### 일반적인 오류

#### 1. 중괄호 누락
```
Error: Expected '{' after 'if' condition
```
**해결**: `if cond then` → `if cond {`

#### 2. 세미콜론 위치
```
Error: Expected ';' after expression
```
**해결**: 마지막 `}` 뒤에 세미콜론 확인

#### 3. 중첩 조건문
```
Error: Unexpected token 'else'
```
**해결**: 내부 조건문의 중괄호 확인

### 도움 받기

- GitHub Issues: https://github.com/anthropics/lang-bmb/issues
- Discord: BMB 커뮤니티 채널

---

## Version History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-12 | v0.32 | Bootstrap 마이그레이션 완료 |
| 2026-01-14 | v0.32 | 마이그레이션 가이드 작성 |

