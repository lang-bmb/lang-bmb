# BMB v0.4 Stem 구현 계획

## 구현 상태: 🚧 진행 중

- **Phase 0**: 요구사항 분석 및 기술 조사 🚧
- **Phase 1**: MIR (중간 표현) 설계 및 구현 ⏳
- **Phase 2**: LLVM IR 생성기 구현 ⏳
- **Phase 3**: 링커 연동 및 실행 파일 생성 ⏳
- **Phase 4**: CLI build 명령어 통합 ⏳
- **Phase 5**: 테스트 및 문서화 ⏳

## 개요

v0.4는 LLVM을 통한 네이티브 코드 생성으로 BMB 프로그램을 실행 파일로 컴파일합니다.

## 기술 스택

| 구성요소 | 라이브러리 | 버전 | 용도 |
|----------|-----------|------|------|
| LLVM 래퍼 | inkwell | 0.5+ | 안전한 LLVM API |
| LLVM | llvm-sys | 18.x | LLVM C 바인딩 |
| 링커 | system linker | - | lld/ld/link.exe |

## 아키텍처

```
┌─────────┐     ┌─────────┐     ┌──────────┐     ┌──────────┐
│   AST   │ ──▶ │   MIR   │ ──▶ │ LLVM IR  │ ──▶ │  Binary  │
└─────────┘     └─────────┘     └──────────┘     └──────────┘
   Parser        Lowering        Codegen          Linker
```

## Phase 1: MIR 설계

### MIR (Middle Intermediate Representation)

```rust
/// MIR 프로그램
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
}

/// MIR 함수
pub struct MirFunction {
    pub name: String,
    pub params: Vec<(String, MirType)>,
    pub ret_ty: MirType,
    pub blocks: Vec<BasicBlock>,
    pub locals: Vec<(String, MirType)>,
}

/// 기본 블록
pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<MirInst>,
    pub terminator: Terminator,
}

/// MIR 명령어
pub enum MirInst {
    /// %dest = const value
    Const { dest: Place, value: Constant },
    /// %dest = %src
    Copy { dest: Place, src: Place },
    /// %dest = %lhs op %rhs
    BinOp { dest: Place, op: BinOp, lhs: Operand, rhs: Operand },
    /// %dest = op %src
    UnaryOp { dest: Place, op: UnaryOp, src: Operand },
    /// %dest = call func(args...)
    Call { dest: Option<Place>, func: String, args: Vec<Operand> },
}

/// 종결자
pub enum Terminator {
    /// return %value
    Return(Option<Operand>),
    /// goto label
    Goto(String),
    /// if %cond then label1 else label2
    Branch { cond: Operand, then_label: String, else_label: String },
}

/// 피연산자
pub enum Operand {
    Place(Place),
    Constant(Constant),
}

/// 장소 (변수/임시값)
pub struct Place {
    pub name: String,
}

/// 상수
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    Unit,
}

/// MIR 타입
pub enum MirType {
    I64,
    F64,
    Bool,
    Unit,
}
```

### AST → MIR 변환

| AST | MIR |
|-----|-----|
| IntLit(n) | Const { dest, Int(n) } |
| BoolLit(b) | Const { dest, Bool(b) } |
| Var(x) | Copy { dest, src: x } |
| Binary | BinOp { dest, op, lhs, rhs } |
| If | Branch + Goto + Phi |
| Let | Const/BinOp + 로컬 변수 |
| Call | Call { dest, func, args } |
| Block | 순차 명령어 |

## Phase 2: LLVM IR 생성

### inkwell 구조

```rust
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::types::*;
use inkwell::values::*;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    // 심볼 테이블
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
}
```

### 타입 매핑

| BMB Type | LLVM Type |
|----------|-----------|
| i64 | i64 |
| f64 | double |
| bool | i1 |
| () | void |

### 코드 생성 규칙

```rust
// 정수 상수
fn gen_int(&self, n: i64) -> IntValue {
    self.context.i64_type().const_int(n as u64, true)
}

// 이항 연산
fn gen_binop(&self, op: BinOp, lhs: IntValue, rhs: IntValue) -> IntValue {
    match op {
        BinOp::Add => self.builder.build_int_add(lhs, rhs, "add"),
        BinOp::Sub => self.builder.build_int_sub(lhs, rhs, "sub"),
        BinOp::Mul => self.builder.build_int_mul(lhs, rhs, "mul"),
        BinOp::Div => self.builder.build_int_signed_div(lhs, rhs, "div"),
        // ...
    }
}

// 함수 호출
fn gen_call(&self, func: FunctionValue, args: &[BasicValueEnum]) -> BasicValueEnum {
    self.builder.build_call(func, args, "call").try_as_basic_value()
}
```

## Phase 3: 링커 연동

### 빌드 파이프라인

```
1. BMB Source (.bmb)
   ↓ parse
2. AST
   ↓ type check
3. Typed AST
   ↓ lower
4. MIR
   ↓ codegen
5. LLVM IR
   ↓ optimize (opt level)
6. Object File (.o)
   ↓ link
7. Executable
```

### 링커 설정

```rust
pub struct BuildConfig {
    pub target_triple: String,      // x86_64-pc-windows-msvc
    pub opt_level: OptLevel,        // Debug, Release
    pub output_type: OutputType,    // Executable, Library
    pub output_path: PathBuf,
}

pub enum OptLevel {
    Debug,      // -O0
    Release,    // -O2
    Size,       // -Os
    Aggressive, // -O3
}
```

### 런타임 지원

```c
// runtime.c (최소 런타임)
#include <stdio.h>
#include <stdlib.h>

void bmb_print_i64(long long x) {
    printf("%lld", x);
}

void bmb_println_i64(long long x) {
    printf("%lld\n", x);
}

void bmb_assert(int cond) {
    if (!cond) {
        fprintf(stderr, "Assertion failed\n");
        exit(1);
    }
}
```

## Phase 4: CLI 통합

### 새로운 명령어

```bash
# 빌드
bmb build <file>              # 기본 빌드 (debug)
bmb build <file> --release    # 릴리스 빌드
bmb build <file> -o <output>  # 출력 경로 지정

# 실행 (빌드 후 실행)
bmb run <file>                # 인터프리터 실행 (기존)
bmb run <file> --native       # 네이티브 빌드 후 실행
```

### CLI 구조

```rust
#[derive(Subcommand)]
enum Command {
    /// Build native executable
    Build {
        file: PathBuf,
        #[arg(long)]
        release: bool,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    // ... 기존 명령어
}
```

## Phase 5: 테스트

### 테스트 케이스

| 카테고리 | 테스트 수 | 예시 |
|----------|-----------|------|
| MIR 생성 | 10 | AST → MIR 변환 |
| 코드젠 | 15 | MIR → LLVM IR |
| 빌드 | 10 | 전체 파이프라인 |
| 런타임 | 10 | 내장 함수 호출 |

### 검증 방법

```bash
# 빌드 테스트
cargo test --features llvm

# 통합 테스트
bmb build examples/hello.bmb -o hello
./hello
# Expected: 42
```

## 프로젝트 구조

```
bmb/src/
├── mir/
│   ├── mod.rs          # MIR 정의
│   └── lower.rs        # AST → MIR 변환
├── codegen/
│   ├── mod.rs          # 코드젠 진입점
│   ├── context.rs      # LLVM 컨텍스트
│   ├── types.rs        # 타입 변환
│   ├── expr.rs         # 표현식 생성
│   └── builtins.rs     # 런타임 함수 선언
├── build/
│   ├── mod.rs          # 빌드 파이프라인
│   ├── config.rs       # 빌드 설정
│   └── linker.rs       # 링커 연동
runtime/
└── runtime.c           # C 런타임
```

## 의존성

```toml
[dependencies]
inkwell = { version = "0.5", features = ["llvm18-0"] }

[build-dependencies]
cc = "1"  # 런타임 컴파일
```

## 참고 자료

- [inkwell GitHub](https://github.com/TheDan64/inkwell)
- [LLVM Kaleidoscope Tutorial](https://llvm.org/docs/tutorial/)
- [Create Your Own Language](https://createlang.rs/)
