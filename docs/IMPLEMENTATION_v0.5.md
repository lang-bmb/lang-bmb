# BMB v0.5 Branch - Implementation Plan

## 목표

```
BMB로 BMB 컴파일러 재작성 시작을 위한 언어 기능 확장
```

## 철학 정렬 (LAWS.md)

v0.5 구현은 다음 BMB 설계 원칙을 준수합니다:

1. **AI-Native 설계**: 모든 새 구문은 AI가 이해하기 쉽게 명시적
2. **계약 우선**: 새 타입들도 계약 검증 대상
3. **안전성**: 참조와 소유권은 컴파일 타임 검증
4. **표현력**: 패턴 매칭으로 복잡한 분기를 명확하게

## Phase 1: 데이터 타입 기반 (Structs, Enums, Match)

### 1.1 Struct 정의

```bmb
-- 구조체 정의
struct Token {
    kind: TokenKind,
    span: Span,
    value: i64
}

struct Span {
    start: i64,
    end: i64
}

-- 구조체 생성
fn make_token(kind: TokenKind, start: i64, end: i64) -> Token {
    Token {
        kind: kind,
        span: Span { start: start, end: end },
        value: 0
    }
}

-- 필드 접근
fn token_length(t: Token) -> i64 {
    t.span.end - t.span.start
}
```

#### 구현 태스크

| 태스크 | 파일 | 설명 |
|--------|------|------|
| Struct AST 노드 | ast/mod.rs | `Item::StructDef` 추가 |
| Struct 타입 | ast/types.rs | `Type::Struct(name, fields)` |
| Struct 파싱 | bmb.lalrpop | `struct Name { field: Type, ... }` |
| Struct 표현식 | ast/expr.rs | `Expr::StructInit`, `Expr::FieldAccess` |
| 타입 체크 | types/mod.rs | 구조체 타입 검사 |
| MIR 확장 | mir/mod.rs | 구조체 Place 지원 |
| LLVM codegen | codegen/llvm.rs | LLVM struct 타입 매핑 |

### 1.2 Enum 정의

```bmb
-- 열거형 정의
enum TokenKind {
    Ident,
    IntLit,
    Plus,
    Minus,
    LParen,
    RParen,
    Eof
}

-- 데이터를 가진 열거형 (v0.5.1+)
enum Expr {
    IntLit(i64),
    Binary { left: Expr, op: BinOp, right: Expr },
    If { cond: Expr, then_branch: Expr, else_branch: Expr }
}

-- 열거형 생성
fn make_plus() -> TokenKind {
    TokenKind::Plus
}
```

#### 구현 태스크

| 태스크 | 파일 | 설명 |
|--------|------|------|
| Enum AST 노드 | ast/mod.rs | `Item::EnumDef` 추가 |
| Enum 타입 | ast/types.rs | `Type::Enum(name, variants)` |
| Enum 파싱 | bmb.lalrpop | `enum Name { Variant, ... }` |
| Variant 표현식 | ast/expr.rs | `Expr::EnumVariant` |
| 타입 체크 | types/mod.rs | 열거형 타입 검사 |
| MIR 확장 | mir/mod.rs | 태그 + 데이터 표현 |
| LLVM codegen | codegen/llvm.rs | Tagged union 매핑 |

### 1.3 Pattern Matching (match)

```bmb
-- 기본 match
fn eval_op(kind: TokenKind) -> i64 {
    match kind {
        TokenKind::Plus => 1,
        TokenKind::Minus => 2,
        _ => 0
    }
}

-- 데이터 추출 (v0.5.1+)
fn eval(expr: Expr) -> i64 {
    match expr {
        Expr::IntLit(n) => n,
        Expr::Binary { left, op, right } => {
            let l = eval(left);
            let r = eval(right);
            l + r
        },
        _ => 0
    }
}
```

#### 구현 태스크

| 태스크 | 파일 | 설명 |
|--------|------|------|
| Match AST 노드 | ast/expr.rs | `Expr::Match` |
| Pattern AST | ast/expr.rs | `Pattern::Variant`, `Pattern::Wildcard` |
| Match 파싱 | bmb.lalrpop | `match expr { pat => expr, ... }` |
| 패턴 검사 | types/mod.rs | exhaustiveness check |
| MIR 확장 | mir/lower.rs | match → switch/branch |
| LLVM codegen | codegen/llvm.rs | switch instruction |

---

## Phase 2: 컬렉션 & 문자열

### 2.1 String 타입

```bmb
-- 문자열 리터럴
fn greet() -> String {
    "Hello, BMB!"
}

-- 문자열 연결
fn hello(name: String) -> String {
    "Hello, " + name + "!"
}

-- 문자열 길이
fn length(s: String) -> i64 {
    s.len()
}

-- 문자 접근
fn first_char(s: String) -> i64 {
    s.at(0)  -- ASCII 코드
}
```

#### 구현 태스크

| 태스크 | 파일 | 설명 |
|--------|------|------|
| String 타입 | ast/types.rs | `Type::String` |
| 문자열 리터럴 | lexer/token.rs, bmb.lalrpop | `"string"` 파싱 |
| 문자열 연산 | ast/expr.rs | `+` 오버로드 |
| 런타임 함수 | runtime/runtime.c | `bmb_string_*` 함수들 |
| 타입 체크 | types/mod.rs | 문자열 타입 검사 |
| MIR/LLVM | mir/, codegen/ | 포인터 + 길이 구조 |

### 2.2 Array/Vec 타입

```bmb
-- 배열 리터럴
fn numbers() -> [i64; 3] {
    [1, 2, 3]
}

-- 동적 벡터
fn make_vec() -> Vec<i64> {
    let v = Vec::new();
    v.push(1);
    v.push(2);
    v
}

-- 인덱스 접근
fn get(arr: [i64; 3], idx: i64) -> i64
    pre idx >= 0 and idx < 3
{
    arr[idx]
}
```

#### 구현 태스크

| 태스크 | 파일 | 설명 |
|--------|------|------|
| Array 타입 | ast/types.rs | `Type::Array(T, size)` |
| Vec 타입 | ast/types.rs | `Type::Vec(T)` (제네릭 부분) |
| 배열 리터럴 | ast/expr.rs | `Expr::ArrayLit` |
| 인덱스 연산 | ast/expr.rs | `Expr::Index` |
| 경계 검사 | types/, verify/ | 계약으로 검증 |
| 런타임 | runtime/ | `bmb_vec_*` 함수들 |
| MIR/LLVM | mir/, codegen/ | 포인터 연산 |

---

## Phase 3: 루프 & 제어흐름

### 3.1 while 루프

```bmb
-- 기본 while
fn sum_to(n: i64) -> i64
    pre n >= 0
    post ret >= 0
{
    let mut result = 0;
    let mut i = 0;
    while i <= n {
        result = result + i;
        i = i + 1
    };
    result
}
```

#### 구현 태스크

| 태스크 | 파일 | 설명 |
|--------|------|------|
| While AST | ast/expr.rs | `Expr::While { cond, body }` |
| 가변 바인딩 | ast/expr.rs | `Expr::Let { mutable: bool }` |
| 할당 표현식 | ast/expr.rs | `Expr::Assign` |
| While 파싱 | bmb.lalrpop | `while cond { body }` |
| 타입 체크 | types/mod.rs | 가변성 추적 |
| MIR | mir/lower.rs | loop header/body/exit 블록 |
| LLVM | codegen/llvm.rs | br/loop |

### 3.2 for 루프 (v0.5.2+)

```bmb
-- 범위 기반 for
fn sum_range() -> i64 {
    let mut sum = 0;
    for i in 0..10 {
        sum = sum + i
    };
    sum
}

-- 배열 순회
fn sum_array(arr: [i64; 5]) -> i64 {
    let mut sum = 0;
    for x in arr {
        sum = sum + x
    };
    sum
}
```

---

## Phase 4: 모듈 시스템

### 4.1 기본 모듈

```bmb
-- lexer.bmb
pub struct Token { ... }
pub enum TokenKind { ... }
pub fn tokenize(source: String) -> Vec<Token> { ... }

-- parser.bmb
use lexer::{Token, TokenKind};
pub fn parse(tokens: Vec<Token>) -> Expr { ... }

-- main.bmb
use lexer::tokenize;
use parser::parse;

fn main() -> i64 {
    let tokens = tokenize("1 + 2");
    let ast = parse(tokens);
    0
}
```

#### 구현 태스크

| 태스크 | 파일 | 설명 |
|--------|------|------|
| Module AST | ast/mod.rs | `Item::Use`, `Item::Mod` |
| pub 가시성 | ast/mod.rs | visibility modifier |
| 모듈 해석 | resolver.rs (신규) | 이름 해석 |
| 파일 탐색 | build/ | 멀티 파일 컴파일 |

---

## Phase 5: 참조 (기본)

### 5.1 불변 참조

```bmb
-- 참조 생성 및 역참조
fn double(x: &i64) -> i64 {
    *x * 2
}

fn main() -> i64 {
    let n = 21;
    double(&n)
}
```

### 5.2 가변 참조 (v0.5.3+)

```bmb
fn increment(x: &mut i64) -> () {
    *x = *x + 1
}
```

---

## 구현 우선순위

### Phase 1 (Essential) - 2주

1. **Struct 기본** - 필드 정의, 생성, 접근
2. **Enum 기본** - 단순 variant (데이터 없음)
3. **Match 기본** - 단순 패턴, wildcard

### Phase 2 (Core) - 2주

4. **String 타입** - 리터럴, 기본 연산
5. **가변 변수** - `let mut`, 할당
6. **while 루프** - 기본 반복

### Phase 3 (Extension) - 2주

7. **Array 타입** - 고정 크기 배열
8. **Enum 데이터** - 데이터를 가진 variant
9. **Match 패턴** - 데이터 추출 패턴

### Phase 4 (Module) - 2주

10. **모듈 시스템** - use, pub
11. **References** - &T 기본

---

## 마일스톤 체크리스트

### Phase 1 완료 조건
- [ ] struct 정의 및 생성 가능
- [ ] enum 정의 및 variant 사용 가능
- [ ] match 기본 패턴 매칭 작동
- [ ] 모든 기존 테스트 통과
- [ ] 새 기능 테스트 10+

### Phase 2 완료 조건
- [ ] 문자열 리터럴 파싱 및 사용
- [ ] let mut 가변 바인딩
- [ ] 할당 연산자 작동
- [ ] while 루프 작동
- [ ] 새 기능 테스트 10+

### Phase 3 완료 조건
- [ ] 고정 크기 배열 지원
- [ ] enum 데이터 variant
- [ ] match 데이터 추출 패턴
- [ ] 새 기능 테스트 10+

### Phase 4 완료 조건
- [ ] 멀티 파일 컴파일
- [ ] use/pub 키워드 작동
- [ ] &T 참조 기본 지원
- [ ] 자기 컴파일 시작 가능 (렉서 BMB 작성)

---

## 리스크 & 완화

| 리스크 | 영향 | 완화 |
|--------|------|------|
| MIR 확장 복잡도 | 높음 | 단계적 확장, 테스트 우선 |
| LLVM ABI 호환성 | 중간 | 표준 레이아웃 사용 |
| 패턴 매칭 exhaustiveness | 중간 | 먼저 기본 구현, 후에 검사 추가 |
| 런타임 확장 | 낮음 | C 런타임 점진적 확장 |

---

## 의존성

```
v0.4 (완료)
  └── v0.5 Phase 1 (Structs, Enums, Match)
        └── v0.5 Phase 2 (String, Mut, While)
              └── v0.5 Phase 3 (Array, Enum Data)
                    └── v0.5 Phase 4 (Modules, References)
                          └── v0.6 (Standard Library)
```
