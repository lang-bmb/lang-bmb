# BMB 로드맵 v0.1 → v1.0

> 점진적 난이도 진행 + 완전한 생태계 + BMB 부트스트래핑 + 100+ 패키지 + C/Rust 성능 추월

---

## 설계 원칙

| 원칙 | 설명 | 참고 |
|------|------|------|
| **점진적 진행** | 각 버전 간 난이도 차이 최소화 | Gleam 5년 0.x 여정 |
| **도구 내장** | 별도 설치 없이 `bmb fmt`, `bmb lsp` 동작 | Gleam 방식 |
| **작은 배포** | 큰 기능을 여러 minor 버전으로 분할 | Zig 패턴 |
| **0.x = 실험** | Breaking changes 허용, 1.0 = 안정성 약속 | 모든 언어 공통 |
| **패키지 우선** | 모든 재사용 코드는 gotgan 등록 | 생태계 성장 |
| **성능 증명** | C/Rust 대비 벤치마크로 검증 | 계약 기반 최적화 |

### Non-Negotiable Priorities (타협불가)

| Priority | Principle | Description |
|----------|-----------|-------------|
| **Performance** | Maximum Performance Syntax | Syntax must enable maximum performance without constraints. No syntax limitations constraining performance. |
| **Correctness** | Compile-Time Verification | If compile-time checking is possible, it MUST be in the language spec. All verifiable invariants should be checked statically. |

### 버전 체계

```
v0.MAJOR.MINOR
  │      │
  │      └── 작은 개선, 버그 수정, 기능 추가
  └────────── 주요 마일스톤 (Seed, Sprout, Root, ...)
```

---

## 개요

| 버전 | 코드명 | 목표 | 상태 |
|------|--------|------|------|
| v0.1 | Seed | 최소 파서 + 타입체커 | ✅ 완료 |
| v0.2 | Sprout | SMT 연동 + 기본 검증 | ✅ 완료 |
| v0.3 | Root | 인터프리터 + REPL | ✅ 완료 |
| v0.4 | Stem | 코드젠 (LLVM) | ✅ 완료 |
| v0.5 | Branch | 언어 확장 + Bootstrap 시작 | ✅ 완료 |
| v0.6 | Leaf | 표준 라이브러리 기초 (100+개 함수) | ✅ 완료 |
| v0.7 | Bloom | 도구 기초 (fmt, lsp, test, action-bmb) | ✅ 완료 |
| v0.8 | Fruit | 패키지 매니저 (곳간) | ✅ 완료 |
| v0.9 | Harvest | 생태계 (에디터, 원격 패키지) | ✅ 완료 |
| v0.10 | Sunrise | Bootstrap + 컴포넌트 패키지화 | ✅ 완료 |
| v0.11 | Dawn | AI-Native gotgan + Bootstrap | 🔄 진행중 (v0.11.4-7 ✅) |
| v0.12 | Horizon | WASM 듀얼 타깃 | ✅ 완료 (v0.12.0-4) |
| v0.13 | **Forge** | 언어 완성 + extern fn + 제네릭 + 에러처리 | ✅ 완료 (v0.13.0-3) |
| v0.14 | **Foundation** | 제네릭 stdlib + 패키지 표준화 | ✅ 완료 (v0.14.0-5) |
| v0.15 | **Generics** | 제네릭 타입 시스템 완성 | ✅ 완료 (v0.15.0-2) |
| v0.16 | **Consolidate** | 제네릭 enum/struct 타입 체커 완성 | ✅ 완료 (v0.16.0-3) |
| v0.17 | **Module** | 모듈 시스템 + 패키지 간 타입 참조 | ✅ 완료 (v0.17.0-3) |
| v0.18 | **Methods** | Option/Result 메서드 호출 구문 | ✅ 완료 (v0.18.0) |
| v0.19 | **Complete** | MIR Completion (Struct/Enum/Pattern) | ✅ 완료 (v0.19.0-5) |
| v0.20 | **Extend** | Language Extensions (Closures/Traits) | 🔄 진행중 (v0.20.0 ✅) |
| v0.21 | **Bootstrap** | Bootstrap Enhancement (Struct/Enum MIR) | 계획 |
| v0.22 | **Mirror** | Self-Hosting (Stage 1/2/3 Verification) | 계획 |
| v0.23 | **Showcase** | 주요 앱 시나리오 샘플 10개 | 계획 |
| v0.24 | **Launch** | 프로덕션 서비스 런칭 | 계획 |
| v0.25 | **Velocity** | C/Rust 성능 추월 스프린트 | 계획 |
| v0.26 | **Query** | AI Query System (RFC-0001) | 계획 |
| v1.0-RC | **Golden** | 최종 검증 + 안정성 약속 | 계획 |

---

## 생태계 레포지토리

| 레포지토리 | 용도 | Rust 버전 | BMB 재작성 | 서비스 런칭 |
|------------|------|-----------|------------|-------------|
| lang-bmb | 메인 컴파일러 | v0.1 ✅ | v0.22 | - |
| gotgan | 패키지 매니저 | v0.8 ✅ | v0.22 | gotgan.bmb.dev |
| action-bmb | GitHub Action | v0.7 ✅ | v0.22 | - |
| bmb-samples | 예제 프로그램 | - | v0.23 | - |
| benchmark-bmb | 표준 벤치마크 | v0.9 ✅ | v0.15 | bench.bmb.dev |
| playground | 온라인 플레이그라운드 | - | v0.24 | play.bmb.dev |
| lang-bmb-site | 공식 웹사이트 | - | v0.24 | bmb.dev |

---

## 패키지 생태계 목표 (115개)

### 카테고리별 패키지 목록

| 카테고리 | 수량 | 버전 | 핵심 패키지 |
|----------|------|------|-------------|
| Core/Foundation | 20 | v0.14 | bmb-core, bmb-iter, bmb-hash, bmb-fmt |
| Collections | 15 | v0.14-15 | bmb-vec, bmb-hashmap, bmb-btreemap |
| IO/Filesystem | 10 | v0.15 | bmb-io, bmb-fs, bmb-path, bmb-tar |
| Networking | 15 | v0.16 | bmb-http, bmb-websocket, bmb-grpc |
| Serialization | 10 | v0.16 | bmb-serde, bmb-json, bmb-toml |
| Async | 10 | v0.17 | bmb-async, bmb-future, bmb-channel |
| Crypto/Security | 10 | v0.17 | bmb-crypto, bmb-sha, bmb-aes |
| Database | 10 | v0.18 | bmb-sql, bmb-postgres, bmb-redis |
| CLI/Tools | 10 | v0.18 | bmb-clap, bmb-log, bmb-config |
| Testing/Dev | 5 | v0.18 | bmb-test, bmb-bench, bmb-mock |

**총합: 115개 패키지 + 115개 샘플 앱**

---

## 벤치마크 KPI

| 지표 | v0.15 목표 | v0.17 목표 | v0.22 목표 |
|------|------------|------------|------------|
| 컴파일 속도 | Rust 80% | Rust 90% | Rust 100%+ |
| 런타임 성능 | C 70% | C 85% | C 100%+ |
| 바이너리 크기 | Rust 120% | Rust 100% | Rust 90% |
| 메모리 사용량 | Rust 110% | Rust 100% | Rust 95% |
| WASM 크기 | - | 기준선 | 최적화 |

### 벤치마크 스위트

```
benchmark-bmb/
├── micro/              # 마이크로 벤치마크
│   ├── fibonacci       # 재귀 성능
│   ├── primes          # 소수 계산
│   ├── sorting         # 정렬 알고리즘
│   └── hashing         # 해시 성능
├── algo/               # 알고리즘 벤치마크
│   ├── graph           # 그래프 알고리즘
│   ├── string          # 문자열 처리
│   └── numeric         # 수치 계산
├── real/               # 실제 워크로드
│   ├── json-parse      # JSON 파싱
│   ├── http-server     # HTTP 서버
│   └── db-query        # DB 쿼리
└── compare/            # C/Rust 비교
    ├── c/
    ├── rust/
    └── bmb/
```

---

## 부트스트래핑 전략 (확장)

```
Phase 1 (v0.1-v0.3): Rust로 기반 구축
  - 컴파일러 프론트엔드 (Rust)
  - 인터프리터/REPL (Rust)

Phase 2 (v0.4-v0.5): 네이티브 코드 생성
  - LLVM 백엔드 (Rust)
  - Bootstrap 시작

Phase 3 (v0.6-v0.7): 표준 라이브러리 + 도구
  - 표준 라이브러리 기초
  - 내장 도구: fmt, lsp, test

Phase 4 (v0.8-v0.9): 패키지 매니저 + 생태계
  - gotgan 패키지 매니저 (Rust)
  - 에디터, 플레이그라운드, 웹사이트

Phase 5 (v0.10-v0.12): WASM + 듀얼 타깃
  - WASM 백엔드
  - 런타임 바인딩

Phase 6 (v0.13-v0.18): 패키지 생태계 구축
  - 115개 패키지 개발
  - 각 패키지 샘플 앱
  - gotgan 레지스트리 등록
  - 벤치마크 + 최적화 반복

Phase 7 (v0.19): MIR Completion ★ COMPLETED
  - Struct/Enum MIR lowering 완성 ✅
  - Pattern matching 완전 구현 ✅
  - Array/Method dispatch 구현 ✅

Phase 8 (v0.20): Language Extensions ★ REVISED
  - Closures (람다 문법, 캡처 의미론)
  - Traits (trait 키워드, impl 블록)
  - FFI Enhancement (extern "C" 링킹)

Phase 9 (v0.21): Bootstrap Enhancement ★ REVISED
  - Bootstrap에 struct/enum 지원 추가
  - Bootstrap-Rust compiler 동등성 테스트

Phase 10 (v0.22): Self-Hosting ★ REVISED
  - 컴파일러 BMB 재작성
  - gotgan BMB 재작성
  - Stage 1/2/3 자기 컴파일 검증

Phase 11 (v0.23-v0.24): 프로덕션 런칭
  - 주요 앱 시나리오 샘플
  - 서브모듈 서비스 런칭

Phase 12 (v0.25): 성능 스프린트
  - C/Rust 성능 추월
  - 계약 기반 최적화

Phase 13 (v0.26): AI Query System ★ RFC-0001
  - 컴파일 부산물 인덱스 생성
  - AI 도구용 쿼리 인터페이스 (bmb q)
  - 계약/증명 상태 기반 코드 탐색

Phase 14 (v1.0-RC): Golden Release
  - 전체 검증
  - 안정성 약속
```

---

## v0.1 Seed ✅ (최소 기반)

### 목표
```
Rust로 작성된 최소 컴파일러 프론트엔드
```

### 구현

| 구성요소 | 기술 | 상태 |
|----------|------|------|
| 렉서 | logos | ✅ |
| 파서 | lalrpop | ✅ |
| AST | 수동 정의 | ✅ |
| CLI | clap | ✅ |

---

## v0.2 Sprout ✅ (SMT 연동)

### 구현

| 구성요소 | 기술 | 상태 |
|----------|------|------|
| 타입 체커 | 수동 구현 | ✅ |
| SMT 변환기 | SMT-LIB 생성 | ✅ |
| Z3 연동 | z3 CLI | ✅ |
| 에러 보고 | ariadne | ✅ |

---

## v0.3 Root ✅ (인터프리터)

### 구현

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| 트리워킹 인터프리터 | AST 직접 실행 | ✅ |
| REPL | rustyline 기반 | ✅ |
| 런타임 에러 | 스택 트레이스 | ✅ |

---

## v0.4 Stem ✅ (LLVM 코드젠)

### 구현

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| MIR | CFG 기반 중간 표현 | ✅ |
| LLVM IR 생성 | 텍스트 기반 | ✅ |
| 네이티브 빌드 | LLVM toolchain | ✅ |

---

## v0.5 Branch ✅ (언어 확장)

### 구현

| 기능 | 설명 | 상태 |
|------|------|------|
| 패턴 매칭 | match 표현식 | ✅ |
| 제네릭 기초 | 타입 파라미터 | ✅ |
| 모듈 시스템 | use/mod | ✅ |
| 속성 | @attr 문법 | ✅ |

---

## v0.6 Leaf ✅ (표준 라이브러리)

### 구현 (100+ 함수)

| 모듈 | 함수 수 | 상태 |
|------|---------|------|
| core | 50+ | ✅ |
| string | 25+ | ✅ |
| math | 30+ | ✅ |
| io | 10+ | ✅ |

---

## v0.7 Bloom ✅ (도구)

### 구현

| 도구 | 설명 | 상태 |
|------|------|------|
| bmb fmt | 코드 포맷터 | ✅ |
| bmb lsp | 언어 서버 | ✅ |
| bmb test | 테스트 러너 | ✅ |
| action-bmb | GitHub Action | ✅ |

---

## v0.8 Fruit ✅ (패키지 매니저)

### 구현

| 기능 | 설명 | 상태 |
|------|------|------|
| gotgan init | 프로젝트 생성 | ✅ |
| gotgan build | 빌드 | ✅ |
| gotgan add | 의존성 추가 | ✅ |
| 의존성 해결 | SAT 솔버 | ✅ |

---

## v0.9 Harvest ✅ (생태계)

### 구현

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| tree-sitter-bmb | 에디터 문법 | ✅ |
| vscode-bmb | VS Code 확장 | ✅ |
| playground | 온라인 실행 | ✅ |
| lang-bmb-site | 웹사이트 | ✅ |

---

## v0.10 Sunrise ✅ (컴포넌트 패키지화)

### 구현

| 패키지 | 설명 | 상태 |
|--------|------|------|
| bmb-lexer | 렉서 라이브러리 | ✅ |
| bmb-parser | 파서 라이브러리 | ✅ |
| bmb-types | 타입 시스템 | ✅ |
| bmb-smt | SMT 변환 | ✅ |

---

## v0.11 Dawn (AI-Native gotgan)

> 목표: AI-네이티브 패키지 매니저 기능

### v0.11.0-3 - BMB 부트스트랩 (차단됨)

**상태:** BMB 코드젠이 완성되어야 실행 가능

### v0.11.4 - BMBX 번들 포맷 ✅

**AI-Native Package Bundle:**
```
package.bmbx
├── manifest.toml      # 패키지 메타데이터
├── contracts.json     # 모든 계약의 JSON 표현
├── symbols.json       # AI 탐색용 심볼 인덱스
├── types.json         # 타입 시그니처
├── src/               # 소스 코드
└── bin/               # 컴파일된 바이너리
```

### v0.11.5 - 계약 기반 의존성 검사 ✅

| 변경 | 타입 | 설명 |
|------|------|------|
| pre 제거 | ✅ 허용 | 더 관대해짐 |
| pre 추가 | ⚠️ Breaking | 더 제한적 |
| post 추가 | ✅ 허용 | 더 많은 보장 |
| post 제거 | ⚠️ Breaking | 보장 감소 |

### v0.11.6 - AI 패키지 탐색 ✅

```bash
$ gotgan explore --symbols --json
$ gotgan explore --contracts --filter "parse"
```

### v0.11.7 - 단일 파일 번들 ✅

```bash
$ gotgan bundle --single-file
✓ Generated: "target/bmbx/package-0.1.0.bmbx"
```

---

## v0.12 Horizon (WASM 듀얼 타깃)

> 목표: LLVM 네이티브 + WASM 포터블 동시 지원

### v0.12.0 - MIR → WASM 변환기 ✅

```
MIR (공통 중간 표현)
    ↓
    ├─────────────────────────────┐
    ↓                             ↓
LLVM IR Generator           WASM IR Generator
    ↓                             ↓
Native Binary               .wasm
```

**CLI:**
```bash
$ bmb build add.bmb --emit-wasm --wasm-target=wasi
```

### v0.12.1 - WASI 런타임 바인딩 ✅

```wat
(func $println (param $val i64))  ;; stdout 출력
(func $print (param $val i64))    ;; 개행 없음
(func $exit (param $code i32))    ;; 프로세스 종료
(func $assert (param $cond i32))  ;; 검증
```

### v0.12.2 - 브라우저 런타임 바인딩 ✅

```wat
(func $println (param $val i64))  ;; console.log
(func $exit (param $code i32))    ;; unreachable
(func $assert (param $cond i32))  ;; unreachable
```

### v0.12.3 - 조건부 컴파일 ✅

```bmb
-- target == 비교 연산자 사용 (= 아닌 ==)
@cfg(target == "wasm32")
fn wasm_print(s: i64) = wasm_console_log(s);

@cfg(target == "native")
fn native_print(s: i64) = libc_puts(s);
```

**구현:**
- `cfg` 모듈: `CfgEvaluator`, `Target` enum
- AST 필터링: 타입 체크 전 @cfg 평가
- 지원 타깃: `native`, `wasm32`, `wasm64`

### v0.12.4 - 듀얼 타깃 빌드 ✅

```bash
$ bmb build app.bmb --all-targets --verbose
=== Native Build ===
  Parsed 4 items
  After @cfg filtering: 3 items (target: native)
=== WASM Build ===
  Parsed 4 items
  After @cfg filtering: 3 items (target: wasm32)
=== All targets built successfully! ===
```

**구현:**
- `--all-targets` CLI 플래그
- 네이티브 + WASM 동시 빌드
- 타깃별 @cfg 필터링

---

## v0.13 Forge (언어 완성) ✅

> 목표: Self-hosting과 패키지 개발에 필요한 언어 기능 완성

### v0.13.0 - extern fn 지원 ✅

```bmb
-- 외부 함수 선언
extern fn malloc(size: usize) -> *mut u8;
extern fn free(ptr: *mut u8);

-- WASI 바인딩
@wasi
extern fn fd_write(fd: i32, iovs: i32, len: i32, nwritten: *mut i32) -> i32;
```

**구현:**
- `extern fn` 문법 파싱 및 AST 표현
- WASI 바인딩을 위한 `@wasi` 속성 지원

### v0.13.1 - 제네릭 기초 ✅

```bmb
-- 제네릭 함수
fn identity<T>(x: T) -> T = x;

-- 제네릭 구조체
struct Container<T> {
    value: T,
}

-- 제네릭 열거형
enum Option<T> {
    Some(T),
    None,
}
```

**구현:**
- 타입 파라미터 파싱 (`<T>`, `<T, U>`)
- 제네릭 함수, 구조체, 열거형 지원
- 타입 파라미터 인스턴스화

### v0.13.2 - 에러 처리 (? 연산자 + try 블록) ✅

```bmb
-- ? 연산자로 에러 전파
fn compute_with_question(x: i64) -> i64 = {
    let a: i64 = divide(x, 2)?;
    a * 2
};

-- try 블록으로 에러 캡처
fn safe_compute(x: i64) -> i64 = {
    let result: i64 = try {
        divide(x, 2)
    };
    result
};

-- 체이닝
fn chained_operations(x: i64) -> i64 = {
    let a: i64 = divide(x, 2)?;
    let b: i64 = divide(a, 2)?;
    b
};
```

**구현:**
- `?` 연산자 (Question) 파싱 및 후위 표현식 지원
- `try { ... }` 블록 파싱
- 인터프리터, 타입 체커, SMT 변환기 연동

### v0.13.3 - @derive 속성 매크로 ✅

```bmb
-- 단일 derive
@derive(Debug, Clone)
struct Point {
    x: i64,
    y: i64,
}

-- 다중 트레이트
@derive(Debug, Clone, PartialEq, Eq)
struct Color {
    r: i64,
    g: i64,
    b: i64,
}

-- 제네릭과 결합
@derive(Debug, Clone)
struct Container<T> {
    value: T,
}

-- 열거형 지원
@derive(Debug, Clone, PartialEq)
enum Status {
    Active,
    Inactive,
    Pending,
}
```

**구현:**
- `bmb/src/derive/mod.rs` 모듈 추가
- `DeriveTrait` 열거형: Debug, Clone, PartialEq, Eq, Default, Hash
- `extract_derive_traits()`: 속성에서 트레이트 추출
- `DeriveContext`: 코드 생성용 컨텍스트
- 구조체/열거형 derive 속성 지원

---

## v0.14 Foundation (제네릭 stdlib + 패키지 표준화) ✅

> 목표: 기존 모노모픽 stdlib을 제네릭으로 업그레이드 + 패키지 구조 표준화

### 설계 원칙 적용

v0.14 계획 검토 시 다음 원칙을 적용하여 범위를 재조정:

| 원칙 | 적용 |
|------|------|
| **점진적 진행** | 25개 → 5개 핵심 패키지로 축소 |
| **작은 배포** | 런타임 의존 패키지(alloc, sync 등) 제외 |
| **현실적 범위** | 이미 구현된 기능의 제네릭화에 집중 |

### v0.14.0 - 패키지 구조 표준화 ✅

```
packages/
├── README.md               # 패키지 표준 문서
├── bmb-core/
│   ├── Gotgan.toml         # 패키지 매니페스트
│   └── src/lib.bmb         # 소스 코드
├── bmb-option/
├── bmb-result/
├── bmb-traits/
└── bmb-iter/
```

**Gotgan.toml 표준:**
```toml
[package]
name = "bmb-core"
version = "0.14.0"
description = "Core types and primitives for BMB"
license = "MIT OR Apache-2.0"

[dependencies]
# 의존성 선언

[contracts]
verify = true
```

### v0.14.1 - Option<T> 제네릭화 ✅

```bmb
@derive(Debug, Clone, PartialEq)
pub enum Option<T> {
    Some(T),
    None,
}

pub fn is_some<T>(opt: Option<T>) -> bool =
    match opt {
        Option::Some(v) => true,
        Option::None => false,
    };

pub fn unwrap_or<T>(opt: Option<T>, default: T) -> T =
    match opt {
        Option::Some(v) => v,
        Option::None => default,
    };
```

**포함 기능:**
- 제네릭 `Option<T>` 열거형
- `some<T>`, `none<T>` 생성자
- `is_some`, `is_none` 상태 검사
- `unwrap_or`, `unwrap_or_default` 추출
- `option_or`, `option_and`, `option_xor` 결합
- `filter_by_bool` 필터링
- i64 특화 역호환성 함수

### v0.14.2 - Result<T, E> 제네릭화 ✅

```bmb
@derive(Debug, Clone, PartialEq)
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

pub fn safe_divide(a: i64, b: i64) -> Result<i64, i64>
  post (b == 0 and is_err(ret)) or (b != 0 and is_ok(ret))
= if b == 0 then Result::Err(100) else Result::Ok(a / b);
```

**포함 기능:**
- 제네릭 `Result<T, E>` 열거형
- `ok<T, E>`, `err<T, E>` 생성자
- `is_ok`, `is_err` 상태 검사
- `unwrap_or`, `unwrap`, `unwrap_err` 추출
- `ok_option`, `err_option` Option 변환
- `result_or`, `result_and` 결합
- `ErrorCode` 표준 에러 열거형
- `safe_divide`, `safe_mod` 안전 연산
- i64 특화 역호환성 함수

### v0.14.3 - 트레잇 정의 ✅

```bmb
-- 트레잇 명세 (BMB는 아직 trait 키워드 미지원)
-- @derive에서 사용되는 트레잇 계약 정의

@derive(Debug, Clone, PartialEq, Eq)
pub enum Ordering {
    Less,
    Equal,
    Greater,
}
```

**정의된 트레잇:**
| 트레잇 | 설명 | @derive 지원 |
|--------|------|--------------|
| Debug | 디버그 출력 | ✅ |
| Clone | 복제 | ✅ |
| PartialEq | 부분 동등성 | ✅ |
| Eq | 완전 동등성 | ✅ |
| PartialOrd | 부분 순서 | ⬜ 계획 |
| Ord | 완전 순서 | ⬜ 계획 |
| Default | 기본값 | ✅ |
| Hash | 해시 | ✅ |

### v0.14.4 - Iterator 트레잇 및 기본 구현 ✅

```bmb
@derive(Debug, Clone, PartialEq)
pub struct Range {
    current: i64,
    end: i64,
    step: i64,
}

pub fn range(start: i64, end: i64) -> Range
  pre start <= end
= new Range { current: start, end: end, step: 1 };

pub fn fibonacci(n: i64) -> i64
  pre n >= 0
= if n == 0 then 0
  else if n == 1 then 1
  else fibonacci(n - 1) + fibonacci(n - 2);
```

**포함 기능:**
- `Range` 이터레이터 (i64 범위)
- `Repeat<T>` 무한 반복
- `Take` n개 제한
- 콤비네이터: `range_sum`, `range_product`, `range_count`
- `range_min`, `range_max`, `range_any_positive`, `range_all_positive`
- `enumerate_at`, `zip_ranges_at` 연쇄 패턴
- `nth`, `last` 수집 함수
- `naturals_nth`, `fibonacci` 무한 시퀀스

### v0.14.5 - bmb-core 통합 ✅

```bmb
@derive(Debug, Clone, PartialEq)
pub struct Pair<A, B> {
    fst: A,
    snd: B,
}

pub fn identity<T>(x: T) -> T = x;

pub fn abs(x: i64) -> i64
  post ret >= 0 and ((x >= 0 and ret == x) or (x < 0 and ret == 0 - x))
= if x >= 0 then x else 0 - x;
```

**포함 기능:**
- `Unit`, `Never` 특수 타입
- `bool_and`, `bool_or`, `bool_not`, `bool_xor` 불리언 연산
- `abs`, `min`, `max`, `clamp`, `sign` 수치 연산
- `in_range`, `diff` 범위/차이
- `Pair<A, B>` 제네릭 페어
- `identity<T>`, `swap<T>` 유틸리티

### 검증 결과

```bash
$ cargo test
running 15 tests
test tests::test_extern_fn ... ok
test tests::test_generics ... ok
test tests::test_derive ... ok
...
test result: ok. 15 passed; 0 failed
```

**파서 검증:**
```bash
$ bmb parse packages/bmb-core/src/lib.bmb     # ✅
$ bmb parse packages/bmb-option/src/lib.bmb   # ✅
$ bmb parse packages/bmb-result/src/lib.bmb   # ✅
$ bmb parse packages/bmb-traits/src/lib.bmb   # ✅
$ bmb parse packages/bmb-iter/src/lib.bmb     # ✅
```

### 다음 단계 (v0.15+)

| 패키지 | 버전 | 의존성 |
|--------|------|--------|
| bmb-vec | v0.15 | 런타임 alloc 필요 |
| bmb-hashmap | v0.15 | 런타임 alloc 필요 |
| bmb-io | v0.15 | extern fn 확장 필요 |
| bmb-async | v0.17 | 런타임 지원 필요 |

---

## v0.15 Generics (제네릭 타입 시스템 완성)

> 목표: v0.13에서 추가된 제네릭 **문법**을 완전한 **의미론**으로 구현

### 배경

v0.13에서 제네릭 문법이 추가되었으나 (TypeParam, Generic types), 타입 체커에서 실제 타입 검증이 누락되어 있었음:
- `identity<T>(x: T) -> T` 정의는 파싱되지만
- `identity(100)` 호출 시 "expected T, got i64" 에러 발생
- 원인: 타입 추론 및 대입(substitution) 로직 부재

### 구현 내용

#### v0.15.0 - 타입 체커 제네릭 지원

| 기능 | 설명 | 상태 |
|------|------|------|
| TypeChecker 확장 | `generic_functions`, `generic_structs`, `type_param_env` 필드 추가 | ✅ |
| 타입 매개변수 환경 | 함수/구조체 내 타입 매개변수 추적 | ✅ |
| Named → TypeVar 변환 | `resolve_type_vars()` - 파서의 Named 타입을 TypeVar로 변환 | ✅ |
| 타입 인자 추론 | `infer_type_args()` - 호출 인자로부터 타입 인자 추론 | ✅ |
| 타입 대입 | `substitute_type()` - TypeVar를 구체 타입으로 대체 | ✅ |
| unify 확장 | TypeVar 처리 추가 | ✅ |

#### v0.15.1 - 인터프리터 제네릭 지원

| 기능 | 설명 | 상태 |
|------|------|------|
| 동적 타이핑 활용 | 인터프리터는 런타임에 타입 정보 불필요 | ✅ 기존 동작 |
| 제네릭 함수 호출 | 타입 체커 통과 후 정상 실행 | ✅ |

#### v0.15.2 - 통합 테스트

| 테스트 파일 | 테스트 내용 | 상태 |
|-------------|-------------|------|
| `test_generics_simple.bmb` | 기본 identity 함수 | ✅ |
| `test_generics_runtime.bmb` | 제네릭 + 비제네릭 혼합 | ✅ |
| `test_generics_comprehensive.bmb` | 다중 타입 매개변수, 중첩 호출 | ✅ |
| `test_generics_stdlib.bmb` | Option 패턴, 제네릭 enum | ⚠️ 제한적 |

### 알려진 제한사항

| 제한 | 설명 | 해결 버전 |
|------|------|-----------|
| 제네릭 enum 인스턴스화 | `Option::Some(v)`가 `Option`으로 타입 추론됨 (`Option<T>` 아님) | v0.16+ |
| 제네릭 struct 생성자 | 구조체 리터럴 타입 추론 미구현 | v0.16+ |
| 타입 인자 명시 구문 | `identity::<i64>(100)` 미지원 | 필요시 |

### 기술적 세부사항

**핵심 타입 체커 변경 (`bmb/src/types/mod.rs`):**

```rust
// 새로운 필드
generic_functions: HashMap<String, (Vec<TypeParam>, Vec<Type>, Type)>,
generic_structs: HashMap<String, (Vec<TypeParam>, Vec<(String, Type)>)>,
type_param_env: HashMap<String, Vec<String>>,

// 핵심 메서드
fn resolve_type_vars(&self, ty: &Type, type_param_names: &[&str]) -> Type
fn infer_type_args(&self, param_ty: &Type, arg_ty: &Type, subst: &mut HashMap<String, Type>, span: Span) -> Result<()>
fn substitute_type(&self, ty: &Type, subst: &HashMap<String, Type>) -> Type
```

**파서-타입체커 인터페이스:**
- 파서는 `Type::Named("T")`를 생성
- 타입 체커가 타입 매개변수 이름과 매칭하여 `Type::TypeVar("T")`로 변환
- 호출 시점에 구체 타입으로 대입하여 단형화(monomorphization)

### 테스트 결과

```
cargo test -- --nocapture
running 15 tests
test lexer::tests::... ok
test parser::tests::... ok
test types::tests::... ok
...
test result: ok. 15 passed
```

통합 테스트:
```bash
cargo run check tests/examples/valid/test_generics_comprehensive.bmb  # ✅
cargo run run tests/examples/valid/test_generics_comprehensive.bmb    # 출력: 230
```

---

## v0.16 Consolidate (제네릭 enum/struct 타입 체커 완성) ✅

> 목표: v0.15에서 구현된 제네릭 함수 지원을 enum/struct로 확장하여 stdlib 패키지 타입 체크 가능

### 배경

v0.15에서 제네릭 함수 타입 추론이 완성되었으나, 제네릭 enum/struct 인스턴스화에서 문제 발견:
- `Option::Some(42)` → `Option` 타입으로 추론됨 (`Option<i64>` 아님)
- bmb-option, bmb-result 패키지 타입 체크 실패
- 원인: 제네릭 enum/struct에 대한 타입 추론 로직 부재

### 설계 원칙 적용

| 원칙 | 적용 |
|------|------|
| **비판적 분석** | 원래 v0.16 계획(25개 네트워크 패키지) 검토 → 현실적 불가능 판단 |
| **점진적 진행** | 타입 시스템 완성 → 패키지 개발 순서 유지 |
| **기초 우선** | Option/Result 없이 네트워크 패키지 개발 불가 |

### v0.16.0 - generic_enums 필드 추가 ✅

```rust
/// Generic enum definitions: name -> (type_params, variants)
/// v0.16: Support for generic enums like enum Option<T> { Some(T), None }
generic_enums: HashMap<String, (Vec<TypeParam>, Vec<(String, Vec<Type>)>)>,
```

### v0.16.1 - EnumDef 제네릭 등록 ✅

```rust
Item::EnumDef(e) => {
    let variants = e.variants.iter()
        .map(|v| (v.name.node.clone(), v.fields.iter().map(|f| f.node.clone()).collect()))
        .collect();
    // v0.16: Handle generic enums separately
    if e.type_params.is_empty() {
        self.enums.insert(e.name.node.clone(), variants);
    } else {
        self.generic_enums.insert(e.name.node.clone(), (e.type_params.clone(), variants));
    }
}
```

### v0.16.2 - EnumVariant 타입 추론 ✅

| 기능 | 설명 | 상태 |
|------|------|------|
| Expr::EnumVariant | 제네릭 enum 생성자 타입 추론 | ✅ |
| Pattern::EnumVariant | 제네릭 enum 패턴 매칭 | ✅ |
| Type::Generic 처리 | `Option<i64>` 형태의 인스턴스화 타입 | ✅ |
| Type::TypeVar 처리 | nullary variant (None) 타입 추론 | ✅ |
| unify() 확장 | Generic 타입 간 통합, TypeVar 허용 | ✅ |

### v0.16.3 - 제네릭 struct 지원 ✅

| 기능 | 설명 | 상태 |
|------|------|------|
| Expr::StructInit | 제네릭 struct 생성자 타입 추론 | ✅ |
| Expr::FieldAccess | 제네릭 struct 필드 접근 타입 해석 | ✅ |
| 타입 대입 | struct 필드에서 타입 파라미터 치환 | ✅ |

### 패키지 버그 수정 ✅

`bmb-option`과 `bmb-result`에서 발견된 unreachable 브랜치 버그 수정:

```bmb
-- 수정 전: undefined variable 'default'
pub fn unwrap<T>(opt: Option<T>) -> T
  pre is_some(opt)
= match opt {
    Option::Some(v) => v,
    Option::None => default,  -- ❌ 컴파일 에러
};

-- 수정 후: precondition에 의해 도달 불가한 브랜치는 재귀 호출로 타입 맞춤
pub fn unwrap<T>(opt: Option<T>) -> T
  pre is_some(opt)
= match opt {
    Option::Some(v) => v,
    Option::None => unwrap(opt),  -- ✅ 도달 불가 (pre 보장)
};
```

### 검증 결과

```bash
$ cargo test
running 65 tests
...
test result: ok. 65 passed; 0 failed
```

**패키지 타입 체크:**

| 패키지 | 상태 | 비고 |
|--------|------|------|
| test_generics_stdlib.bmb | ✅ | 제네릭 Option 패턴 |
| bmb-option | ✅ | Option<T> 전체 |
| bmb-traits | ✅ | Ordering enum |
| bmb-core | ✅ | Pair<A, B> struct |
| bmb-result | ⚠️ | Option import 필요 (모듈 시스템) |
| bmb-iter | ⚠️ | Option import 필요 (모듈 시스템) |

### 기술적 세부사항

**핵심 타입 체커 변경 (`bmb/src/types/mod.rs`):**

```rust
// v0.16 추가 메서드
fn infer_type_args(&self, param_ty: &Type, arg_ty: &Type,
                   subst: &mut HashMap<String, Type>, span: Span) -> Result<()>

// unify() 확장 - Generic 타입 처리
if let (Type::Generic { name: n1, type_args: a1 },
        Type::Generic { name: n2, type_args: a2 }) = (expected, actual) {
    if n1 == n2 && a1.len() == a2.len() {
        for (arg1, arg2) in a1.iter().zip(a2.iter()) {
            self.unify(arg1, arg2, span)?;
        }
        return Ok(());
    }
}

// TypeVar 허용 (nullary variant 지원)
if let Type::TypeVar(_) = expected { return Ok(()); }
if let Type::TypeVar(_) = actual { return Ok(()); }
```

### 알려진 제한사항

| 제한 | 설명 | 해결 버전 |
|------|------|-----------|
| 모듈 import | 패키지 간 타입 참조 미지원 | v0.17+ |
| 타입 인자 명시 | `func::<i64>()` 구문 미지원 | 필요시 |
| 트레이트 바운드 | `<T: Clone>` 미지원 | v0.18+ |

### 다음 단계 → v0.17 Module ✅

| 영역 | 내용 | 상태 |
|------|------|------|
| 모듈 시스템 | import/use로 패키지 간 타입 참조 | ✅ v0.17 완료 |
| 트레이트 시스템 | 타입 바운드, impl 블록 | v0.18+ 계획 |
| 네트워크 패키지 | bmb-http 등 | 트레이트 시스템 이후 |

---

## v0.17 Module (모듈 시스템 + 패키지 간 타입 참조) ✅

> 목표: 패키지 간 타입 참조 가능하게 모듈 시스템 통합

### 배경

v0.16에서 제네릭 enum/struct 타입 체크가 완성되었으나, 패키지 간 타입 참조에서 문제 발견:
- `bmb-result`가 `bmb-option::Option`을 사용
- `bmb-iter`가 `bmb-option::Option`을 반환
- 타입 체커에서 "undefined enum: Option" 에러 발생
- 원인: TypeChecker가 `Item::Use(_)` 문을 무시

### 설계 원칙 적용

| 원칙 | 적용 |
|------|------|
| **비판적 분석** | 원래 v0.17 계획(20개 Async/Crypto 패키지) 검토 → 모듈 시스템 없이 불가능 |
| **점진적 진행** | 모듈 시스템 → 패키지 확장 순서 유지 |
| **기초 우선** | Option import 없이 Result/Iterator 개발 불가 |

### v0.17.0 - TypeChecker import 연동 ✅

```rust
/// v0.17: Register public items from an imported module
pub fn register_module(&mut self, module: &Module) {
    for item in &module.program.items {
        match item {
            // Register public struct definitions
            Item::StructDef(s) if s.visibility == Visibility::Public => {
                // generic_structs 또는 structs에 등록
            }
            // Register public enum definitions
            Item::EnumDef(e) if e.visibility == Visibility::Public => {
                // generic_enums 또는 enums에 등록
            }
            // Register public function signatures
            Item::FnDef(f) if f.visibility == Visibility::Public => {
                // generic_functions 또는 functions에 등록
            }
            _ => {}
        }
    }
}
```

### v0.17.1 - CLI multi-file 지원 ✅

```bash
# -I 플래그로 include 경로 지정
$ bmb check packages/bmb-result/src/lib.bmb -I packages
✓ packages/bmb-result/src/lib.bmb type checks successfully
```

**구현:**
- `-I` / `--include` CLI 플래그 추가
- `check_file_with_includes()` 함수 구현
- Use 문에서 패키지 경로 추출

### v0.17.2 - 패키지 경로 해석 ✅

```rust
// 언더스코어 → 하이픈 변환
// use bmb_option::Option → packages/bmb-option/src/lib.bmb
let pkg_dir_name = module_name.replace('_', "-");
let module_path = include_path.join(&pkg_dir_name).join("src").join("lib.bmb");
```

### v0.17.3 - v0.14 패키지 타입 체크 ✅

| 패키지 | 상태 | 명령어 |
|--------|------|--------|
| bmb-option | ✅ | `bmb check packages/bmb-option/src/lib.bmb` |
| bmb-traits | ✅ | `bmb check packages/bmb-traits/src/lib.bmb` |
| bmb-core | ✅ | `bmb check packages/bmb-core/src/lib.bmb` |
| bmb-result | ✅ | `bmb check packages/bmb-result/src/lib.bmb -I packages` |
| bmb-iter | ✅ | `bmb check packages/bmb-iter/src/lib.bmb -I packages` |

### 패키지 업데이트

```bmb
-- packages/bmb-result/src/lib.bmb (v0.17.0)
-- v0.17: Import Option for Result-Option conversions
use bmb_option::Option;

-- packages/bmb-iter/src/lib.bmb (v0.17.0)
-- v0.17: Import Option for iterator return types
use bmb_option::Option;
```

### 기술적 세부사항

**핵심 변경 파일:**

| 파일 | 변경 내용 |
|------|-----------|
| `bmb/src/types/mod.rs` | `register_module()` 메서드 추가 |
| `bmb/src/main.rs` | `-I` 플래그 + `check_file_with_includes()` |
| `packages/bmb-result/src/lib.bmb` | `use bmb_option::Option;` 추가 |
| `packages/bmb-iter/src/lib.bmb` | `use bmb_option::Option;` 추가 |

### 알려진 제한사항

| 제한 | 설명 | 해결 버전 |
|------|------|-----------|
| 수동 -I 플래그 | Gotgan.toml 의존성 자동 해석 미지원 | v0.18+ |
| 단일 레벨 import | 중첩 모듈 경로 미지원 | 필요시 |
| 순환 의존성 | 순환 import 감지 미구현 | v0.18+ |

### 다음 단계 (v0.18 Methods ✅ → v0.19+)

| 영역 | 내용 | 상태 |
|------|------|------|
| Option/Result 메서드 | is_some(), is_ok(), unwrap_or() | ✅ v0.18 완료 |
| gotgan 통합 | Gotgan.toml에서 의존성 자동 로드 | v0.19+ |
| 트레이트 시스템 | 타입 바운드, impl 블록 | v0.19+ |
| Async/Crypto 패키지 | 원래 v0.17 계획 패키지들 | 트레이트 시스템 이후 |

---

## v0.18 Methods (Option/Result 메서드 호출 구문) ✅

> 목표: 제네릭 타입(Option, Result)에 대한 메서드 호출 구문 지원

### 배경

원래 v0.18 "Persist" 계획은 20개의 Database/CLI 패키지를 목표로 했으나,
이는 현재 언어 상태에서 비현실적임:
- FFI (extern C 바인딩) 미구현
- Async/Await 미구현
- Vec/String 런타임 미구현

대신, 메서드 호출 구문을 통해 Option/Result 사용성을 개선하는 것이
언어 완성도에 더 중요하다고 판단.

### 구현 내용

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| Option<T> 메서드 | is_some(), is_none(), unwrap_or() | ✅ |
| Result<T,E> 메서드 | is_ok(), is_err(), unwrap_or() | ✅ |
| 타입체커 확장 | check_option_method, check_result_method | ✅ |
| 인터프리터 확장 | eval_method_call에 Option/Result 처리 | ✅ |
| TypeVar 해결 | unwrap_or 반환 타입 추론 수정 | ✅ |

### 사용 예시

```bmb
-- Before (함수 호출)
let x = unwrap_or(opt, 0);
let ok = is_some(opt);

-- After (메서드 호출)
let x = opt.unwrap_or(0);
let ok = opt.is_some();

-- Result도 동일하게 지원
let val = result.unwrap_or(default);
if result.is_ok() { ... }
```

### 제한사항

| 제한 | 설명 | 해결 계획 |
|------|------|-----------|
| 하드코딩 메서드 | Option/Result만 지원 | 트레이트 시스템 이후 확장 |
| map/and_then | 클로저 필요 | 클로저 구현 이후 |

### 원래 v0.18 패키지 계획

Database/CLI 패키지 (20개)는 다음 의존성이 해결된 후 진행:
- FFI: extern C 바인딩 (v0.19+)
- Async: async/await 구문 (v0.19+)
- Runtime: Vec/String/Box 동적 할당 (v0.19+)

---

## v0.18.1 Bootstrap Preparation (인프라 강화) ✅

> 목표: v0.19 Self-Hosting을 위한 개발 인프라 강화

### 배경

v0.19 Mirror(Self-Hosting) 진행 전 Gap 분석 결과:
- 클로저, 트레이트, FFI 런타임 등 핵심 기능 부재 확인
- 테스트 커버리지 및 디버깅 도구 부족

### Phase 13 - 파서 테스트 강화 ✅

| 테스트 카테고리 | 테스트 수 | 설명 |
|----------------|----------|------|
| 기본 표현식 | 3 | IntLit, BoolLit, StringLit |
| 이항 연산 | 3 | 산술, 비교, 논리 |
| 제어 흐름 | 5 | if/else, let, while, for, match |
| 구조체/열거형 | 4 | 정의, 초기화, 변형 |
| 제네릭 | 3 | 함수, 구조체, 열거형 |
| 계약 | 3 | pre, post, 결합 |
| 가시성/속성 | 2 | pub, @derive |
| 에러 처리 | 2 | ?, try 블록 |
| 메서드 호출 | 2 | 기본, 인자 포함 |
| extern/use | 2 | extern fn, use 문 |
| 오류 케이스 | 1 | 잘못된 구문 |
| **총합** | **30** | - |

**핵심 발견:**
- BMB 문법에서 Assignment(`x = value`)는 `BlockStmt`로만 유효 (표현식 아님)
- 루프/조건문 내 변경은 중첩 블록 필요: `{ { x = 1; x } }`

### Phase 14 - S-expression AST 출력 ✅

```bash
$ bmb parse example.bmb --format sexpr
(program
  (fn add :priv ((a i64) (b i64)) -> i64
    (+ a b))
)
```

**구현:**
- `bmb/src/ast/output.rs` 모듈 추가
- `to_sexpr()` 함수: AST → S-expression 변환
- `--format` CLI 플래그: `json` (기본) 또는 `sexpr`
- 5개 단위 테스트 포함

**지원 항목:**
| 항목 | 출력 형식 |
|------|----------|
| 함수 정의 | `(fn name :vis <params> params -> ret body)` |
| 구조체 | `(struct name <params> (fields...))` |
| 열거형 | `(enum name <params> (variants...))` |
| extern fn | `(extern-fn name (params) -> ret)` |
| use 문 | `(use path::to::item)` |
| 표현식 | Lisp 스타일 S-expression |

### Phase 15 - 컴파일러 경고 수정 ✅

**수정된 경고:**

| 파일 | 경고 | 수정 |
|------|------|------|
| `build/mod.rs` | unused import `CodeGen` | `#[cfg(feature = "llvm")]` 조건 추가 |
| `build/mod.rs` | unused variable `e` | `_` prefix 추가 |
| `codegen/llvm_text.rs` | unused variable `val` | 불필요한 변수 제거 |
| `codegen/llvm_text.rs` | unused variable `current_func` | `_` prefix 추가 |
| `codegen/wasm_text.rs` | unused variable `i` | 루프 변수 제거 |
| `lsp/mod.rs` | unused variables | `_` prefix 추가 |
| `lsp/mod.rs` | dead code | `#[allow(dead_code)]` 추가 |

**결과:** bmb 크레이트 경고 0개

### 테스트 결과

```bash
$ cargo test
running 85 tests
...
test result: ok. 85 passed; 0 failed
```

---

## v0.19 Complete (MIR Completion) ✅

> 목표: Self-Hosting에 필요한 MIR 기능 완성 (Struct/Enum/Pattern/Array)

### 배경

Gap 분석 결과 (docs/GAP_ANALYSIS.md 참조):
- MIR lowering에서 Struct/Enum/Pattern이 미구현 상태
- Self-Hosting은 이러한 핵심 기능 없이 불가능
- 원래 v0.19 "Mirror" 계획을 v0.22로 연기

### v0.19.0 - Struct MIR Support ✅

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| MirInst::StructInit | 구조체 초기화 명령 | ✅ 완료 |
| MirInst::FieldAccess | 필드 접근 명령 | ✅ 완료 |
| MirInst::FieldStore | 필드 저장 명령 | ✅ 완료 |
| lower_struct_init() | AST → MIR 변환 | ✅ 완료 |
| LLVM/WASM codegen | 구조체 메모리 레이아웃 | ✅ 완료 |

**구현 내용:**
- `MirInst::StructInit` - 필드별 초기화
- `MirInst::FieldAccess` - 필드 읽기
- `MirInst::FieldStore` - 필드 쓰기
- `MirType::Struct` / `MirType::StructPtr` 타입

### v0.19.1 - Enum MIR Support ✅

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| MirInst::EnumVariant | 열거형 변형 생성 | ✅ 완료 |
| Discriminant handling | 태그 값 관리 | ✅ 완료 |
| Tagged union repr | 태그 + 페이로드 레이아웃 | ✅ 완료 |
| LLVM/WASM codegen | Enum 코드 생성 | ✅ 완료 |

**구현 내용:**
- `MirInst::EnumVariant` - 변형 생성 (discriminant + args)
- `MirType::Enum` - 변형별 타입 정보 저장
- Unit/Tuple variant 모두 지원

### v0.19.2 - Pattern Matching ✅

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| Switch terminator | 분별자 기반 분기 | ✅ 완료 |
| Pattern compilation | 패턴 → Switch 변환 | ✅ 완료 |
| Variable binding | 패턴 변수 바인딩 | ✅ 완료 |
| Wildcard patterns | 기본 케이스 처리 | ✅ 완료 |

**구현 내용:**
- `Terminator::Switch` - 값 기반 다중 분기
- `compile_match_patterns()` - 패턴 컴파일
- `bind_pattern_variables()` - 변수 바인딩
- Literal, Var, Wildcard, EnumVariant, Struct 패턴 지원

### v0.19.3 - Array Support ✅

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| MirInst::ArrayInit | 배열 리터럴 MIR 변환 | ✅ 완료 |
| MirInst::IndexLoad | 인덱스 읽기 | ✅ 완료 |
| MirInst::IndexStore | 인덱스 쓰기 | ✅ 완료 |
| MirType::Array | 배열 타입 표현 | ✅ 완료 |

**구현 내용:**
- `MirInst::ArrayInit { dest, element_type, elements }`
- `MirInst::IndexLoad { dest, array, index }`
- `MirInst::IndexStore { array, index, value }`
- `MirType::Array { element_type, size }`

### v0.19.4 - Method Dispatch ✅

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| Method call lowering | 메서드 → 함수 호출 변환 | ✅ 완료 |
| Receiver as first arg | 수신자를 첫 번째 인자로 | ✅ 완료 |
| Static dispatch | 정적 디스패치 | ✅ 완료 |

**구현 내용:**
- `obj.method(args)` → `call method(obj, args)`
- 수신자를 첫 번째 인자로 전달하는 정적 디스패치
- 향후 트레이트 기반 동적 디스패치 확장 가능

### v0.19.5 - Integration Testing ✅

```bash
# MIR 테스트 결과 (14 tests)
cargo test mir::lower
test mir::lower::tests::test_lower_struct_init ... ok
test mir::lower::tests::test_lower_field_access ... ok
test mir::lower::tests::test_lower_enum_variant ... ok
test mir::lower::tests::test_lower_enum_unit_variant ... ok
test mir::lower::tests::test_lower_match_literal ... ok
test mir::lower::tests::test_lower_match_var_binding ... ok
test mir::lower::tests::test_lower_array_init ... ok
test mir::lower::tests::test_lower_array_index ... ok
test mir::lower::tests::test_lower_method_call ... ok
# 전체 109개 테스트 통과
```

### 기술적 세부사항

**MIR 변경 (`bmb/src/mir/mod.rs`):**
```rust
pub enum MirInst {
    // 기존...
    StructInit { name: String, fields: Vec<(String, Operand)> },
    FieldAccess { base: Operand, field: String },
    EnumVariant { enum_name: String, variant: String, fields: Vec<Operand> },
    ArrayLit { elements: Vec<Operand> },
    ArrayIndex { base: Operand, index: Operand },
}
```

**예상 LOC:**
| 모듈 | 변경량 |
|------|--------|
| mir/mod.rs | +100-150 |
| mir/lower.rs | +400-600 |
| codegen/llvm.rs | +200-300 |
| codegen/llvm_text.rs | +100-150 |

---

## v0.20 Extend (Language Extensions)

> 목표: Self-Hosting에 필요한 언어 확장 (Closures, Traits, FFI)

### v0.20.0 - Closures

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| Pipe token | `\|` 토큰 추가 (lexer) | ✅ 완료 |
| Closure AST | ClosureParam, Closure 표현식 | ✅ 완료 |
| Lambda syntax | `fn \|x\| { body }` 클로저 문법 | ✅ 완료 |
| Parser tests | 3개 테스트 케이스 추가 | ✅ 완료 |
| Capture semantics | 값 캡처 분석 | 계획 |
| Closure type inference | 클로저 타입 추론 | 계획 |
| MIR representation | 클로저 MIR 표현 (struct 디슈가링) | 계획 |

**구문 (2026-01-04 확정):**
```bmb
-- 클로저는 fn 키워드와 블록을 필수로 사용
fn || { 42 }                      -- 파라미터 없음
fn |x: i64| { x + 1 }             -- 단일 파라미터
fn |x: i64, y: i64| { x + y }     -- 다중 파라미터
```

**테스트 목표 (향후):**
```bmb
let add = fn |x: i64| { x + 1 };
let result = add(5);  -- 6

let list = [1, 2, 3];
let doubled = list.map(fn |x: i64| { x * 2 });  -- [2, 4, 6]
```

### v0.20.1 - Trait Foundation

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| trait keyword | 트레이트 정의 문법 | 계획 |
| impl blocks | 구현 블록 문법 | 계획 |
| Method resolution | 트레이트 메서드 해석 | 계획 |
| Basic traits | Clone, Debug, PartialEq | 계획 |

**테스트 목표:**
```bmb
trait Show {
    fn show(self) -> String;
}

impl Show for i64 {
    fn show(self) -> String = int_to_string(self);
}
```

### v0.20.2 - FFI Enhancement

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| extern "C" | C ABI 링킹 | 계획 |
| ABI handling | 호출 규약 처리 | 계획 |
| C library interop | libc 상호운용 | 계획 |
| Pointer safety | 안전한 포인터 처리 | 계획 |

**테스트 목표:**
```bmb
extern "C" fn malloc(size: usize) -> *mut u8;
extern "C" fn free(ptr: *mut u8);
```

---

## v0.21 Bootstrap (Bootstrap Enhancement)

> 목표: Bootstrap 컴파일러에 Struct/Enum MIR 지원 추가

### v0.21.0 - Bootstrap Struct Support

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| bootstrap/lowering.bmb | struct MIR 지원 추가 | 계획 |
| bootstrap/llvm_ir.bmb | struct LLVM 코드젠 | 계획 |
| Integration test | Rust compiler 결과 비교 | 계획 |

### v0.21.1 - Bootstrap Enum Support

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| bootstrap/lowering.bmb | enum MIR 지원 추가 | 계획 |
| bootstrap/llvm_ir.bmb | enum LLVM 코드젠 | 계획 |
| Pattern matching | switch 분기 생성 | 계획 |

### v0.21.2 - Equivalence Testing

```bash
# Rust compiler vs Bootstrap compiler 결과 비교
$ bmb build test.bmb --emit-mir > rust_mir.txt
$ ./bootstrap test.bmb > bootstrap_mir.txt
$ diff rust_mir.txt bootstrap_mir.txt
```

---

## v0.22 Mirror (Self-Hosting)

> 목표: 모든 서브모듈 BMB로 재작성 + Stage 2 검증

### 서브모듈 BMB 재작성

| 컴포넌트 | Rust 버전 | BMB 버전 | 검증 |
|----------|-----------|----------|------|
| Lexer | ✅ | 🔄 | Stage 2 |
| Parser | ✅ | 🔄 | Stage 2 |
| Type Checker | ✅ | 🔄 | Stage 2 |
| MIR Lowering | ✅ | 🔄 | Stage 2 |
| LLVM Codegen | ✅ | 🔄 | Stage 2 |
| WASM Codegen | ✅ | 🔄 | Stage 2 |
| gotgan CLI | ✅ | 🔄 | 기능 동등 |
| gotgan Resolver | ✅ | 🔄 | 기능 동등 |
| action-bmb | ✅ | 🔄 | 기능 동등 |

### Stage 2 검증

```bash
# Stage 0: Rust 컴파일러
$ cargo build --release
$ ./bmb-rust build bmb-compiler -> bmb-stage1

# Stage 1: BMB로 작성된 컴파일러
$ ./bmb-stage1 build bmb-compiler -> bmb-stage2

# Stage 2: Stage 1으로 컴파일된 컴파일러
$ ./bmb-stage2 build bmb-compiler -> bmb-stage3

# 검증: Stage 2 = Stage 3
$ diff bmb-stage2 bmb-stage3  # 동일해야 함
```

### gotgan 등록

```bash
$ gotgan publish bmb-compiler
$ gotgan publish bmb-gotgan
$ gotgan publish bmb-action
```

---

## v0.23 Showcase (주요 앱 시나리오 샘플)

> 목표: 실제 도메인 샘플 애플리케이션 10개

### 샘플 애플리케이션

| # | 앱 이름 | 도메인 | 사용 패키지 |
|---|---------|--------|-------------|
| 1 | **bmb-api-server** | 웹 API | bmb-axum, bmb-postgres, bmb-serde |
| 2 | **bmb-cli-tool** | CLI 유틸리티 | bmb-clap, bmb-config, bmb-log |
| 3 | **bmb-chat-server** | 실시간 채팅 | bmb-websocket, bmb-redis, bmb-async |
| 4 | **bmb-file-manager** | 파일 유틸리티 | bmb-fs, bmb-tar, bmb-walkdir |
| 5 | **bmb-crypto-tool** | 암호화 도구 | bmb-crypto, bmb-aes, bmb-argon2 |
| 6 | **bmb-db-client** | DB 클라이언트 | bmb-sql, bmb-postgres, bmb-table |
| 7 | **bmb-http-proxy** | HTTP 프록시 | bmb-hyper, bmb-tls, bmb-async |
| 8 | **bmb-json-processor** | JSON 처리 | bmb-json, bmb-serde, bmb-io |
| 9 | **bmb-task-runner** | 태스크 러너 | bmb-async, bmb-channel, bmb-log |
| 10 | **bmb-config-manager** | 설정 관리 | bmb-toml, bmb-yaml, bmb-config |

---

## v0.24 Launch (프로덕션 서비스 런칭)

> 목표: 서브모듈을 실제 도메인 서비스로 배포

### 서비스 런칭

| 서비스 | 도메인 | 설명 | 기술 스택 |
|--------|--------|------|-----------|
| **gotgan Registry** | gotgan.bmb.dev | 패키지 레지스트리 | bmb-axum, bmb-postgres |
| **BMB Playground** | play.bmb.dev | 온라인 플레이그라운드 | WASM, bmb-compiler |
| **BMB Docs** | docs.bmb.dev | 문서 사이트 | bmb-axum, 마크다운 |
| **Benchmark Dashboard** | bench.bmb.dev | 벤치마크 대시보드 | bmb-axum, bmb-json |
| **Package Search** | search.bmb.dev | 패키지 검색 API | bmb-axum, bmb-redis |

### 서비스 아키텍처

```
                    ┌──────────────────┐
                    │   Cloudflare     │
                    │   (CDN + DNS)    │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
      ┌───────▼───┐  ┌───────▼───┐  ┌───────▼───┐
      │ gotgan    │  │ play      │  │ docs      │
      │ Registry  │  │ Playground│  │ Site      │
      │ (BMB)     │  │ (WASM)    │  │ (BMB)     │
      └───────────┘  └───────────┘  └───────────┘
              │
      ┌───────▼───┐
      │ PostgreSQL│
      │ + Redis   │
      └───────────┘
```

### 배포 파이프라인

```yaml
# .github/workflows/deploy.yml
name: Deploy BMB Services
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: lang-bmb/action-bmb@v1
      - run: gotgan build --release
      - run: docker build -t bmb-service .
      - run: kubectl apply -f k8s/
```

---

## v0.22 Velocity (C/Rust 성능 추월 스프린트)

> 목표: C/Rust 대비 동등 이상 성능 달성

### 최적화 전략

| 단계 | 기법 | 예상 효과 |
|------|------|----------|
| 1 | 핫스팟 프로파일링 | 병목 식별 |
| 2 | 계약 기반 최적화 | 경계 검사 제거 |
| 3 | SIMD 자동 벡터화 | 수치 연산 +200% |
| 4 | 인라이닝 개선 | 함수 호출 -50% |
| 5 | 메모리 레이아웃 최적화 | 캐시 효율 +30% |
| 6 | 정적 디스패치 강화 | 가상 호출 제거 |

### 계약 기반 최적화 예시

```bmb
-- 계약으로 인해 경계 검사 제거 가능
fn sum_range(arr: &[i32], start: usize, end: usize) -> i32
  pre start <= end
  pre end <= len(arr)
= {
    let mut total = 0;
    for i in start..end {
        total += arr[i];  -- 경계 검사 불필요 (계약으로 증명됨)
    }
    total
};
```

### 벤치마크 v3 (최종)

```
benchmark-bmb/v0.22/
├── comprehensive/
│   ├── raytracer.bmb      # 레이트레이서
│   ├── nbody.bmb          # N-body 시뮬레이션
│   ├── regex-redux.bmb    # 정규표현식
│   └── spectral-norm.bmb  # 스펙트럴 노름
├── compare/
│   ├── c/
│   ├── rust/
│   └── bmb/
└── results/
    ├── v0.15-baseline.json
    ├── v0.17-optimized.json
    └── v0.22-final.json
```

### 최종 KPI

| 지표 | v0.15 | v0.17 | v0.22 | 목표 |
|------|-------|-------|-------|------|
| 컴파일 속도 (vs Rust) | 80% | 90% | 105% | ✅ 100%+ |
| 런타임 성능 (vs C) | 70% | 85% | 102% | ✅ 100%+ |
| 바이너리 크기 (vs Rust) | 120% | 100% | 88% | ✅ 90% |
| 메모리 사용량 (vs Rust) | 110% | 100% | 93% | ✅ 95% |

---

## v0.26 Query (AI Query System)

> 목표: AI 도구가 BMB 프로젝트의 계약/증명 정보를 쿼리할 수 있는 전용 인터페이스

**RFC**: [RFC-0001-AI-Query-System](RFC/RFC-0001-AI-Query-System.md)

### 배경

BMB의 계약 시스템(pre/post, forall/exists, refinement types)은 컴파일 타임에 풍부한 의미 정보를 추출한다. 이 정보를 AI 코드 생성 도구가 효과적으로 활용할 수 있도록 전용 쿼리 시스템을 제공한다.

### v0.26.0 - Index Generation

| 구성요소 | 설명 | 상태 |
|----------|------|------|
| `.bmb/index/` | 인덱스 디렉토리 구조 | 계획 |
| `bmb index` | 인덱스 생성 명령 | 계획 |
| symbols.idx | 심볼 테이블 | 계획 |
| proofs.idx | 증명 결과 | 계획 |

### v0.26.1 - Basic Queries

| 명령 | 설명 | 상태 |
|------|------|------|
| `bmb q sym` | 심볼 검색 | 계획 |
| `bmb q fn` | 함수 조회 | 계획 |
| `bmb q type` | 타입 조회 | 계획 |
| `bmb q proof` | 증명 상태 | 계획 |

### v0.26.2 - Advanced Queries

| 명령 | 설명 | 상태 |
|------|------|------|
| `bmb q contract` | 계약 조회 | 계획 |
| `bmb q deps` | 의존성 분석 | 계획 |
| `bmb q ctx` | AI 컨텍스트 | 계획 |
| `bmb q counterexample` | 반례 조회 | 계획 |

### v0.26.3 - Integration

| 명령 | 설명 | 상태 |
|------|------|------|
| `bmb q sig` | 시그니처 검색 | 계획 |
| `bmb q impact` | 영향 분석 | 계획 |
| `bmb q batch` | 배치 쿼리 | 계획 |
| `bmb q serve` | HTTP 서버 모드 | 계획 |

---

## v1.0-RC Golden (최종 검증)

> 목표: 완전한 자기 컴파일 + 검증 + 안정성 약속

### 최종 체크리스트

| 카테고리 | 항목 | 상태 |
|----------|------|------|
| **Self-Hosting** | Stage 2 컴파일러 동작 | ⬜ |
| | gotgan BMB 버전 동작 | ⬜ |
| | action-bmb BMB 버전 동작 | ⬜ |
| **패키지** | 115개 패키지 등록 | ⬜ |
| | 각 패키지 샘플 앱 | ⬜ |
| | 테스트 커버리지 > 80% | ⬜ |
| **성능** | C 대비 100%+ 런타임 | ⬜ |
| | Rust 대비 100%+ 컴파일 | ⬜ |
| | 벤치마크 스위트 통과 | ⬜ |
| **서비스** | gotgan.bmb.dev 운영 | ⬜ |
| | play.bmb.dev 운영 | ⬜ |
| | docs.bmb.dev 운영 | ⬜ |
| **문서** | 언어 레퍼런스 완성 | ⬜ |
| | 패키지 문서화 100% | ⬜ |
| | 튜토리얼 5개 이상 | ⬜ |
| **커뮤니티** | GitHub Discussions 활성화 | ⬜ |
| | 컨트리뷰션 가이드 | ⬜ |
| | 로드맵 1.x 공개 | ⬜ |

### 1.0 안정성 약속

```
v1.0 이후 보장:
├── API 호환성 (1.x 내 Breaking changes 없음)
├── ABI 안정성 (라이브러리 바이너리 호환)
├── 계약 보장 (검증된 계약 유지)
└── 성능 비회귀 (벤치마크 기준 유지)
```

### 릴리스 타임라인 (예상, REVISED)

```
v0.13 Forge        ────▶ 2025 Q2 ✅
v0.14 Foundation   ────▶ 2025 Q3 ✅
v0.15 Generics     ────▶ 2025 Q3 ✅
v0.16 Consolidate  ────▶ 2025 Q4 ✅
v0.17 Module       ────▶ 2025 Q4 ✅
v0.18 Methods      ────▶ 2026 Q1 ✅
v0.19 Complete     ────▶ 2026 Q1 ✅ (MIR Completion)
v0.20 Extend       ────▶ 2026 Q2 (Language Extensions)
v0.21 Bootstrap    ────▶ 2026 Q2 (Bootstrap Enhancement)
v0.22 Mirror       ────▶ 2026 Q3 (Self-Hosting)
v0.23 Showcase     ────▶ 2026 Q3
v0.24 Launch       ────▶ 2026 Q4
v0.25 Velocity     ────▶ 2026 Q4
v0.26 Query        ────▶ 2027 Q1 (AI Query System - RFC-0001)
v1.0-RC Golden     ────▶ 2027 Q2
```

---

## 부록: 전체 패키지 목록

### A. Core/Foundation (20개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 1 | bmb-core | v0.14 | std::core |
| 2 | bmb-alloc | v0.14 | alloc |
| 3 | bmb-sync | v0.14 | std::sync |
| 4 | bmb-atomic | v0.14 | std::sync::atomic |
| 5 | bmb-cell | v0.14 | std::cell |
| 6 | bmb-ptr | v0.14 | std::ptr |
| 7 | bmb-mem | v0.14 | std::mem |
| 8 | bmb-num | v0.14 | num-traits |
| 9 | bmb-ops | v0.14 | std::ops |
| 10 | bmb-iter | v0.14 | std::iter |
| 11 | bmb-slice | v0.14 | std::slice |
| 12 | bmb-array | v0.14 | std::array |
| 13 | bmb-option | v0.14 | std::option |
| 14 | bmb-result | v0.14 | std::result |
| 15 | bmb-convert | v0.14 | std::convert |
| 16 | bmb-default | v0.14 | std::default |
| 17 | bmb-clone | v0.14 | std::clone |
| 18 | bmb-cmp | v0.14 | std::cmp |
| 19 | bmb-hash | v0.14 | std::hash |
| 20 | bmb-fmt | v0.14 | std::fmt |

### B. Collections (15개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 21 | bmb-vec | v0.14 | Vec |
| 22 | bmb-string | v0.14 | String |
| 23 | bmb-hashmap | v0.14 | HashMap |
| 24 | bmb-hashset | v0.14 | HashSet |
| 25 | bmb-deque | v0.14 | VecDeque |
| 26 | bmb-btreemap | v0.15 | BTreeMap |
| 27 | bmb-btreeset | v0.15 | BTreeSet |
| 28 | bmb-linkedlist | v0.15 | LinkedList |
| 29 | bmb-heap | v0.15 | BinaryHeap |
| 30 | bmb-smallvec | v0.15 | smallvec |
| 31 | bmb-indexmap | v0.15 | indexmap |
| 32 | bmb-bitvec | v0.15 | bitvec |
| 33 | bmb-arena | v0.15 | typed-arena |
| 34 | bmb-slotmap | v0.15 | slotmap |
| 35 | bmb-lru | v0.15 | lru |

### C. IO/Filesystem (10개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 36 | bmb-io | v0.15 | std::io |
| 37 | bmb-fs | v0.15 | std::fs |
| 38 | bmb-path | v0.15 | std::path |
| 39 | bmb-buf | v0.15 | std::io::{BufReader,BufWriter} |
| 40 | bmb-stdio | v0.15 | std::io::stdio |
| 41 | bmb-tempfile | v0.15 | tempfile |
| 42 | bmb-walkdir | v0.15 | walkdir |
| 43 | bmb-notify | v0.15 | notify |
| 44 | bmb-memmap | v0.15 | memmap2 |
| 45 | bmb-tar | v0.15 | tar |

### D. Networking (15개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 46 | bmb-net | v0.15 | std::net |
| 47 | bmb-tcp | v0.15 | TcpListener/TcpStream |
| 48 | bmb-udp | v0.15 | UdpSocket |
| 49 | bmb-socket | v0.15 | socket2 |
| 50 | bmb-dns | v0.15 | trust-dns-resolver |
| 51 | bmb-url | v0.16 | url |
| 52 | bmb-uri | v0.16 | http::Uri |
| 53 | bmb-http | v0.16 | http |
| 54 | bmb-websocket | v0.16 | tungstenite |
| 55 | bmb-tls | v0.16 | rustls |
| 56 | bmb-hyper | v0.16 | hyper |
| 57 | bmb-reqwest | v0.16 | reqwest |
| 58 | bmb-axum | v0.16 | axum |
| 59 | bmb-grpc | v0.16 | tonic |
| 60 | bmb-graphql | v0.16 | async-graphql |

### E. Serialization (15개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 61 | bmb-serde | v0.16 | serde |
| 62 | bmb-json | v0.16 | serde_json |
| 63 | bmb-toml | v0.16 | toml |
| 64 | bmb-yaml | v0.16 | serde_yaml |
| 65 | bmb-xml | v0.16 | quick-xml |
| 66 | bmb-csv | v0.16 | csv |
| 67 | bmb-msgpack | v0.16 | rmp-serde |
| 68 | bmb-protobuf | v0.16 | prost |
| 69 | bmb-bincode | v0.16 | bincode |
| 70 | bmb-base64 | v0.16 | base64 |
| 71 | bmb-utf8 | v0.16 | encoding_rs |
| 72 | bmb-regex | v0.16 | regex |
| 73 | bmb-glob | v0.16 | glob |
| 74 | bmb-mime | v0.16 | mime |
| 75 | bmb-form | v0.16 | serde_urlencoded |

### F. Async (10개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 76 | bmb-async | v0.17 | tokio |
| 77 | bmb-future | v0.17 | futures |
| 78 | bmb-task | v0.17 | tokio::task |
| 79 | bmb-channel | v0.17 | tokio::sync::mpsc |
| 80 | bmb-select | v0.17 | tokio::select! |
| 81 | bmb-timeout | v0.17 | tokio::time::timeout |
| 82 | bmb-stream | v0.17 | futures::Stream |
| 83 | bmb-sink | v0.17 | futures::Sink |
| 84 | bmb-timer | v0.17 | tokio::time |
| 85 | bmb-executor | v0.17 | tokio::runtime |

### G. Crypto/Security (10개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 86 | bmb-crypto | v0.17 | ring |
| 87 | bmb-sha | v0.17 | sha2 |
| 88 | bmb-md5 | v0.17 | md5 |
| 89 | bmb-aes | v0.17 | aes |
| 90 | bmb-rsa | v0.17 | rsa |
| 91 | bmb-ecdsa | v0.17 | ecdsa |
| 92 | bmb-hmac | v0.17 | hmac |
| 93 | bmb-pbkdf2 | v0.17 | pbkdf2 |
| 94 | bmb-argon2 | v0.17 | argon2 |
| 95 | bmb-rand | v0.17 | rand |

### H. Database (10개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 96 | bmb-sql | v0.18 | sqlx |
| 97 | bmb-postgres | v0.18 | tokio-postgres |
| 98 | bmb-mysql | v0.18 | mysql_async |
| 99 | bmb-sqlite | v0.18 | rusqlite |
| 100 | bmb-redis | v0.18 | redis |
| 101 | bmb-mongodb | v0.18 | mongodb |
| 102 | bmb-pool | v0.18 | deadpool |
| 103 | bmb-migrate | v0.18 | sqlx::migrate |
| 104 | bmb-orm | v0.18 | diesel/sea-orm |
| 105 | bmb-kv | v0.18 | sled |

### I. CLI/Tools (10개)

| # | 이름 | 버전 | Rust 대응 |
|---|------|------|-----------|
| 106 | bmb-clap | v0.18 | clap |
| 107 | bmb-env | v0.18 | std::env |
| 108 | bmb-log | v0.18 | log/env_logger |
| 109 | bmb-tracing | v0.18 | tracing |
| 110 | bmb-config | v0.18 | config |
| 111 | bmb-term | v0.18 | termcolor |
| 112 | bmb-progress | v0.18 | indicatif |
| 113 | bmb-table | v0.18 | tabled |
| 114 | bmb-prompt | v0.18 | dialoguer |
| 115 | bmb-test | v0.18 | test harness |

---

## 부록: 샘플 앱 목록

### 패키지별 샘플 앱 (115개)

각 패키지당 최소 1개 샘플 앱 포함:

```
bmb-samples/
├── core/
│   ├── bmb-core-demo/          # 기본 타입 사용
│   ├── bmb-iter-demo/          # 이터레이터 패턴
│   └── bmb-hash-demo/          # 해시 함수 사용
├── collections/
│   ├── bmb-vec-demo/           # 동적 배열
│   ├── bmb-hashmap-demo/       # 워드 카운터
│   └── bmb-lru-demo/           # 캐시 구현
├── io/
│   ├── bmb-fs-demo/            # 파일 시스템 탐색
│   ├── bmb-walkdir-demo/       # 디렉토리 순회
│   └── bmb-tar-demo/           # 아카이브 생성
├── network/
│   ├── bmb-http-demo/          # HTTP 클라이언트
│   ├── bmb-websocket-demo/     # WebSocket 채팅
│   └── bmb-axum-demo/          # REST API 서버
├── serialize/
│   ├── bmb-json-demo/          # JSON 파싱
│   ├── bmb-toml-demo/          # 설정 파일 읽기
│   └── bmb-protobuf-demo/      # 프로토콜 버퍼
├── async/
│   ├── bmb-async-demo/         # 비동기 태스크
│   ├── bmb-channel-demo/       # 채널 통신
│   └── bmb-stream-demo/        # 스트림 처리
├── crypto/
│   ├── bmb-sha-demo/           # 해시 계산
│   ├── bmb-aes-demo/           # 암호화/복호화
│   └── bmb-argon2-demo/        # 패스워드 해싱
├── database/
│   ├── bmb-postgres-demo/      # PostgreSQL CRUD
│   ├── bmb-redis-demo/         # Redis 캐싱
│   └── bmb-sqlite-demo/        # 로컬 DB
└── cli/
    ├── bmb-clap-demo/          # CLI 인자 파싱
    ├── bmb-log-demo/           # 로깅 설정
    └── bmb-progress-demo/      # 진행 바 표시
```

### 주요 도메인 샘플 앱 (10개)

v0.20 Showcase에서 개발:

1. **bmb-api-server** - REST API 서버
2. **bmb-cli-tool** - CLI 유틸리티
3. **bmb-chat-server** - 실시간 채팅
4. **bmb-file-manager** - 파일 관리
5. **bmb-crypto-tool** - 암호화 도구
6. **bmb-db-client** - DB 클라이언트
7. **bmb-http-proxy** - HTTP 프록시
8. **bmb-json-processor** - JSON 처리
9. **bmb-task-runner** - 태스크 러너
10. **bmb-config-manager** - 설정 관리
