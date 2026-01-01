# BMB v0.1 Seed 구현 계획서

## 기술 스택

| 구성요소 | 라이브러리 | 버전 | 선택 이유 |
|----------|-----------|------|-----------|
| 렉서 | **logos** | 0.14 | 빠른 성능, derive 매크로, regex 지원 |
| 파서 | **lalrpop** | 0.21 | LR(1), logos 통합, 문법 파일 분리 |
| 에러 리포팅 | **ariadne** | 0.4 | 컴파일러급 진단, 멀티파일 지원 |
| CLI | **clap** | 4.x | 표준, derive 매크로 |
| 직렬화 | **serde** | 1.x | AST 직렬화 (디버그용) |

## 프로젝트 구조

```
lang-bmb/
├── Cargo.toml              # Workspace 루트
├── bmb/                    # 컴파일러 크레이트
│   ├── Cargo.toml
│   ├── build.rs            # lalrpop 빌드 스크립트
│   └── src/
│       ├── main.rs         # CLI 진입점
│       ├── lib.rs          # 라이브러리 루트
│       ├── lexer/
│       │   ├── mod.rs
│       │   └── token.rs    # logos 토큰 정의
│       ├── parser/
│       │   ├── mod.rs
│       │   └── grammar.lalrpop
│       ├── ast/
│       │   ├── mod.rs
│       │   ├── expr.rs     # 표현식 AST
│       │   ├── stmt.rs     # 문장 AST
│       │   ├── types.rs    # 타입 AST
│       │   └── span.rs     # 위치 정보
│       ├── types/
│       │   ├── mod.rs
│       │   └── checker.rs  # 타입 체커
│       └── error/
│           ├── mod.rs
│           └── reporter.rs # ariadne 리포터
├── tests/                  # 통합 테스트
│   ├── lexer/
│   ├── parser/
│   └── examples/           # BMB 예시 파일
└── docs/
```

## Phase 1: 프로젝트 초기화

### Cargo.toml (Workspace)
```toml
[workspace]
members = ["bmb"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/iyulab/lang-bmb"

[workspace.dependencies]
logos = "0.14"
lalrpop-util = "0.21"
ariadne = "0.4"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

## Phase 2: 렉서 (토큰 정의)

### v0.1 지원 토큰

```rust
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip r"--[^\n]*")]  // 주석
pub enum Token<'src> {
    // 키워드
    #[token("fn")]     Fn,
    #[token("let")]    Let,
    #[token("var")]    Var,
    #[token("if")]     If,
    #[token("then")]   Then,
    #[token("else")]   Else,
    #[token("pre")]    Pre,
    #[token("post")]   Post,
    #[token("true")]   True,
    #[token("false")]  False,

    // 타입 키워드
    #[token("i32")]    I32,
    #[token("i64")]    I64,
    #[token("f64")]    F64,
    #[token("bool")]   Bool,

    // 리터럴
    #[regex(r"[0-9]+", |lex| lex.slice())]
    IntLit(&'src str),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice())]
    FloatLit(&'src str),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice())]
    Ident(&'src str),

    // 기호
    #[token(":")]  Colon,
    #[token("->")]  Arrow,
    #[token("=>")]  FatArrow,
    #[token("=")]  Eq,
    #[token(";")]  Semi,
    #[token(",")]  Comma,
    #[token("(")]  LParen,
    #[token(")")]  RParen,
    #[token("{")]  LBrace,
    #[token("}")]  RBrace,

    // 연산자
    #[token("+")]  Plus,
    #[token("-")]  Minus,
    #[token("*")]  Star,
    #[token("/")]  Slash,
    #[token("==")]  EqEq,
    #[token("!=")]  NotEq,
    #[token("<")]  Lt,
    #[token(">")]  Gt,
    #[token("<=")]  LtEq,
    #[token(">=")]  GtEq,
    #[token("and")]  And,
    #[token("or")]  Or,
    #[token("not")]  Not,
}
```

## Phase 3: AST 정의

### 핵심 노드

```rust
/// 소스 위치
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// 프로그램 (최상위)
pub struct Program {
    pub items: Vec<Item>,
}

/// 최상위 아이템
pub enum Item {
    FnDef(FnDef),
}

/// 함수 정의
pub struct FnDef {
    pub name: Spanned<String>,
    pub params: Vec<Param>,
    pub ret_ty: Spanned<Type>,
    pub pre: Option<Spanned<Expr>>,   // pre 조건
    pub post: Option<Spanned<Expr>>,  // post 조건
    pub body: Spanned<Expr>,
    pub span: Span,
}

/// 파라미터
pub struct Param {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
}

/// 타입
pub enum Type {
    I32,
    I64,
    F64,
    Bool,
    Unit,
    Named(String),
}

/// 표현식
pub enum Expr {
    // 리터럴
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    Unit,

    // 변수
    Var(String),

    // 연산
    Binary(Box<Spanned<Expr>>, BinOp, Box<Spanned<Expr>>),
    Unary(UnOp, Box<Spanned<Expr>>),

    // 제어
    If(Box<Spanned<Expr>>, Box<Spanned<Expr>>, Box<Spanned<Expr>>),

    // 바인딩
    Let(String, Option<Spanned<Type>>, Box<Spanned<Expr>>, Box<Spanned<Expr>>),

    // 호출
    Call(String, Vec<Spanned<Expr>>),

    // 블록
    Block(Vec<Spanned<Expr>>),
}

/// 이항 연산자
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
}

/// 단항 연산자
pub enum UnOp {
    Neg, Not,
}
```

## Phase 4: 파서 (LALRPOP 문법)

### grammar.lalrpop 핵심 규칙

```lalrpop
use crate::ast::*;
use crate::lexer::Token;

grammar<'input>(input: &'input str);

extern {
    type Location = usize;
    type Error = LexError;
    enum Token<'input> { ... }
}

pub Program: Program = {
    <items:Item*> => Program { items }
};

Item: Item = {
    <f:FnDef> => Item::FnDef(f),
};

FnDef: FnDef = {
    "fn" <name:Ident> "(" <params:Params> ")" "->" <ret:Type>
    <pre:("pre" <Expr>)?>
    <post:("post" <Expr>)?>
    "=" <body:Expr> ";" => { ... }
};

Expr: Spanned<Expr> = {
    <l:@L> <e:ExprInner> <r:@R> => Spanned::new(e, Span::new(l, r))
};

ExprInner: Expr = {
    OrExpr,
    "if" <c:Expr> "then" <t:Expr> "else" <e:Expr> =>
        Expr::If(Box::new(c), Box::new(t), Box::new(e)),
    "let" <n:Ident> <ty:(":" <Type>)?> "=" <v:Expr> ";" <b:Expr> =>
        Expr::Let(n, ty, Box::new(v), Box::new(b)),
};

// 연산자 우선순위
OrExpr: Expr = { ... };
AndExpr: Expr = { ... };
CmpExpr: Expr = { ... };
AddExpr: Expr = { ... };
MulExpr: Expr = { ... };
UnaryExpr: Expr = { ... };
Primary: Expr = { ... };
```

## Phase 5: 타입 체커

### 기본 기능

```rust
pub struct TypeChecker {
    env: HashMap<String, Type>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    /// 함수 정의 검사
    pub fn check_fn(&mut self, f: &FnDef) -> Result<(), TypeError>;

    /// 표현식 타입 추론
    pub fn infer_expr(&mut self, e: &Expr) -> Result<Type, TypeError>;

    /// 타입 일치 검사
    pub fn unify(&self, expected: &Type, actual: &Type) -> Result<(), TypeError>;
}
```

### v0.1 검사 범위

| 검사 | 구현 |
|------|------|
| 기본 타입 일치 | ✅ |
| 함수 반환 타입 | ✅ |
| 변수 바인딩 타입 | ✅ |
| 연산자 타입 규칙 | ✅ |
| pre/post 표현식 | 파싱만 (검증은 v0.2) |

## Phase 6: 에러 리포터

### Ariadne 통합

```rust
pub fn report_error(source: &str, filename: &str, error: &CompileError) {
    let report = Report::build(ReportKind::Error, (filename, error.span.clone()))
        .with_code(error.code)
        .with_message(&error.message)
        .with_label(
            Label::new((filename, error.span.clone()))
                .with_message(&error.detail)
                .with_color(Color::Red),
        );

    if let Some(hint) = &error.hint {
        report.with_help(hint);
    }

    report.finish()
        .print((filename, Source::from(source)))
        .unwrap();
}
```

## Phase 7: CLI

### 명령어

```bash
bmb check <file.bmb>   # 타입 검사
bmb parse <file.bmb>   # AST 출력 (디버그)
bmb tokens <file.bmb>  # 토큰 출력 (디버그)
```

### Clap 정의

```rust
#[derive(Parser)]
#[command(name = "bmb", version, about = "BMB Compiler")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Type check a BMB source file
    Check { file: PathBuf },
    /// Parse and dump AST
    Parse { file: PathBuf },
    /// Tokenize and dump tokens
    Tokens { file: PathBuf },
}
```

## Phase 8: 테스트

### 테스트 케이스 구조

```
tests/
├── examples/
│   ├── valid/
│   │   ├── 001_identity.bmb
│   │   ├── 002_arithmetic.bmb
│   │   ├── 003_comparison.bmb
│   │   ├── 004_if_else.bmb
│   │   ├── 005_let_binding.bmb
│   │   ├── 010_simple_contract.bmb
│   │   └── ...
│   └── invalid/
│       ├── err_001_type_mismatch.bmb
│       ├── err_002_undefined_var.bmb
│       └── ...
├── lexer_tests.rs
├── parser_tests.rs
└── type_tests.rs
```

### 예시 BMB 코드

```bmb
-- 001_identity.bmb
fn identity(x: i32) -> i32 = x;

-- 002_arithmetic.bmb
fn add(a: i32, b: i32) -> i32 = a + b;

-- 010_simple_contract.bmb
fn abs(x: i32) -> i32
  pre true
  post ret >= 0
= if x >= 0 then x else 0 - x;
```

## 마일스톤 체크리스트

- [ ] Phase 1: Cargo workspace 초기화
- [ ] Phase 2: 렉서 구현 (토큰 30+)
- [ ] Phase 3: AST 정의 완료
- [ ] Phase 4: 파서 구현 (기본 문법)
- [ ] Phase 5: 타입체커 기본 완료
- [ ] Phase 6: 에러 리포터 구현
- [ ] Phase 7: CLI 구현
- [ ] Phase 8: 테스트 50+ 통과

## 예상 의존성

```toml
[dependencies]
logos = "0.14"
lalrpop-util = { version = "0.21", features = ["lexer"] }
ariadne = "0.4"
clap = { version = "4", features = ["derive"] }
thiserror = "2"

[build-dependencies]
lalrpop = "0.21"

[dev-dependencies]
insta = "1"  # 스냅샷 테스트
```
