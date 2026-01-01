# BMB v0.3 Root 구현 계획

## 구현 상태: ✅ 완료

- **Phase 1**: 인터프리터 코어 ✅
- **Phase 2**: 내장 함수 및 입출력 ✅
- **Phase 3**: REPL 환경 ✅
- **Phase 4**: CLI 통합 ✅
- **Phase 5**: 테스트 및 문서화 ✅

## 개요

v0.3은 인터프리터와 REPL을 추가하여 BMB 코드를 실행할 수 있게 합니다.

## 기술 스택

| 구성요소 | 라이브러리 | 용도 |
|----------|-----------|------|
| 인터프리터 | 자체 구현 | Tree-walking 인터프리터 |
| REPL | rustyline | 라인 편집, 히스토리 |
| 입출력 | 표준 라이브러리 | print, println, read |

## 인터프리터 설계

### 1. 값 표현 (Value)

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Unit,
    // 향후 확장: Array, Struct, Closure
}
```

### 2. 환경 (Environment)

```rust
pub struct Environment {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}
```

- 렉시컬 스코프 지원
- 함수 호출 시 새 환경 생성
- 부모 환경 참조로 스코프 체인 구성

### 3. 평가기 (Evaluator)

```rust
pub struct Interpreter {
    global_env: Rc<RefCell<Environment>>,
    functions: HashMap<String, FnDef>,
    builtins: HashMap<String, BuiltinFn>,
}

impl Interpreter {
    pub fn eval(&self, expr: &Expr, env: &Env) -> Result<Value, RuntimeError>;
    pub fn run(&self, program: &Program) -> Result<Value, RuntimeError>;
}
```

### 4. 평가 규칙

| 표현식 | 평가 방식 |
|--------|----------|
| `IntLit(n)` | `Value::Int(n)` |
| `FloatLit(f)` | `Value::Float(f)` |
| `BoolLit(b)` | `Value::Bool(b)` |
| `Unit` | `Value::Unit` |
| `Var(name)` | 환경에서 조회 |
| `Binary` | 좌/우 평가 후 연산 |
| `Unary` | 피연산자 평가 후 연산 |
| `If` | 조건 평가 → 분기 |
| `Let` | value 평가 → 바인딩 → body 평가 |
| `Call` | 인자 평가 → 함수 본문 평가 |
| `Block` | 순차 평가, 마지막 값 반환 |

### 5. 내장 함수

| 함수 | 시그니처 | 설명 |
|------|----------|------|
| `print` | `(x: i64) -> ()` | 값 출력 (줄바꿈 없음) |
| `println` | `(x: i64) -> ()` | 값 출력 (줄바꿈) |
| `read_int` | `() -> i64` | 정수 입력 |
| `assert` | `(cond: bool) -> ()` | 조건 검사 |
| `abs` | `(n: i64) -> i64` | 절대값 |
| `min` | `(a: i64, b: i64) -> i64` | 최소값 |
| `max` | `(a: i64, b: i64) -> i64` | 최대값 |

## 프로젝트 구조

```
bmb/src/
├── interp/
│   ├── mod.rs          # 모듈 진입점 및 내보내기
│   ├── value.rs        # Value 타입 정의 (Int, Float, Bool, Unit)
│   ├── env.rs          # Environment 스코프 체인
│   ├── eval.rs         # Interpreter 및 내장 함수 구현
│   └── error.rs        # RuntimeError 정의
├── repl/
│   └── mod.rs          # REPL 구현 (rustyline 통합)
examples/
└── hello.bmb           # 예제 프로그램
```

## 구현 페이즈

### Phase 1: 인터프리터 코어

1. `value.rs`: Value 열거형 정의
2. `env.rs`: Environment 스코프 체인
3. `eval.rs`: 표현식 평가기
   - 리터럴 평가
   - 변수 조회
   - 이항/단항 연산
   - 조건문
   - let 바인딩
   - 함수 호출

### Phase 2: 내장 함수 및 입출력

1. `builtins.rs`:
   - print/println 구현
   - read_line 구현
   - assert 구현
2. 함수 테이블 관리

### Phase 3: REPL 환경

1. rustyline 통합
2. 표현식/문장 파싱
3. 히스토리 저장
4. 에러 복구

### Phase 4: CLI 통합

1. `bmb run <file>` 명령어
2. `bmb repl` 명령어
3. 종료 코드 정의

### Phase 5: 테스트 및 문서화

1. 단위 테스트 작성
2. 통합 테스트 (예제 실행)
3. 문서 최신화

## 예상 출력

### bmb run

```bash
$ bmb run examples/hello.bmb
Hello, BMB!

$ bmb run examples/factorial.bmb
120
```

### bmb repl

```
BMB REPL v0.3
Type :help for help, :quit to exit.

> 1 + 2
3

> let x = 10; x * 2
20

> fn square(n: i32) -> i32 = n * n
defined: square

> square(5)
25

> :quit
Goodbye!
```

## 테스트 케이스

| 카테고리 | 테스트 수 | 예시 |
|----------|-----------|------|
| 리터럴 | 10 | 정수, 실수, 불린 |
| 연산 | 20 | 산술, 비교, 논리 |
| 제어흐름 | 15 | if-else, 중첩 조건 |
| 바인딩 | 10 | let, 섀도잉 |
| 함수 | 20 | 호출, 재귀 |
| 내장함수 | 10 | print, assert |
| REPL | 15 | 대화형 입력 |

**총: 100+ 테스트**
