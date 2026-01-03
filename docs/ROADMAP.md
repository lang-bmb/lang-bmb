# BMB 로드맵 v0.1 → v1.0

> 점진적 난이도 진행 + 완전한 생태계 + BMB 부트스트래핑

---

## 설계 원칙

| 원칙 | 설명 | 참고 |
|------|------|------|
| **점진적 진행** | 각 버전 간 난이도 차이 최소화 | Gleam 5년 0.x 여정 |
| **도구 내장** | 별도 설치 없이 `bmb fmt`, `bmb lsp` 동작 | Gleam 방식 |
| **작은 배포** | 큰 기능을 여러 minor 버전으로 분할 | Zig 패턴 |
| **0.x = 실험** | Breaking changes 허용, 1.0 = 안정성 약속 | 모든 언어 공통 |

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
| v0.11 | Dawn | AI-Native gotgan + Bootstrap 완성 | 🔄 진행중 (v0.11.4-7 ✅) |
| v0.12 | Horizon | WASM 듀얼 타깃 | 🔄 진행중 (v0.12.0-2 ✅) |
| v0.13 | Summit | 생태계 완성 (MCP, 레지스트리) | 계획 |
| v1.0-RC | Golden | 부트스트래핑 완료 + 검증 | 계획 |

---

## 생태계 레포지토리

| 레포지토리 | 용도 | 시작 | Rust | BMB 재작성 |
|------------|------|------|------|------------|
| [lang-bmb](https://github.com/lang-bmb/lang-bmb) | 메인 컴파일러 | v0.1 ✅ | v0.1 ✅ | v0.11 |
| [bmb-samples](https://github.com/lang-bmb/bmb-samples) | 예제 프로그램 | v0.3 | N/A | BMB 코드 |
| [gotgan](https://github.com/lang-bmb/gotgan) | 패키지 매니저 | v0.8 | v0.8 | v0.11 |
| [benchmark-bmb](https://github.com/lang-bmb/benchmark-bmb) | 표준 벤치마크 | v0.9 | Rust Runner | N/A |
| [action-bmb](https://github.com/lang-bmb/action-bmb) | GitHub Action | v0.7 | v0.7 | v0.11 |
| [tree-sitter-bmb](https://github.com/lang-bmb/tree-sitter-bmb) | 에디터 문법 | v0.9 | N/A | Tree-sitter |
| [vscode-bmb](https://github.com/lang-bmb/vscode-bmb) | VS Code 확장 | v0.9 | N/A | TypeScript |
| [playground](https://github.com/lang-bmb/playground) | 온라인 플레이그라운드 | v0.9 | N/A | React+WASM |
| [lang-bmb-site](https://github.com/lang-bmb/lang-bmb-site) | 공식 웹사이트 | v0.9 | N/A | Astro |

### 부트스트래핑 전략

```
Phase 1 (v0.1-v0.3): Rust로 기반 구축
  - 컴파일러 프론트엔드 (Rust)
  - 인터프리터/REPL (Rust)

Phase 2 (v0.4-v0.5): 네이티브 코드 생성 + 자기 컴파일 시작
  - LLVM 백엔드 (Rust)
  - Bootstrap 시작: lexer.bmb, parser.bmb

Phase 3 (v0.6-v0.7): 표준 라이브러리 + 도구
  - 표준 라이브러리 기초 (50개 함수)
  - 내장 도구: fmt, lsp, test

Phase 4 (v0.8-v0.9): 패키지 매니저 + 생태계
  - gotgan 패키지 매니저 (Rust)
  - 에디터 지원, 플레이그라운드, 웹사이트, 벤치마크

Phase 5 (v0.10-v0.11): 부트스트래핑 완성
  - 타입 체커 BMB 작성
  - 코드 생성기 BMB 작성
  - gotgan BMB 재작성
  - Stage 2 자기 컴파일 검증

Phase 6 (v1.0-RC): 완전한 자기 컴파일
  - 모든 핵심 도구 BMB로 재작성 완료
  - 검증 완료
```

---

## v0.1 Seed ✅ (최소 기반)

### 목표
```
Rust로 작성된 최소 컴파일러 프론트엔드
```

### 구성요소

| 구성요소 | 상태 | 설명 |
|----------|------|------|
| 렉서 | ✅ 완료 | logos 기반 토큰화 |
| 파서 | ✅ 완료 | lalrpop 기반 AST 생성 |
| AST | ✅ 완료 | 자료구조 정의 |
| 타입체커 | ✅ 완료 | 기본 타입 + 함수 검사 |
| 에러 리포터 | ✅ 완료 | ariadne 기반 |
| CLI | ✅ 완료 | `bmb check/parse/tokens` |

---

## v0.2 Sprout ✅ (검증 기반)

### 목표
```
SMT 연동으로 계약 검증 시작
```

### 구성요소

| 구성요소 | 상태 | 설명 |
|----------|------|------|
| SMT 변환기 | ✅ 완료 | AST → SMT-LIB2 |
| Z3 연동 | ✅ 완료 | 외부 프로세스 연동 |
| 반례 파서 | ✅ 완료 | SMT 결과 해석 |
| 반례 리포터 | ✅ 완료 | 사용자 친화 출력 |
| 검증 CLI | ✅ 완료 | `bmb verify` 명령어 |

### 계약 문법

| 구문 | 상태 | 설명 |
|------|------|------|
| `pre` / `post` | ✅ 완료 | 함수 전/후 조건 |
| `forall` / `exists` | ✅ 완료 | 전칭/존재 한정사 |
| `=>` (implication) | ✅ 완료 | 논리적 함축 |
| `is` pattern | ✅ 완료 | 패턴 매칭 조건 |
| `..` range | ✅ 완료 | 범위 연산자 |
| `old(expr)` | ✅ 완료 | post에서 이전 값 참조 |
| Refinement types | ✅ 완료 | `T{constraint}` 형식 |
| `@disjoint` | ⏳ 파싱만 | SMT 통합은 v0.3 |

> 📋 전체 계약 체크리스트: [docs/CONTRACT_CHECKLIST.md](CONTRACT_CHECKLIST.md)

---

## v0.3 Root ✅ (실행 기반)

### 목표
```
인터프리터로 실행 가능
```

### 구성요소

| 구성요소 | 상태 | 설명 |
|----------|------|------|
| 인터프리터 | ✅ 완료 | Tree-walking AST 실행 |
| REPL | ✅ 완료 | rustyline 기반 대화형 환경 |
| 표준 입출력 | ✅ 완료 | print, println, read_int |
| 내장 함수 | ✅ 완료 | abs, min, max, assert |
| CLI run/repl | ✅ 완료 | `bmb run`, `bmb repl` |

### 계약 확장 (v0.2에서 지연됨)

| 기능 | 상태 | 설명 |
|------|------|------|
| `@invariant` | ✅ 완료 | 루프 불변식 |
| `@decreases` | ✅ 완료 | 종료 증명 (감소 표현식) |
| `@disjoint` SMT | ✅ 완료 | 분리 조건 SMT 검증 |
| `<=>` 동치 | ✅ 완료 | 논리적 동치 연산자 |

---

## v0.4 Stem ✅ (네이티브 기반)

### 목표
```
LLVM으로 네이티브 코드 생성
```

### 구성요소

| 구성요소 | 상태 | 설명 |
|----------|------|------|
| MIR | ✅ 완료 | 중간 표현 (CFG 기반) |
| LLVM IR 생성 | ✅ 완료 | MIR → LLVM (inkwell) |
| 링커 연동 | ✅ 완료 | 플랫폼별 링커 지원 |
| 최적화 패스 | ✅ 완료 | -O0, -O2, -O3 지원 |

---

## v0.5 Branch ✅ (언어 확장 + Bootstrap)

### 목표
```
BMB로 BMB 컴파일러 재작성 시작을 위한 언어 기능 확장
```

### 구성요소

| 구성요소 | 상태 | 설명 |
|----------|------|------|
| Struct 타입 | ✅ 완료 | 구조체 정의, 생성, 필드 접근 |
| Enum 타입 | ✅ 완료 | 열거형 정의, variant 사용 |
| Pattern Matching | ✅ 완료 | match 기본, wildcard |
| String 타입 | ✅ 완료 | 문자열 리터럴, 연결, 길이 |
| Mutable 변수 | ✅ 완료 | let mut, 할당 연산자 |
| While/For 루프 | ✅ 완료 | 기본 반복문, Range |
| 모듈 시스템 | ✅ 완료 | pub 가시성, use 문 파싱 |
| 참조 타입 | ✅ 완료 | &T, &mut T 참조 |
| 배열 타입 | ✅ 완료 | [T; N] 고정 크기, 인덱스 접근 |
| 멀티 파일 | ✅ 완료 | resolver 모듈, 모듈 로딩/파싱 |
| 메서드 호출 | ✅ 완료 | expr.method(args) 지원 |
| 렉서 (BMB) | ✅ 완료 | bootstrap/lexer.bmb |
| 파서 (BMB) | ✅ 완료 | bootstrap/parser.bmb |

### Bootstrap 산출물

```
bootstrap/
├── lexer.bmb       # BMB 렉서 (8KB, 순수 함수형)
├── parser.bmb      # BMB 파서 (22KB, 재귀 하강)
├── parser_ast.bmb  # AST 출력 파서 (21KB, S-expression)
├── parser_test.bmb # 종합 테스트 (25KB, 15개 테스트)
└── README.md       # Bootstrap 문서
```

---

## v0.6 Leaf ✅ (표준 라이브러리 기초)

> 목표: 최소 실용 표준 라이브러리 (100+개 함수) - 완료

### v0.6.0 - Core 기초 (48개) ✅ 완료

| 모듈 | 함수 수 | 함수 | 설명 |
|------|---------|------|------|
| core::num | 10 | `abs`, `min`, `max`, `clamp`, `sign`, `in_range`, `diff`, `is_power_of_two`, `div_trunc`, `mod_op` | 수치 연산 + 계약 |
| core::bool | 9 | `bool_not`, `implies`, `iff`, `xor`, `to_int`, `from_int`, `select`, `all2`, `any2` | 논리 연산 |
| core::option | 12 | `is_some`, `is_none`, `unwrap_or`, `unwrap`, `map_add`, `and_then_positive`, `filter_positive`, `option_or`, `zip_sum`, `some`, `none` + enum | Option 타입 (i64 특화) |
| core::result | 17 | `is_ok`, `is_err`, `unwrap_or_result`, `unwrap_ok`, `unwrap_err`, `err_code`, `map_ok_add`, `map_err_add`, `and_then_double`, `result_or`, `ok_to_option`, `ok`, `err`, `safe_divide`, `safe_sqrt_check` + enum + 에러 코드 | Result 타입 (i64, 에러코드) |

**마일스톤**:
- [x] Option 타입 정의 및 구현 (i64 특화, 제네릭은 v0.6.1+)
- [x] Result 타입 정의 및 구현 (i64/에러코드 특화)
- [x] 기본 수치 함수 (계약 포함)
- [x] 기본 논리 함수 (계약 포함)
- [x] 테스트 파일 작성

### 산출물

```
stdlib/
├── README.md           # stdlib 문서 (100+ 함수 문서화)
├── core/
│   ├── num.bmb        # 10개 수치 함수
│   ├── bool.bmb       # 9개 논리 함수
│   ├── option.bmb     # 12개 Option 함수
│   └── result.bmb     # 17개 Result 함수
├── string/
│   └── mod.bmb        # 30+개 문자열 함수
└── array/
    └── mod.bmb        # 25+개 배열 함수
tests/stdlib/
├── test_num.bmb       # 수치 함수 테스트
├── test_option.bmb    # Option 테스트
├── test_result.bmb    # Result 테스트
├── test_string.bmb    # String 함수 테스트
└── test_array.bmb     # Array 함수 테스트
```

### 제네릭 지원 노트

현재 구현은 타입 특화 버전:
- `Option` = i64 전용 (Generic `Option<T>`는 v0.6.1+)
- `Result` = i64/에러코드 전용 (Generic `Result<T, E>`는 v0.6.1+)

제네릭 지원을 위해 필요한 언어 기능:
- [ ] 타입 파라미터 문법 (`fn foo<T>(x: T) -> T`)
- [ ] 타입 제약 (`where T: Eq`)
- [ ] 제네릭 인스턴스화 (`Option<i64>`, `Option<String>`)

### v0.6.1 - String 확장 (30+개) ✅ 완료

| 카테고리 | 함수 | 설명 |
|----------|------|------|
| 문자 분류 | `char_is_whitespace`, `char_is_digit`, `char_is_lower`, `char_is_upper`, `char_is_alpha`, `char_is_alnum` | ASCII 문자 분류 |
| 문자 변환 | `char_to_upper`, `char_to_lower`, `digit_to_int`, `int_to_digit` | 대소문자, 숫자 변환 |
| 문자열 검색 | `contains_char`, `starts_with`, `ends_with`, `index_of_char`, `count_char` | 검색 및 카운트 |
| 문자열 트림 | `find_trim_start`, `find_trim_end`, `is_blank`, `trim_start_indices`, `trim_end_indices` | 공백 처리 |
| 정수 파싱 | `parse_uint`, `parse_int`, `is_valid_int` | 문자열→정수 변환 |
| 문자열 비교 | `string_compare`, `string_eq` | 사전순 비교, 동등성 |
| 유틸리티 | `reverse_indices`, `split_first_len`, `char_count` | 기타 유틸 |

### v0.6.2 - Array 유틸리티 (25+개) ✅ 완료

> Note: Vec/HashMap은 동적 메모리가 필요하여 Rust 빌트인으로 v0.7+에서 구현 예정.
> 현재는 고정 크기 배열 `[i64; 8]` 유틸리티 제공.

| 카테고리 | 함수 | 설명 |
|----------|------|------|
| 검색 | `contains_i64`, `index_of_i64`, `count_i64` | 값 검색 및 카운트 |
| 집계 | `sum_i64`, `min_i64`, `max_i64`, `avg_i64`, `product_i64` | 합계, 최소, 최대, 평균, 곱 |
| 서술자 | `all_positive`, `all_non_negative`, `any_positive`, `any_zero`, `is_sorted_asc`, `is_sorted_desc`, `all_equal` | 조건 검사 |
| 경계 | `is_valid_index`, `clamp_index`, `wrap_index` | 인덱스 검증 |
| 범위 | `sum_range`, `count_range` | 범위 연산 |

---

## v0.7 Bloom (도구 기초) ✅ 완료

> 목표: 기본 개발 도구 내장 (Gleam 방식)

### v0.7.0 - Formatter 내장 ✅ 완료

```bash
bmb fmt              # 현재 파일 포맷
bmb fmt --check      # 포맷 검사만
bmb fmt .            # 디렉토리 전체
```

**마일스톤**:
- [x] AST → 소스코드 프린터
- [x] `bmb fmt` CLI 통합
- [ ] 포맷 규칙 정의 (bmb.toml) - 추후

**제한사항**:
- 코멘트 보존 안됨 (파싱 시 제거)
- 기본 포맷팅 규칙만 적용

### v0.7.1 - LSP 기초 ✅ 완료

```bash
bmb lsp              # LSP 서버 시작 (stdio 통신)
```

**지원 기능**:
- [x] `textDocument/diagnostic` - 렉서/파서/타입 체커 에러 표시
- [x] `textDocument/hover` - 키워드, 내장 함수, 사용자 정의 심볼 타입 정보
- [x] `textDocument/completion` - 키워드 (30+), 내장 함수, 사용자 정의 함수/구조체/열거형

**구현 세부**:
- tower-lsp 기반 비동기 LSP 서버
- 실시간 문서 분석 및 진단 발행
- 스니펫 지원 자동완성

**산출물**:
```
bmb/src/lsp/
└── mod.rs           # LSP Backend (300+ lines)
```

### v0.7.2 - 테스트 러너 ✅ 완료

```bash
bmb test              # 모든 테스트 실행
bmb test module.bmb   # 특정 파일
bmb test --filter "pattern"
bmb test -v           # 상세 출력
```

**마일스톤**:
- [x] test_ 접두사 함수 자동 탐지
- [x] 테스트 실행 및 결과 리포트
- [x] 필터링 지원 (--filter)
- [x] 상세 출력 모드 (-v, --verbose)
- [x] stdlib/test 어설션 라이브러리 (40+ 함수)

### stdlib/test 모듈 (v0.7.2) ✅ 완료

| 카테고리 | 함수 | 설명 |
|----------|------|------|
| 기본 | `assert_true`, `assert_false` | 불리언 검증 |
| 정수 | `assert_eq_i64`, `assert_ne_i64`, `assert_lt_i64`, `assert_le_i64`, `assert_gt_i64`, `assert_ge_i64`, `assert_in_range`, `assert_positive`, `assert_non_negative`, `assert_negative`, `assert_zero`, `assert_non_zero` | 정수 비교 |
| 불리언 | `assert_eq_bool`, `assert_truthy`, `assert_falsy` | 불리언 비교 |
| 문자열 | `assert_string_eq`, `assert_string_ne`, `assert_starts_with`, `assert_ends_with`, `assert_contains_char`, `assert_empty`, `assert_not_empty`, `assert_blank`, `assert_not_blank`, `assert_string_len` | 문자열 검증 |
| 배열 | `assert_array_contains`, `assert_array_not_contains`, `assert_sorted_asc`, `assert_sorted_desc`, `assert_all_equal`, `assert_all_positive`, `assert_array_sum`, `assert_array_len` | 배열 검증 |
| 복합 | `assert_all2`, `assert_all3`, `assert_any2`, `assert_any3`, `assert_xor`, `assert_implies` | 논리 조합 |
| 결과 | `count_passed`, `count_failed`, `all_passed`, `any_failed` | 테스트 결과 집계 |

### v0.7.3 - action-bmb ✅ 완료

```bash
# 사용법
- uses: lang-bmb/action-bmb@v1
  with:
    version: '0.7.3'    # 버전 지정 (기본: latest)
    command: 'check .'  # 설치 후 실행할 명령 (선택)
    cache: 'true'       # 캐싱 활성화 (기본: true)
```

**산출물**:
```
ecosystem/action-bmb/
├── action.yml           # GitHub Action 정의 (composite)
├── README.md            # 사용 설명서
├── scripts/
│   └── install.sh       # 로컬 설치 스크립트
└── examples/
    ├── basic.yml        # 기본 CI 워크플로우
    ├── verify.yml       # 계약 검증 워크플로우
    └── full-ci.yml      # 종합 CI/CD 워크플로우
```

**기능**:
- [x] 크로스 플랫폼 지원 (Linux, macOS, Windows)
- [x] 자동 캐싱으로 빠른 재실행
- [x] 버전 고정 지원
- [x] 설치 후 명령 실행 옵션
- [x] Z3 연동 가이드 (계약 검증용)

---

## v0.8 Fruit (패키지 매니저)

> 목표: 곳간(gotgan) 최소 기능

### v0.8.0 - 프로젝트 구조 ✅ 완료

```bash
gotgan new hello      # 새 프로젝트
gotgan new mylib --lib   # 라이브러리 프로젝트
gotgan init           # 현재 디렉토리 초기화
gotgan init --name myproj   # 이름 지정 초기화
```

**구현 세부**:
- clap 기반 CLI (new, init 서브커맨드)
- TOML 기반 gotgan.toml 파싱/생성
- 바이너리/라이브러리 템플릿 지원
- 프로젝트 이름 검증 (alphanumeric + _ + -)

**산출물**:
```
ecosystem/gotgan/
├── Cargo.toml          # 패키지 정의
├── README.md           # 사용 가이드
└── src/
    ├── main.rs         # CLI 엔트리포인트
    ├── config.rs       # gotgan.toml 파싱
    └── project.rs      # 프로젝트 생성/초기화
```

**생성되는 프로젝트 구조**:
```toml
# gotgan.toml
[package]
name = "hello"
version = "0.1.0"
edition = "2025"

[dependencies]

[dev-dependencies]
```

### v0.8.1 - 빌드 시스템 ✅ 완료

```bash
gotgan build          # 빌드 (LLVM 필요)
gotgan build --release   # 릴리스 빌드 (-O3)
gotgan run            # 인터프리터로 실행
gotgan run --release  # 네이티브 빌드 후 실행
gotgan check          # 타입 검사만
gotgan verify         # 계약 검증
gotgan test           # 테스트 실행
gotgan test -v        # 상세 출력
```

**구현 세부**:
- bmb 컴파일러 래퍼 (build, run, check, verify, test)
- 프로젝트 컨텍스트 자동 탐지 (gotgan.toml 기준)
- 디버그/릴리스 모드 지원
- 바이너리/라이브러리 프로젝트 구분

**산출물**:
```
ecosystem/gotgan/src/
├── build.rs          # 빌드 시스템 (200+ lines)
└── error.rs          # 통합 에러 타입
```

### v0.8.2 - 로컬 의존성 ✅ 완료

```toml
[dependencies]
mylib = { path = "../mylib" }
```

**구현 세부**:
- DependencyResolver: 로컬 경로 의존성 해석
- 순환 의존성 탐지
- 전이적 의존성 자동 해석
- 빌드 순서 결정 (의존성 → 프로젝트)

**산출물**:
```
ecosystem/gotgan/src/
└── resolver.rs       # 의존성 해석기 (200+ lines)
```

### v0.8.3 - 유틸리티 명령어 ✅ 완료

```bash
gotgan clean          # 빌드 아티팩트 정리 (target/)
gotgan tree           # 의존성 트리 출력
gotgan tree -a        # 상세 정보 포함 (경로, 소스 파일 수)
```

**구현 세부**:
- clean: target 디렉토리 완전 삭제
- tree: 프로젝트 및 의존성 트리 시각화
- 의존성 없는 경우 적절한 안내 메시지

### v0.8.4 - Lock 파일 ✅ 완료

```bash
gotgan update         # 의존성 갱신 및 lock 파일 재생성
# gotgan build 시 자동으로 gotgan.lock 생성/갱신
```

**gotgan.lock 형식**:
```toml
# This file is auto-generated by gotgan.
version = 1

[[package]]
name = "mylib"
version = "0.1.0"
path = "/absolute/path/to/mylib"
source_count = 3
```

**구현 세부**:
- 재현 가능한 빌드를 위한 의존성 잠금
- 빌드 시 자동 lock 파일 생성/갱신
- `gotgan update` 명령어로 수동 갱신
- 의존성 변경 감지 (source_count 포함)

**산출물**:
```
ecosystem/gotgan/src/
└── lock.rs           # Lock 파일 관리 (120+ lines)
```

### v0.8.5 - 의존성 추가 명령어 ✅ 완료

```bash
gotgan add mylib --path ../mylib    # 로컬 의존성 추가
gotgan add mylib --path ../mylib --dev  # 개발 의존성으로 추가
```

**구현 세부**:
- `gotgan add <name> --path <path>` 명령어
- gotgan.toml 자동 수정 (dependencies/dev-dependencies)
- 기존 의존성 존재 시 경고 후 업데이트

**Note**: 원격 레지스트리 지원은 v0.9.3 (gotgan add <name>으로 레지스트리에서 추가)

### v0.8.6 - Rust Fallback (계획)

```toml
[dependencies.rust]
regex = "1.10"
```

**특징**:
- Cargo 호환: Rust crates를 의존성으로 사용
- FFI 자동 생성
- 혼합 프로젝트 (BMB + Rust)

---

## v0.9 Harvest (생태계)

> 목표: 에디터 + 원격 패키지 + 웹 인프라

### v0.9.0 - LSP 확장 ✅ 완료

**추가 기능**:
- [x] `textDocument/formatting` - 문서 포맷팅 (AST 기반)
- [x] `textDocument/definition` - 정의로 이동 (심볼 테이블 기반)
- [x] `textDocument/references` - 참조 찾기 (AST 순회)

**구현 세부**:
- 심볼 테이블 구축 (SymbolDef, SymbolRef)
- 함수, 구조체, 열거형 정의 추적
- 표현식 내 참조 수집
- AST 프리티 프린터 (format_program, format_expr 등)

**산출물**:
```
bmb/src/lsp/
└── mod.rs           # LSP Backend (1000+ lines)
```

### v0.9.1 - tree-sitter-bmb ✅ 완료

**구현 세부**:
- grammar.js: BMB 전체 문법 정의 (500+ lines)
- highlights.scm: 구문 하이라이팅 쿼리
- folds.scm: 코드 폴딩 쿼리
- indents.scm: 자동 들여쓰기 쿼리
- Node.js 바인딩 (binding.cc, index.js)
- Rust 바인딩 (lib.rs, Cargo.toml)

**산출물**:
```
ecosystem/tree-sitter-bmb/
├── grammar.js          # 문법 정의
├── package.json        # npm 패키지
├── binding.gyp         # Node.js 네이티브
├── bindings/
│   ├── node/           # Node.js 바인딩
│   └── rust/           # Rust 바인딩
├── queries/
│   ├── highlights.scm  # 구문 하이라이팅
│   ├── folds.scm       # 코드 폴딩
│   └── indents.scm     # 자동 들여쓰기
└── README.md           # 사용 가이드
```

### v0.9.2 - vscode-bmb ✅ 완료

```
vscode-bmb/
├── package.json            # 확장 매니페스트
├── language-configuration.json  # 언어 설정
├── tsconfig.json           # TypeScript 설정
├── .eslintrc.json          # 린트 설정
├── syntaxes/
│   └── bmb.tmLanguage.json # TextMate 문법
├── src/
│   └── extension.ts        # LSP 클라이언트
└── README.md               # 사용 가이드
```

**기능**:
- 구문 하이라이팅 (TextMate)
- 에러 표시 (LSP)
- 자동완성
- Go to Definition
- Find References
- 코드 포매팅

### v0.9.3 - 원격 패키지 레지스트리 ✅ 완료

```bash
gotgan publish            # 패키지 아카이브 생성 및 배포 준비
gotgan search <query>     # 레지스트리 검색
gotgan add <name>         # 레지스트리에서 의존성 추가
gotgan add <name> -v 1.0  # 버전 지정 추가
```

**구현 내용**:
- `registry.rs` - 레지스트리 클라이언트 모듈
- GitHub 기반 패키지 레지스트리 지원
- 패키지 아카이브 생성 (.tar.gz)
- 검색 및 패키지 조회 기능

### v0.9.4 - playground ✅ 완료

```
playground/
├── package.json            # Vite + React + TypeScript
├── vite.config.ts          # Vite 설정
├── tsconfig.json           # TypeScript 설정
├── index.html              # 엔트리 HTML
├── src/
│   ├── main.tsx            # React 엔트리
│   ├── App.tsx             # 메인 컴포넌트
│   ├── compiler.ts         # BMB 인터프리터 (플레이스홀더)
│   ├── index.css           # 스타일
│   ├── components/
│   │   ├── Editor.tsx      # Monaco 에디터 래퍼
│   │   ├── Output.tsx      # 실행 결과 패널
│   │   └── Header.tsx      # 헤더 컴포넌트
│   ├── monaco/
│   │   └── bmb-language.ts # Monaco BMB 언어 정의
│   └── utils/
│       └── sharing.ts      # URL 공유 (LZ-String)
└── README.md               # 사용 가이드
```

**기능**:
- Monaco 에디터 + BMB 구문 하이라이팅
- 플레이스홀더 인터프리터 (WASM 타겟 준비 전)
- 계약 검증 시각화
- URL 공유 (LZ-String 압축)
- 반응형 디자인

**Note**: WASM 기반 실행은 BMB WASM 타겟 완성 후 연동 예정

### v0.9.5 - lang-bmb-site ✅ 완료

```
lang-bmb-site/
├── package.json            # Astro 4.x 프로젝트
├── astro.config.mjs        # Astro 설정
├── tsconfig.json           # TypeScript 설정
├── public/
│   └── favicon.svg         # 파비콘
├── src/
│   ├── components/
│   │   ├── Header.astro    # 네비게이션
│   │   ├── Footer.astro    # 푸터
│   │   └── CodeBlock.astro # BMB 코드 하이라이팅
│   ├── layouts/
│   │   ├── Base.astro      # 기본 레이아웃
│   │   └── Docs.astro      # 문서 레이아웃
│   ├── pages/
│   │   ├── index.astro     # 랜딩 페이지
│   │   ├── download.astro  # 다운로드 페이지
│   │   ├── changes.astro   # 변경로그
│   │   ├── docs/index.astro
│   │   └── blog/index.astro
│   └── styles/
│       └── global.css      # 전역 스타일
└── content/                # 마크다운 콘텐츠 (추후)
```

**페이지**:
- `/` - Landing page (Hero, Features, Quick Start)
- `/docs` - Documentation (Introduction)
- `/download` - 설치 가이드 (Quick install, Binaries)
- `/changes` - Changelog (Version timeline)
- `/blog` - 개발 블로그 (Posts list)

### v0.9.6 - benchmark-bmb ✅ 완료

```
benchmark-bmb/
├── benches/
│   ├── compute/
│   │   ├── fibonacci/        # 재귀 함수 호출
│   │   │   ├── c/main.c
│   │   │   └── bmb/main.bmb
│   │   └── n_body/           # N-body 시뮬레이션
│   │       ├── c/main.c
│   │       └── bmb/main.bmb
│   └── contract/
│       └── bounds_check/     # 경계 검사 제거
│           ├── c/main.c
│           └── bmb/main.bmb
├── runner/
│   ├── Cargo.toml
│   └── src/main.rs           # 벤치마크 러너
└── results/
```

**구현 완료**:
- Rust 기반 벤치마크 러너 (CLI)
- run, list, new, compare, validate 명령어
- fibonacci 벤치마크 (compute)
- n_body 벤치마크 (compute, C 완료)
- bounds_check 벤치마크 (contract)

**목표**: BMB >= C -O3

---

## v0.10 Sunrise (Bootstrap 진행)

> 목표: BMB로 BMB 도구 재작성 시작

### v0.10.0 - 타입 체커 BMB 작성 ✅ 완료

```
bootstrap/
├── lexer.bmb       # ✅ 완료 (8KB)
├── parser.bmb      # ✅ 완료 (22KB)
├── parser_ast.bmb  # ✅ 완료 (21KB)
├── parser_test.bmb # ✅ 완료 (25KB)
├── types.bmb       # ✅ 완료 (15KB) - 신규
└── README.md
```

**types.bmb 구현 내용**:
- 타입 인코딩: `kind * 1000` (i32=1000, i64=2000, bool=4000, String=5000, Unit=6000)
- 환경: 문자열 기반 name:type 쌍, 선형 검색
- 내장 함수 시그니처 (println, print, assert, read_int, abs, min, max)
- 연산자 타입 검사 (+, -, *, /, %, ==, !=, <, >, <=, >=, and, or, not)
- if-then-else 타입 검사 (조건 bool, 분기 일치)
- let 바인딩 타입 검사
- 함수 호출 타입 검사 (arity + 인자 타입)
- 37개 테스트 통과

### v0.10.1 - MIR 기초 정의 ✅ 완료

```
bootstrap/
├── lexer.bmb       # ✅ 완료 (8KB)
├── parser.bmb      # ✅ 완료 (22KB)
├── parser_ast.bmb  # ✅ 완료 (21KB)
├── parser_test.bmb # ✅ 완료 (25KB)
├── types.bmb       # ✅ 완료 (15KB)
├── mir.bmb         # ✅ 완료 (18KB) - 신규
└── README.md
```

**mir.bmb 구현 내용**:
- 명령어 인코딩: `kind * 1000` (CONST=1000, COPY=2000, BINOP=3000, UNARY=4000, CALL=5000)
- 종료자 인코딩: RETURN=10000, GOTO=11000, BRANCH=12000
- 이항/단항 연산자 인코딩 및 심볼 출력
- 상수 인코딩: `I:42`, `B:1`, `S:hello`, `U`
- 플레이스 인코딩: `%name`, `%_t0` (임시 변수)
- 텍스트 기반 MIR 출력 포맷
- 예제 lowering 함수 (add, max with if)
- 46개 테스트 통과

### v0.10.2 - AST → MIR Lowering ✅ 완료

```
bootstrap/
├── lexer.bmb       # ✅ 완료 (8KB)
├── parser.bmb      # ✅ 완료 (22KB)
├── parser_ast.bmb  # ✅ 완료 (21KB)
├── parser_test.bmb # ✅ 완료 (25KB)
├── types.bmb       # ✅ 완료 (15KB)
├── mir.bmb         # ✅ 완료 (18KB)
├── lowering.bmb    # ✅ 완료 (25KB) - 신규
└── README.md
```

**lowering.bmb 구현 내용**:
- S-expression AST 파싱 (parser_ast.bmb 출력 형식)
- 표현식 lowering: int, bool, var, binop, unary, if, let, call
- 함수 lowering (기본 블록 생성)
- 프로그램 lowering (다중 함수)
- Pack/unpack 결과 형식: `temp:block:place:text`
- 41개 테스트 통과 (95%)

**지원 변환:**
```lisp
(int 42)              →  %_t0 = const I:42
(var <x>)             →  %x
(op + (var <a>) (var <b>)) →  %_t0 = + %a, %b
(if (var <c>) (int 1) (int 2)) →  branch %c, then_0, else_0 ...
(call <foo> (var <a>))        →  %_t0 = call foo(%a)
```

**Note**: 원래 계획된 표준 라이브러리 확장(io, fs, net, time)은 OS FFI가 필요하여 bootstrap 범위를 벗어남. v0.11+ Rust FFI 통합 시 추가 예정.

### v0.10.3 - End-to-End 파이프라인 ✅ 완료

```
bootstrap/
├── lexer.bmb       # ✅ 완료 (8KB)
├── parser.bmb      # ✅ 완료 (22KB)
├── parser_ast.bmb  # ✅ 완료 (21KB)
├── parser_test.bmb # ✅ 완료 (25KB)
├── types.bmb       # ✅ 완료 (15KB)
├── mir.bmb         # ✅ 완료 (18KB)
├── lowering.bmb    # ✅ 완료 (25KB)
├── pipeline.bmb    # ✅ 완료 (25KB) - 신규
└── README.md
```

**pipeline.bmb 구현 내용**:
- 통합 Source → AST → MIR 파이프라인 데모
- S-expression AST 생성 (parser_ast.bmb 패턴)
- MIR 텍스트 생성 (lowering.bmb 패턴)
- 표현식 레벨 컴파일: `compile_expr(src) -> MIR text`
- 14개 테스트 통과

**지원 컴파일:**
```bmb
compile_expr("42")         →  "%_t0 = const I:42"
compile_expr("a + b")      →  "%_t0 = + %a, %b"
compile_expr("a * b + c")  →  "%_t0 = * %a, %b|%_t1 = + %_t0, %c"
compile_expr("-x")         →  "%_t0 = neg %x"
compile_expr("not b")      →  "%_t0 = not %b"
```

**Note**: gotgan migrate (Rust crate 분석/마이그레이션)는 Rust 전용 도구로 v0.11+ 계획.

### v0.10.4 - MIR → C 코드 생성 ✅ 완료 (레거시)

> ⚠️ **레거시**: C 코드 경로는 LLVM IR 경로로 대체됨 (v0.10.5+)
> BMB 철학 "최대 성능, C/Rust 초월"에 부합하는 LLVM IR 직접 생성으로 전환

```
bootstrap/
├── lexer.bmb       # ✅ 완료 (8KB)
├── parser.bmb      # ✅ 완료 (22KB)
├── parser_ast.bmb  # ✅ 완료 (21KB)
├── parser_test.bmb # ✅ 완료 (25KB)
├── types.bmb       # ✅ 완료 (15KB)
├── mir.bmb         # ✅ 완료 (18KB)
├── lowering.bmb    # ✅ 완료 (25KB)
├── pipeline.bmb    # ✅ 완료 (25KB)
└── codegen.bmb     # ✅ 완료 (18KB) - C 백엔드 (레거시)
```

### v0.10.5 - LLVM IR 기초 (타입, 상수, 산술) ✅ 완료

```
bootstrap/
└── llvm_ir.bmb     # LLVM IR 텍스트 생성 (35KB)
```

**구현 내용:**
- LLVM IR 타입 매핑: i64 → i64, i32 → i32, bool → i1, unit → void
- 상수 생성: 정수, 불리언
- 산술 연산: add, sub, mul, sdiv, srem
- 비교 연산: icmp eq/ne/slt/sgt/sle/sge
- 논리 연산: and, or, xor
- 단항 연산: sub (neg), xor -1 (not)
- 31개 테스트 통과

**LLVM IR 생성 예시:**
```llvm
; 상수
%_t0 = add i64 0, 42           ; const I:42

; 산술 연산
%_t0 = add i64 %a, %b          ; +
%_t0 = sub i64 %a, %b          ; -
%_t0 = mul i64 %a, %b          ; *
%_t0 = sdiv i64 %a, %b         ; /
%_t0 = srem i64 %a, %b         ; %

; 비교 연산
%_t0 = icmp eq i64 %a, %b      ; ==
%_t0 = icmp slt i64 %a, %b     ; <
```

### v0.10.6 - LLVM IR 제어 흐름 (branch, label, phi) ✅ 완료

**구현 내용:**
- 레이블 생성: `entry:`, `then_0:`, `else_0:`, `merge_0:`
- 무조건 분기: `br label %target`
- 조건 분기: `br i1 %cond, label %then, label %else`
- PHI 노드: `%result = phi i64 [ %a, %then ], [ %b, %else ]`
- 반환문: `ret i64 %value`, `ret void`
- 20개 테스트 통과

**LLVM IR 제어 흐름 예시:**
```llvm
entry:
  %cond = icmp sgt i64 %a, %b
  br i1 %cond, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  br label %merge_0
merge_0:
  %result = phi i64 [ %a, %then_0 ], [ %b, %else_0 ]
```

### v0.10.7 - LLVM IR 함수/프로그램 생성 ✅ 완료

**구현 내용:**
- 함수 정의: `define i64 @name(i64 %a, i64 %b) { ... }`
- 함수 호출: `%_t0 = call i64 @foo(i64 %a)`
- 매개변수 변환: MIR → LLVM 매개변수 형식
- MIR 함수 헤더 파싱: `|` 구분자로 name/ret_type/params 추출
- 완전한 함수 변환: MIR → LLVM IR 함수
- 24개 테스트 통과

**LLVM IR 함수 예시:**
```llvm
declare i64 @println(i64)

define i64 @add(i64 %a, i64 %b) {
entry:
  %_t0 = add i64 %a, %b
  ret i64 %_t0
}

define i64 @max(i64 %a, i64 %b) {
entry:
  %cond = icmp sgt i64 %a, %b
  br i1 %cond, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  br label %merge_0
merge_0:
  %result = phi i64 [ %a, %then_0 ], [ %b, %else_0 ]
  ret i64 %result
}
```

### v0.10.8 - Full Compiler Pipeline 통합 ✅ 완료

**구현 내용:**
- 프로그램 생성: `||` 구분자로 다중 함수 지원
- 모듈 헤더: ModuleID, target triple
- 런타임 선언: println, abs, min, max extern 선언
- End-to-End 예제: example_add_mir/llvm, example_max_mir/llvm
- 패턴 검증: has_define, has_entry_label, has_ret, has_pattern
- 18개 테스트 통과 (총 93개)

**파이프라인:**
```
BMB Source
    ↓ lexer.bmb
  Tokens
    ↓ parser_ast.bmb
  S-expr AST
    ↓ lowering.bmb
  MIR Text
    ↓ llvm_ir.bmb
  LLVM IR (.ll)
    ↓ llc (외부)
  Native Binary
```

**검증:**
```bash
# BMB 부트스트랩 컴파일러로 LLVM IR 생성
bmb run bootstrap/compiler.bmb < test.bmb > test.ll

# LLVM 도구로 컴파일
llc test.ll -o test.s
gcc test.s -o test

# 실행 검증
./test
```

### v0.10.9 - Unified Compiler Entry Point ✅ 완료

**구현 내용:**
- 통합 컴파일러 진입점: `compiler.bmb` (42KB)
- Source → AST → MIR → LLVM IR 완전 파이프라인
- 모듈 조립: header + runtime declarations + functions
- `compile_program(source) -> LLVM IR module`
- `compile_function(source) -> LLVM IR function`
- 에러 처리: `is_compile_error()`, `get_error_type()`
- 8개 테스트 통과 (pre-computed AST 기반)

**API:**
```bmb
-- 전체 컴파일
fn compile_program(source: String) -> String

-- 단일 함수 컴파일
fn compile_function(source: String) -> String

-- 모듈 생성
fn gen_module_header() -> String
fn gen_runtime_decls() -> String
```

**제한사항:**
- 인터프리터 스택 제한으로 인해 테스트는 pre-computed AST 사용
- parse_source() 호출은 별도 파일에서만 사용 가능

### v0.10.10 - Integration Testing with LLVM Toolchain ✅ 완료

**구현 내용:**
- C 런타임 라이브러리 (`runtime/runtime.c`) 확장
- Bootstrap 런타임 함수: `println`, `abs`, `min`, `max`
- 테스트 LLVM IR 파일: `test_add.ll`, `test_max.ll`
- 검증 스크립트: `validate_llvm_ir.sh`
- Windows 빌드 스크립트: `build_test.ps1`
- LLVM IR 구문 검증 및 오브젝트 파일 컴파일 확인
- 심볼 검증: `llvm-nm`으로 함수 심볼 확인

**파일 구조:**
```
runtime/
├── runtime.c           # C 런타임 (bootstrap 함수 포함)
├── test_add.ll         # 간단한 LLVM IR 테스트 (add 함수)
├── test_max.ll         # 복잡한 LLVM IR 테스트 (if-then-else, PHI)
├── validate_llvm_ir.sh # Shell 검증 스크립트
└── build_test.ps1      # PowerShell 빌드 스크립트
```

**검증 결과:**
```bash
$ bash validate_llvm_ir.sh
[1/3] Validating LLVM IR syntax...
  ✓ LLVM IR syntax valid
[2/3] Compiling to object file...
  ✓ Object file created (724 bytes)
[3/3] Verifying symbols...
  ✓ Symbol 'add' found (defined)
  ✓ Symbol 'main' found (defined)
  ✓ Symbol 'println' found (external reference)
=== All validations passed ===
```

**test_max.ll 예시 (제어 흐름 + PHI):**
```llvm
define i64 @max_manual(i64 %a, i64 %b) {
entry:
  %cmp = icmp sgt i64 %a, %b
  br i1 %cmp, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  br label %merge_0
merge_0:
  %result = phi i64 [ %a, %then_0 ], [ %b, %else_0 ]
  ret i64 %result
}
```

### v0.10.11 - End-to-End Program Compilation Validation ✅ 완료

**구현 내용:**
- 종합 테스트 프로그램: `fibonacci.bmb`, `factorial.bmb`
- 수작성 LLVM IR: bootstrap 패턴 검증용 `.ll` 파일
- 인터프리터 vs 네이티브 결과 비교 프레임워크
- 자동화 스크립트: `validate_all.sh`, `run_test.sh`, `run_test.ps1`
- 심볼 검증: `llvm-nm`으로 함수 심볼 확인

**파일 구조:**
```
examples/bootstrap_test/
├── fibonacci.bmb       # 재귀 피보나치 (fib(10) = 55)
├── fibonacci.ll        # 수작성 LLVM IR (PHI 노드 포함)
├── factorial.bmb       # 반복 팩토리얼 (factorial(5) = 120)
├── factorial.ll        # 수작성 LLVM IR (꼬리 재귀 패턴)
├── validate_all.sh     # LLVM IR 컴파일 검증
├── run_test.sh         # Unix e2e 테스트
└── run_test.ps1        # Windows e2e 테스트
```

**검증 결과:**
```bash
# LLVM IR 컴파일
$ clang -c fibonacci.ll -o fibonacci.obj
$ llvm-nm fibonacci.obj
00000000 T fib
00000060 T main
         U println

# 인터프리터 결과
$ bmb run examples/bootstrap_test/fibonacci.bmb
55

$ bmb run examples/bootstrap_test/factorial.bmb
120
```

**fibonacci.ll 핵심 (재귀 + PHI):**
```llvm
define i64 @fib(i64 %n) {
entry:
  %cmp = icmp sle i64 %n, 1
  br i1 %cmp, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  %n_minus_1 = sub i64 %n, 1
  %fib_n1 = call i64 @fib(i64 %n_minus_1)
  %n_minus_2 = sub i64 %n, 2
  %fib_n2 = call i64 @fib(i64 %n_minus_2)
  %sum = add i64 %fib_n1, %fib_n2
  br label %merge_0
merge_0:
  %result = phi i64 [ %n, %then_0 ], [ %sum, %else_0 ]
  ret i64 %result
}
```

### v0.10.12 - Text-based LLVM IR Backend ✅ 완료

**구현 내용:**
- MIR → LLVM IR 텍스트 생성기: `bmb/src/codegen/llvm_text.rs`
- PHI 노드 지원: SSA 형식 준수를 위한 MIR 확장
- Windows 빌드 파이프라인: UCRT/MSVC 헤더 탐지, lld-link 통합
- 외부 clang 의존: inkwell/LLVM C API 없이 .ll → .exe 컴파일

**핵심 특징:**
```rust
// TextCodeGen - LLVM IR 텍스트 생성
pub struct TextCodeGen {
    target_triple: String,
}

impl TextCodeGen {
    pub fn generate(&self, program: &MirProgram) -> TextCodeGenResult<String>
    // MIR → LLVM IR 문자열 변환
}
```

**빌드 파이프라인:**
```
BMB Source → AST → MIR → LLVM IR (.ll) → clang → Object (.obj) → lld-link → Executable
```

**PHI 노드 예시 (생성된 LLVM IR):**
```llvm
define i64 @fib(i64 %n) {
entry:
  %_t0 = icmp sle i64 %n, 1
  br i1 %_t0, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  %_t1 = sub i64 %n, 1
  %_t2 = call i64 @fib(i64 %_t1)
  %_t3 = sub i64 %n, 2
  %_t4 = call i64 @fib(i64 %_t3)
  %_t5 = add i64 %_t2, %_t4
  br label %merge_0
merge_0:
  %result = phi i64 [ %n, %then_0 ], [ %_t5, %else_0 ]
  ret i64 %result
}
```

**검증 결과:**
```powershell
# fibonacci.bmb 컴파일 및 실행
PS> cargo run --release -- build examples/bootstrap_test/fibonacci.bmb
PS> .\examples\bootstrap_test\fibonacci.exe
55

# factorial.bmb 컴파일 및 실행
PS> cargo run --release -- build examples/bootstrap_test/factorial.bmb
PS> .\examples\bootstrap_test\factorial.exe
120
```

**Windows 툴체인 통합:**
- UCRT 헤더 경로 자동 탐지
- MSVC 헤더/라이브러리 경로 자동 탐지
- lld-link 직접 호출 (MSVC 링커 우회)
- runtime.c 함수 충돌 해결 (`abs` → `bmb_abs`)

### v0.10.13 - 부트스트랩 컴포넌트 패키지화 준비 ✅ 완료

**목표**: Bootstrap BMB 코드에서 재사용 가능한 컴포넌트 식별 및 분리

**결정**: gotgan 별도 패키지 대신 stdlib 확장 채택 (프로젝트 철학 정렬)
- 이유: 합성 가능성, 계약 기반 설계, 중복 방지
- 기존 stdlib/string과의 일관성 유지

**분석 완료:**
| 컴포넌트 | 출처 | 배치 |
|----------|------|------|
| String 유틸리티 | llvm_ir.bmb | stdlib/string (확장) |
| 파싱 헬퍼 | llvm_ir.bmb, parser.bmb | stdlib/parse (신규) |

### v0.10.14 - stdlib/string 확장 ✅ 완료

**추가된 함수:**
```bmb
-- Integer to String Conversion
pub fn digit_char(d: i64) -> String
  pre d >= 0 and d <= 9
  post ret.len() == 1;

pub fn int_to_string(n: i64) -> String
  post n >= 0 => ret.len() >= 1
  post n < 0 => ret.len() >= 2;

-- ASCII Code to String
pub fn char_to_string(c: i64) -> String
  pre c >= 32 and c <= 126
  post ret.len() == 1;
```

### v0.10.15 - stdlib/parse 모듈 생성 ✅ 완료

**새 모듈**: Position-based parsing utilities (20+ functions)

```bmb
-- Whitespace Handling
pub fn skip_ws(s: String, pos: i64) -> i64;
pub fn skip_all_ws(s: String, pos: i64) -> i64;

-- Character Search
pub fn find_char(s: String, c: i64, pos: i64) -> i64;
pub fn find_pipe(s: String, pos: i64) -> i64;

-- Token Reading
pub fn read_until_ws(s: String, pos: i64) -> String;
pub fn read_ident(s: String, pos: i64) -> String;

-- Field Extraction (pipe-delimited)
pub fn extract_field(s: String, index: i64) -> String;
pub fn count_fields(s: String) -> i64;

-- Pattern Matching
pub fn find_arrow(s: String, pos: i64) -> i64;
pub fn has_pattern(s: String, pat: String) -> bool;
```

**산출물:**
```
stdlib/
├── string/mod.bmb     # 40+ functions (확장됨)
├── parse/mod.bmb      # 20+ functions (신규)
tests/stdlib/
├── test_string.bmb    # string 테스트
└── test_parse.bmb     # parse 테스트 (신규)
```

---

## v0.11 Dawn (AI-Native gotgan + Bootstrap)

> 목표: AI-네이티브 패키지 매니저 + Stage 2 컴파일러 + BMB 재작성
> 참고: [GOTGAN_DESIGN.md](./GOTGAN_DESIGN.md)

### v0.11.0 - BMB 컴파일러 완성

```bash
# Rust 컴파일러 (Stage 0)
cargo build --release

# BMB로 작성된 컴파일러 (Stage 1)
./bmb-rust build bmb-compiler -> bmb-stage1

# Stage 1으로 자기 컴파일 (Stage 2)
./bmb-stage1 build bmb-compiler -> bmb-stage2

# 검증: Stage 2가 동일한 결과 생성
./bmb-stage2 build bmb-compiler -> bmb-stage3
diff bmb-stage2 bmb-stage3  # 동일해야 함
```

### v0.11.1 - gotgan BMB 재작성

| 구성요소 | Rust → BMB |
|----------|------------|
| CLI | ✅ |
| 의존성 해결 | ✅ |
| 빌드 시스템 | ✅ |
| 레지스트리 클라이언트 | ✅ |

### v0.11.2 - action-bmb BMB 재작성

- GitHub Action 로직을 BMB로 재작성
- Rust 대신 BMB 바이너리 사용

### v0.11.3 - 표준 라이브러리 완성 (200개)

| 모듈 | 함수 수 | 설명 |
|------|---------|------|
| core | 50+ | 기본 타입, 연산 |
| collections | 30+ | Vec, Map, Set |
| string | 25+ | 문자열 처리 |
| io | 20+ | 파일, 스트림 |
| fs | 15+ | 파일 시스템 |
| net | 15+ | 네트워크 |
| async | 20+ | 비동기 |
| math | 30+ | 수학 함수 |

### v0.11.4 - BMBX 번들 포맷 구현 ✅

**AI-Native Package Bundle:**
```
package.bmbx
├── manifest.toml      # 패키지 메타데이터
├── contracts.json     # 모든 계약의 JSON 표현
├── symbols.json       # AI 탐색용 심볼 인덱스
├── types.json         # 타입 시그니처
├── src/               # 소스 코드
└── bin/               # 컴파일된 바이너리 (multi-target)
```

**구현 (Rust):**
- `gotgan bundle` 명령어 - contracts.json, symbols.json, types.json 생성
- `gotgan explore` 명령어 - 심볼/계약 탐색 및 JSON 출력
- ContractsJson, SymbolsJson, TypesJson 스키마 정의
- AI 힌트 자동 추론 (use_when, semantic_tags)

### v0.11.5 - 계약 기반 의존성 검사 ✅

```toml
[dependencies]
math = { version = "^1.0", contracts = ["divide.pre: b != 0"] }
```

**기능 (구현됨):**
- ✅ 계약 호환성 자동 검사 (`gotgan compat`)
- ✅ 계약 약화/강화 감지 (ContractChange enum)
- ✅ 비호환 변경 경고 (Breaking changes: AddedPre, RemovedPost)
- ✅ 호환 변경 허용 (RemovedPre, AddedPost)

**계약 변경 규칙:**
| 변경 | 타입 | 설명 |
|------|------|------|
| pre 제거 | ✅ 허용 | 더 관대해짐 |
| pre 추가 | ⚠️ Breaking | 더 제한적 |
| post 추가 | ✅ 허용 | 더 많은 보장 |
| post 제거 | ⚠️ Breaking | 보장 감소 |

### v0.11.6 - AI 패키지 탐색 ✅

```bash
# 심볼 탐색
$ gotgan explore --symbols

# 계약 확인
$ gotgan explore --contracts

# JSON 출력 (AI 파싱용)
$ gotgan explore --json

# 필터링
$ gotgan explore --filter "parse" --contracts
```

**구현됨:**
- `gotgan explore` - 심볼 인덱스 출력
- `--contracts` - 계약 정보 출력
- `--types` - 타입 정보 출력
- `--json` - JSON 형식 출력 (AI 파싱용)
- `--filter` - 이름 패턴 필터링

### v0.11.7 - 단일 파일 번들 ✅

```bash
# 모든 의존성을 하나의 .bmbx로 번들 (tar.gz 압축)
$ gotgan bundle --single-file
✓ Generated single-file bundle: "target/bmbx/package-0.1.0.bmbx"
  Package: package v0.1.0
  Contracts: 2 functions, 0 types
  Exports: 5 symbols
```

**구현됨:**
- `gotgan bundle --single-file` - tar.gz 형식 .bmbx 아카이브 생성
- 아카이브 내용: manifest.toml, contracts.json, symbols.json, types.json, src/
- flate2 + tar 크레이트 활용

**Note:** `--preserve-contracts` 옵션은 추후 구현 예정.

---

## v0.12 Horizon (WASM 듀얼 타깃)

> 목표: LLVM 네이티브 + WASM 포터블 동시 지원

### v0.12.0 - MIR → WASM IR 변환기 ✅

```
MIR (공통 중간 표현)
    ↓
    ├─────────────────────────────┐
    ↓                             ↓
LLVM IR Generator           WASM IR Generator
    ↓                             ↓
Native Binary               .wasm
```

**구현됨:**
- `wasm_text.rs` - WAT (WebAssembly Text Format) 생성기
- 3가지 타겟 환경 지원:
  - `wasi` - WASI 표준 런타임 (기본값)
  - `browser` - 브라우저/JavaScript 환경
  - `standalone` - 순수 계산 (외부 의존 없음)
- CLI 옵션: `bmb build --emit-wasm --wasm-target=wasi`

**생성 예시:**
```bash
$ bmb build add.bmb --emit-wasm -v
Compiling add.bmb to WASM...
Generated: add.wat
  Target: Wasi
  Size: 1382 bytes
```

**출력 형식:**
```wat
(module
  ;; Generated by BMB compiler (v0.12.0)
  (memory (export "memory") 1)
  (import "wasi_snapshot_preview1" "fd_write" ...)

  (func $add (param $a i32) (param $b i32) (result i32)
    (local $_t0 i32)
    local.get $a
    local.get $b
    i32.add
    local.set $_t0
    local.get $_t0
    return
  )

  (export "add" (func $add))
)
```

### v0.12.1 - WASI 런타임 바인딩 ✅

**구현 방식:** BMB 언어에 extern fn이 없으므로, WASM 코드젠에서 직접 런타임 함수 생성

**WASI 런타임 함수 (wasm_text.rs):**
```wat
;; I/O 함수
(func $println (param $val i64))  ;; i64를 stdout에 출력 (개행 포함)
(func $print (param $val i64))    ;; i64를 stdout에 출력 (개행 없음)

;; 프로세스 제어
(func $exit (param $code i32))    ;; 프로세스 종료
(func $assert (param $cond i32))  ;; 조건 검사, 실패시 종료

;; 내부 헬퍼
(func $i64_to_str (param $val i64) (result i32))  ;; i64→문자열 변환
```

**WASI 임포트:**
- `fd_write` - 파일 디스크립터에 쓰기 (stdout 출력용)
- `proc_exit` - 프로세스 종료

### v0.12.2 - 브라우저 런타임 바인딩 ✅

**브라우저 런타임 함수:**
```wat
;; I/O 함수 (console.log 사용)
(func $println (param $val i64))  ;; console.log로 출력
(func $print (param $val i64))    ;; console.log로 출력

;; 프로세스 제어
(func $exit (param $code i32))    ;; unreachable 트랩
(func $assert (param $cond i32))  ;; unreachable 트랩 (실패시)
```

**브라우저 임포트:**
- `console_log` - JavaScript console.log 바인딩
- `console_log_f64` - 부동소수점 출력용

**테스트:**
```rust
#[test] fn test_wasi_runtime_functions() { ... }      // 7개 assertion
#[test] fn test_browser_runtime_functions() { ... }   // 6개 assertion
#[test] fn test_standalone_no_runtime() { ... }       // 3개 assertion
```

### v0.12.3 - 조건부 컴파일

```bmb
#[cfg(target = "wasm32")]
fn print(s: String) = js_console_log(s);

#[cfg(target = "native")]
fn print(s: String) = libc_puts(s);
```

### v0.12.4 - 듀얼 타깃 빌드

```toml
# gotgan.toml
[targets]
native = ["x86_64-linux", "aarch64-darwin"]
wasm = ["wasm32-unknown-unknown", "wasm32-wasi"]
```

```bash
$ gotgan build --all-targets
→ target/x86_64-linux/release/app
→ target/wasm32/release/app.wasm
```

---

## v0.13 Summit (생태계 완성)

> 목표: 완전한 AI-네이티브 생태계

### v0.13.0 - gotgan MCP 서버

```json
{
  "tools": [
    "search_packages",
    "explore_package",
    "generate_example",
    "verify_contracts"
  ]
}
```

### v0.13.1 - 레지스트리 자동화

- 패키지 자동 검증
- 계약 커버리지 점수
- AI 인덱스 자동 생성
- CDN 배포

### v0.13.2 - 패키지 품질 점수

| 지표 | 가중치 |
|------|--------|
| 계약 커버리지 | 30% |
| 테스트 커버리지 | 25% |
| 문서화 | 20% |
| AI 힌트 | 15% |
| 다운로드 수 | 10% |

### v0.13.3 - 의존성 그래프 시각화

- 웹 기반 의존성 탐색기
- 계약 전파 시각화
- 호환성 매트릭스

---

## v1.0-RC Golden (부트스트래핑 완료)

> 목표: 완전한 자기 컴파일 + 검증 + 안정성 약속

### 부트스트래핑 체크리스트

| 구성요소 | Rust 버전 | BMB 버전 | 검증 |
|----------|-----------|----------|------|
| 렉서 | ✅ | ✅ | ✅ |
| 파서 | ✅ | ✅ | ✅ |
| 타입체커 | ✅ | ✅ | ✅ |
| SMT 변환기 | ✅ | ✅ | ✅ |
| MIR | ✅ | ✅ | ✅ |
| LLVM IR 생성 | ✅ | ✅ | ✅ |
| 곳간 | ✅ | ✅ | ✅ |
| 표준 라이브러리 | - | ✅ | ✅ |

### 전체 검증 매트릭스

| 검증 항목 | 방법 | 기준 |
|-----------|------|------|
| 컴파일러 정확성 | 자기 컴파일 | Stage2 == Stage3 |
| 계약 검증 | SMT | 모든 계약 Verified |
| 테스트 | 테스트 스위트 | 100% 통과 |
| 벤치마크 | benchmark-bmb | BMB >= C -O3 |
| 메모리 안전 | Valgrind | 에러 0 |

### 릴리스 체크리스트

- [ ] 부트스트래핑 3단계 통과
- [ ] 모든 계약 검증됨 (10,000+ 계약)
- [ ] 테스트 100% 통과 (5,000+ 테스트)
- [ ] 벤치마크 목표 달성 (BMB >= C)
- [ ] 문서 100% 완료
- [ ] 플레이그라운드 작동
- [ ] 패키지 레지스트리 작동
- [ ] VS Code 확장 배포
- [ ] 홈페이지 배포

---

## 생태계 타임라인

| 레포지토리 | 시작 | 최소 기능 | Rust 완성 | BMB 재작성 |
|------------|------|-----------|-----------|------------|
| lang-bmb | v0.1 ✅ | v0.1 ✅ | v0.5 ✅ | v0.11 |
| bmb-samples | v0.3 | v0.6 | N/A | BMB 코드 |
| action-bmb | v0.7 | v0.7.3 | v0.9 | v0.11 |
| gotgan | v0.8 | v0.8.5 ✅ | v0.9 | v0.11 |
| tree-sitter-bmb | v0.9 | v0.9.1 | N/A | Tree-sitter |
| vscode-bmb | v0.9 | v0.9.2 | N/A | TypeScript |
| playground | v0.9 | v0.9.4 | N/A | React+WASM |
| lang-bmb-site | v0.9 | v0.9.5 | N/A | Astro |
| benchmark-bmb | v0.9 | v0.9.6 | Rust Runner | N/A |

---

## 난이도 진행 (완만)

```
v0.5 → v0.6.0: 핵심 타입 20개 (📈 적당)
v0.6.0 → v0.6.1: 문자열 15개 (📈 적당)
v0.6.1 → v0.6.2: 컬렉션 15개 (📈 적당)
v0.6.2 → v0.7.0: 포매터 (📈 적당)
v0.7.0 → v0.7.1: LSP 기초 (📈 적당)
v0.7.1 → v0.7.2: 테스트 러너 (📈 적당)
v0.7.2 → v0.7.3: action-bmb (📈 적당)
v0.7.3 → v0.8.0: 곳간 기초 (📈 적당) ✅
v0.8.0 → v0.8.1: 빌드 시스템 (📈 적당) ✅
v0.8.1 → v0.8.2: 로컬 의존성 (📈 적당) ✅
v0.8.2 → v0.8.3: 유틸리티 명령어 (📈 적당) ✅
v0.8.3 → v0.8.4: Lock 파일 (📈 적당) ✅
v0.8.4 → v0.8.5: 의존성 추가 (📈 적당) ✅
v0.8.5 → v0.9.0: LSP 확장 (📈 적당) ✅
v0.9.0 → v0.9.1: tree-sitter-bmb (📈 적당) ✅
v0.9.1 → v0.9.2: vscode-bmb (📈 적당) ✅
v0.9.2 → v0.9.3: 원격 레지스트리 (📈 적당) ✅
v0.9.3 → v0.9.4: playground (📈 적당) ✅
v0.9.4 → v0.9.5: lang-bmb-site (📈 적당) ✅
v0.9.5 → v0.9.6: benchmark-bmb (📈 적당) ✅
v0.9.6 → v0.10.0: 타입 체커 BMB (📈 적당) ✅
v0.10.0 → v0.10.1: MIR 기초 정의 (📈 적당) ✅
v0.10.1 → v0.10.2: AST→MIR Lowering (📈 적당) ✅
v0.10.2 → v0.10.3: End-to-End 파이프라인 (📈 적당) ✅
v0.10.3 → v0.10.4: MIR→C 코드 생성 (📈 적당) ✅ (레거시)
v0.10.4 → v0.10.5: LLVM IR 기초 (📈 적당) ✅
v0.10.5 → v0.10.6: LLVM IR 제어 흐름 (📈 적당) ✅
v0.10.6 → v0.10.7: LLVM IR 함수 생성 (📈 적당) ✅
v0.10.7 → v0.10.8: Full Pipeline 통합 (📈 적당) ✅
v0.10.8 → v0.10.9: Unified Compiler Entry Point (📈 적당) ✅
v0.10.9 → v0.10.10: Integration Testing (📈 적당) ✅
v0.10.10 → v0.10.11: End-to-End Validation (📈 적당) ✅
v0.10.11 → v0.10.12: Text-based LLVM IR Backend (📈 적당) ✅
v0.10.12 → v0.10.13: 컴포넌트 패키지화 준비 (📈 적당) ✅
v0.10.13 → v0.10.14: stdlib/string 확장 (📈 적당) ✅
v0.10.14 → v0.10.15: stdlib/parse 모듈 (📈 적당) ✅
v0.10.15 → v0.11.0: BMB 컴파일러 완성 (📈 도전적)
v0.11.0 → v0.11.4: BMBX 번들 포맷 (📈 적당) ✅
v0.11.4 → v0.11.7: AI 패키지 탐색 + 단일 파일 번들 (📈 적당) ✅
v0.11.7 → v0.12.0: WASM IR 변환기 (📈 도전적)
v0.12.0 → v0.12.4: 듀얼 타깃 완성 (📈 적당)
v0.12.4 → v0.13.0: MCP 서버 (📈 적당)
v0.13.0 → v0.13.3: 생태계 완성 (📈 적당)
v0.13.3 → v1.0-RC: 자기 호스팅 (📈 도전적)
```

---

## 요약

```
v0.1-0.5: 기반 (파서 + 검증 + 실행 + LLVM + 언어확장) ✅
v0.6: 표준 라이브러리 기초 (100+개 함수) ✅
v0.7: 도구 기초 (fmt, lsp, test, action-bmb) ✅
v0.8: 패키지 기초 (곳간) ✅
v0.9: 생태계 (에디터, 원격 패키지, playground, site, benchmark) ✅
v0.10: Bootstrap (타입체커✅, MIR✅, LLVM IR✅, Text LLVM✅, stdlib확장✅) ✅
v0.11: AI-Native gotgan + Bootstrap 완성 (BMBX✅, 계약검색✅, 단일번들✅, Stage 2)
v0.12: WASM 듀얼 타깃 (WASI, 브라우저, 조건부 컴파일)
v0.13: 생태계 완성 (MCP 서버, 레지스트리 자동화, 품질 점수)
v1.0: 안정성 약속 + 완전한 자기 호스팅

핵심 지표:
- 계약: 10,000+
- 테스트: 5,000+
- 표준 라이브러리: 170+ 함수 (v0.10.15 기준)
- 에코시스템: 8개 레포지토리
- 벤치마크: BMB >= C -O3
- 부트스트래핑: 완료
- AI-Native: gotgan + MCP + 계약 탐색
- 듀얼 타깃: LLVM Native + WASM
```
