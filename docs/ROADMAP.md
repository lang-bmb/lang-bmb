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
| v0.14 | **Foundation** | Core 패키지 25개 + gotgan 등록 | 계획 |
| v0.15 | **Stream** | Collections/IO 패키지 25개 + 벤치마크 v1 | 계획 |
| v0.16 | **Connect** | Network/Serialization 25개 + 최적화 1차 | 계획 |
| v0.17 | **Parallel** | Async/Crypto 20개 + 벤치마크 v2 | 계획 |
| v0.18 | **Persist** | Database/CLI 20개 + 최적화 2차 | 계획 |
| v0.19 | **Mirror** | Self-Hosting (BMB 자기 컴파일) | 계획 |
| v0.20 | **Showcase** | 주요 앱 시나리오 샘플 10개 | 계획 |
| v0.21 | **Launch** | 프로덕션 서비스 런칭 | 계획 |
| v0.22 | **Velocity** | C/Rust 성능 추월 스프린트 | 계획 |
| v1.0-RC | **Golden** | 최종 검증 + 안정성 약속 | 계획 |

---

## 생태계 레포지토리

| 레포지토리 | 용도 | Rust 버전 | BMB 재작성 | 서비스 런칭 |
|------------|------|-----------|------------|-------------|
| lang-bmb | 메인 컴파일러 | v0.1 ✅ | v0.19 | - |
| gotgan | 패키지 매니저 | v0.8 ✅ | v0.19 | gotgan.bmb.dev |
| action-bmb | GitHub Action | v0.7 ✅ | v0.19 | - |
| bmb-samples | 예제 프로그램 | - | v0.20 | - |
| benchmark-bmb | 표준 벤치마크 | v0.9 ✅ | v0.15 | bench.bmb.dev |
| playground | 온라인 플레이그라운드 | - | v0.21 | play.bmb.dev |
| lang-bmb-site | 공식 웹사이트 | - | v0.21 | bmb.dev |

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

Phase 6 (v0.13-v0.18): 패키지 생태계 구축 ★ NEW
  - 115개 패키지 개발
  - 각 패키지 샘플 앱
  - gotgan 레지스트리 등록
  - 벤치마크 + 최적화 반복

Phase 7 (v0.19): Self-Hosting ★ NEW
  - 컴파일러 BMB 재작성
  - gotgan BMB 재작성
  - Stage 2 자기 컴파일 검증

Phase 8 (v0.20-v0.21): 프로덕션 런칭 ★ NEW
  - 주요 앱 시나리오 샘플
  - 서브모듈 서비스 런칭

Phase 9 (v0.22): 성능 스프린트 ★ NEW
  - C/Rust 성능 추월
  - 계약 기반 최적화

Phase 10 (v1.0-RC): Golden Release
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

## v0.14 Foundation (Core 패키지 25개)

> 목표: 핵심 패키지 25개 + gotgan 등록 + 샘플 앱

### 패키지 목록

| # | 패키지 | 설명 | Rust 대응 |
|---|--------|------|-----------|
| 1 | bmb-core | 핵심 타입, 트레이트 | std::core |
| 2 | bmb-alloc | 메모리 할당자 | alloc |
| 3 | bmb-sync | 동기화 프리미티브 | std::sync |
| 4 | bmb-atomic | 원자적 연산 | std::sync::atomic |
| 5 | bmb-cell | 내부 가변성 | std::cell |
| 6 | bmb-ptr | 포인터 유틸리티 | std::ptr |
| 7 | bmb-mem | 메모리 유틸리티 | std::mem |
| 8 | bmb-num | 수치 트레이트 | num-traits |
| 9 | bmb-ops | 연산자 트레이트 | std::ops |
| 10 | bmb-iter | 이터레이터 | std::iter |
| 11 | bmb-slice | 슬라이스 유틸리티 | std::slice |
| 12 | bmb-array | 배열 유틸리티 | std::array |
| 13 | bmb-option | Option 타입 | std::option |
| 14 | bmb-result | Result 타입 | std::result |
| 15 | bmb-convert | 타입 변환 | std::convert |
| 16 | bmb-default | Default 트레이트 | std::default |
| 17 | bmb-clone | Clone 트레이트 | std::clone |
| 18 | bmb-cmp | 비교 트레이트 | std::cmp |
| 19 | bmb-hash | 해싱 | std::hash |
| 20 | bmb-fmt | 포맷팅 | std::fmt |
| 21 | bmb-vec | 동적 배열 | Vec |
| 22 | bmb-string | 문자열 | String |
| 23 | bmb-hashmap | 해시맵 | HashMap |
| 24 | bmb-hashset | 해시셋 | HashSet |
| 25 | bmb-deque | 양방향 큐 | VecDeque |

### 샘플 앱 (25개)

각 패키지당 1개 이상:
- `bmb-core-demo`: 기본 타입 사용 예제
- `bmb-vec-demo`: 동적 배열 활용
- `bmb-hashmap-demo`: 워드 카운터
- ...

### gotgan 등록

```bash
$ gotgan publish bmb-core
$ gotgan publish bmb-vec
...
```

---

## v0.15 Stream (Collections/IO 25개 + 벤치마크 v1)

> 목표: 컬렉션/IO 패키지 + C/Rust 벤치마크 기준선

### 패키지 목록

| # | 패키지 | 설명 |
|---|--------|------|
| 26 | bmb-btreemap | B-트리 맵 |
| 27 | bmb-btreeset | B-트리 셋 |
| 28 | bmb-linkedlist | 연결 리스트 |
| 29 | bmb-heap | 바이너리 힙 |
| 30 | bmb-smallvec | 스택 최적화 벡터 |
| 31 | bmb-indexmap | 순서 유지 맵 |
| 32 | bmb-bitvec | 비트 벡터 |
| 33 | bmb-arena | 아레나 할당자 |
| 34 | bmb-slotmap | 슬롯 맵 |
| 35 | bmb-lru | LRU 캐시 |
| 36 | bmb-io | IO 트레이트 |
| 37 | bmb-fs | 파일 시스템 |
| 38 | bmb-path | 경로 처리 |
| 39 | bmb-buf | 버퍼 IO |
| 40 | bmb-stdio | 표준 입출력 |
| 41 | bmb-tempfile | 임시 파일 |
| 42 | bmb-walkdir | 디렉토리 순회 |
| 43 | bmb-notify | 파일 감시 |
| 44 | bmb-memmap | 메모리 맵 |
| 45 | bmb-tar | tar 아카이브 |
| 46 | bmb-net | 네트워크 기초 |
| 47 | bmb-tcp | TCP 소켓 |
| 48 | bmb-udp | UDP 소켓 |
| 49 | bmb-socket | 소켓 추상화 |
| 50 | bmb-dns | DNS 리졸버 |

### 벤치마크 v1 (기준선)

```
benchmark-bmb/v0.15/
├── micro/
│   ├── fibonacci.bmb      # vs fibonacci.c, fibonacci.rs
│   ├── primes.bmb
│   └── sorting.bmb
├── algo/
│   ├── dijkstra.bmb
│   └── quicksort.bmb
└── results/
    └── baseline-v0.15.json
```

**KPI 측정:**
- 컴파일 속도: Rust 대비 80%
- 런타임 성능: C 대비 70%
- 바이너리 크기: Rust 대비 120%

---

## v0.16 Connect (Network/Serialization 25개 + 최적화 1차)

> 목표: 네트워크/직렬화 패키지 + 첫 번째 성능 최적화

### 패키지 목록

| # | 패키지 | 설명 |
|---|--------|------|
| 51 | bmb-url | URL 파싱 |
| 52 | bmb-uri | URI 처리 |
| 53 | bmb-http | HTTP 클라이언트/서버 |
| 54 | bmb-websocket | WebSocket |
| 55 | bmb-tls | TLS/SSL |
| 56 | bmb-hyper | 고성능 HTTP (hyper 포트) |
| 57 | bmb-reqwest | HTTP 클라이언트 (reqwest 포트) |
| 58 | bmb-axum | 웹 프레임워크 (axum 포트) |
| 59 | bmb-grpc | gRPC |
| 60 | bmb-graphql | GraphQL |
| 61 | bmb-serde | 직렬화 프레임워크 |
| 62 | bmb-json | JSON |
| 63 | bmb-toml | TOML |
| 64 | bmb-yaml | YAML |
| 65 | bmb-xml | XML |
| 66 | bmb-csv | CSV |
| 67 | bmb-msgpack | MessagePack |
| 68 | bmb-protobuf | Protocol Buffers |
| 69 | bmb-bincode | 바이너리 인코딩 |
| 70 | bmb-base64 | Base64 |
| 71 | bmb-utf8 | UTF-8 처리 |
| 72 | bmb-regex | 정규표현식 |
| 73 | bmb-glob | 글로브 패턴 |
| 74 | bmb-mime | MIME 타입 |
| 75 | bmb-form | 폼 데이터 |

### 최적화 1차

| 영역 | 기법 | 목표 |
|------|------|------|
| 컴파일 | 증분 컴파일 | -20% 시간 |
| 런타임 | 인라이닝 개선 | +10% 성능 |
| 메모리 | 할당 최적화 | -15% 사용량 |

---

## v0.17 Parallel (Async/Crypto 20개 + 벤치마크 v2)

> 목표: 비동기/암호화 패키지 + 최적화 검증

### 패키지 목록

| # | 패키지 | 설명 |
|---|--------|------|
| 76 | bmb-async | 비동기 런타임 |
| 77 | bmb-future | Future 트레이트 |
| 78 | bmb-task | 태스크 스포닝 |
| 79 | bmb-channel | 비동기 채널 |
| 80 | bmb-select | select 매크로 |
| 81 | bmb-timeout | 타임아웃 |
| 82 | bmb-stream | Stream 트레이트 |
| 83 | bmb-sink | Sink 트레이트 |
| 84 | bmb-timer | 비동기 타이머 |
| 85 | bmb-executor | 실행기 |
| 86 | bmb-crypto | 암호화 기초 |
| 87 | bmb-sha | SHA 해시 |
| 88 | bmb-md5 | MD5 해시 |
| 89 | bmb-aes | AES 암호화 |
| 90 | bmb-rsa | RSA |
| 91 | bmb-ecdsa | ECDSA |
| 92 | bmb-hmac | HMAC |
| 93 | bmb-pbkdf2 | PBKDF2 |
| 94 | bmb-argon2 | Argon2 |
| 95 | bmb-rand | 난수 생성 |

### 벤치마크 v2

```
benchmark-bmb/v0.17/
├── async/
│   ├── spawn-million.bmb    # 100만 태스크 생성
│   ├── channel-throughput.bmb
│   └── http-concurrent.bmb
├── crypto/
│   ├── sha256-throughput.bmb
│   └── aes-encrypt.bmb
└── results/
    ├── v0.15-baseline.json
    └── v0.17-optimized.json
```

**KPI 검증:**
- 컴파일 속도: Rust 대비 90% ✓
- 런타임 성능: C 대비 85% ✓

---

## v0.18 Persist (Database/CLI 20개 + 최적화 2차)

> 목표: 데이터베이스/CLI 패키지 + 두 번째 최적화

### 패키지 목록

| # | 패키지 | 설명 |
|---|--------|------|
| 96 | bmb-sql | SQL 빌더 |
| 97 | bmb-postgres | PostgreSQL |
| 98 | bmb-mysql | MySQL |
| 99 | bmb-sqlite | SQLite |
| 100 | bmb-redis | Redis |
| 101 | bmb-mongodb | MongoDB |
| 102 | bmb-pool | 커넥션 풀 |
| 103 | bmb-migrate | 마이그레이션 |
| 104 | bmb-orm | ORM |
| 105 | bmb-kv | 키-밸류 스토어 |
| 106 | bmb-clap | 인자 파싱 |
| 107 | bmb-env | 환경 변수 |
| 108 | bmb-log | 로깅 |
| 109 | bmb-tracing | 트레이싱 |
| 110 | bmb-config | 설정 관리 |
| 111 | bmb-term | 터미널 색상 |
| 112 | bmb-progress | 진행 바 |
| 113 | bmb-table | 테이블 출력 |
| 114 | bmb-prompt | 대화형 프롬프트 |
| 115 | bmb-test | 테스트 프레임워크 |

### 최적화 2차

| 영역 | 기법 | 목표 |
|------|------|------|
| LLVM | 최적화 패스 튜닝 | +15% 성능 |
| 계약 | 경계 검사 제거 | +10% 성능 |
| SIMD | 자동 벡터화 | +20% 특정 연산 |

---

## v0.19 Mirror (Self-Hosting)

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

## v0.20 Showcase (주요 앱 시나리오 샘플)

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

### 각 샘플 구조

```
bmb-api-server/
├── gotgan.toml
├── src/
│   ├── main.bmb
│   ├── routes.bmb
│   ├── models.bmb
│   └── db.bmb
├── tests/
│   └── api_test.bmb
└── README.md
```

### gotgan 등록

```bash
$ gotgan publish bmb-api-server
$ gotgan publish bmb-cli-tool
...
```

---

## v0.21 Launch (프로덕션 서비스 런칭)

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

### 릴리스 타임라인 (예상)

```
v0.13 Forge        ────▶ 2025 Q2
v0.14 Foundation   ────▶ 2025 Q3
v0.15 Stream       ────▶ 2025 Q3
v0.16 Connect      ────▶ 2025 Q4
v0.17 Parallel     ────▶ 2025 Q4
v0.18 Persist      ────▶ 2026 Q1
v0.19 Mirror       ────▶ 2026 Q1
v0.20 Showcase     ────▶ 2026 Q2
v0.21 Launch       ────▶ 2026 Q2
v0.22 Velocity     ────▶ 2026 Q3
v1.0-RC Golden     ────▶ 2026 Q4
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
