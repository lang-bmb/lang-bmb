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
| v0.7 | Bloom | 도구 기초 (fmt, lsp, test) | 🔄 진행중 |
| v0.8 | Fruit | 패키지 매니저 (곳간) | 계획 |
| v0.9 | Harvest | 생태계 (에디터, 원격 패키지) | 계획 |
| v0.10 | Sunrise | Bootstrap 진행 | 계획 |
| v0.11 | Dawn | Bootstrap 완성 | 계획 |
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

## v0.7 Bloom (도구 기초) 🔄 진행중

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

### v0.7.1 - LSP 기초 (계획)

```
지원 기능:
- textDocument/diagnostic    # 에러/경고 표시
- textDocument/hover         # 타입 정보 표시
- textDocument/completion    # 기본 자동완성 (키워드)
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

### v0.7.3 - action-bmb 시작

```
action-bmb/
├── action.yml        # GitHub Action 정의
├── src/              # Action 로직 (Shell/Rust)
└── examples/         # 사용 예제
```

---

## v0.8 Fruit (패키지 매니저)

> 목표: 곳간(gotgan) 최소 기능

### v0.8.0 - 프로젝트 구조

```bash
gotgan new hello      # 새 프로젝트
gotgan init           # 현재 디렉토리 초기화
```

```toml
# gotgan.toml
[package]
name = "hello"
version = "0.1.0"
edition = "2025"

[dependencies]
# BMB 패키지
json = "0.1"

# Rust fallback (crates.io)
[dependencies.rust]
serde = "1.0"
```

### v0.8.1 - 빌드 시스템

```bash
gotgan build          # 빌드
gotgan run            # 빌드 + 실행
gotgan check          # 타입 검사만
gotgan verify         # 계약 검증
gotgan test           # 테스트 실행
```

### v0.8.2 - 로컬 의존성 + Rust Fallback

```toml
[dependencies]
mylib = { path = "../mylib" }

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

### v0.9.0 - LSP 확장

```
추가 기능:
- textDocument/definition     # 정의로 이동
- textDocument/references     # 참조 찾기
- textDocument/formatting     # 포맷팅 연동
```

### v0.9.1 - tree-sitter-bmb

```
tree-sitter-bmb/
├── grammar.js        # 문법 정의
├── queries/
│   ├── highlights.scm
│   ├── folds.scm
│   └── indents.scm
└── bindings/         # Node.js, Rust 바인딩
```

### v0.9.2 - vscode-bmb

```
vscode-bmb/
├── package.json
├── syntaxes/bmb.tmLanguage.json
└── src/extension.ts  # LSP 클라이언트
```

**기능**:
- 구문 하이라이팅 (TextMate)
- 에러 표시 (LSP)
- 자동완성

### v0.9.3 - 원격 패키지 레지스트리

```bash
gotgan publish        # 패키지 배포
gotgan search json    # 검색
gotgan add json       # 의존성 추가
```

### v0.9.4 - playground

```
playground/
├── src/
│   ├── App.tsx       # React 앱
│   ├── Editor.tsx    # Monaco 에디터
│   └── wasm/         # BMB WASM
└── public/
```

**기능**:
- 온라인 편집기 (Monaco + BMB 하이라이팅)
- WASM 기반 실행
- 실시간 타입 체크 + 계약 검증
- URL 공유 링크

### v0.9.5 - lang-bmb-site

```
lang-bmb-site/
├── src/pages/
│   ├── index.astro   # Landing page
│   ├── docs/         # Documentation
│   ├── download.astro
│   ├── changes.astro
│   └── blog/
└── content/          # Markdown 콘텐츠
```

**페이지**:
- `/` - Landing page
- `/docs` - Documentation
- `/download` - 설치 가이드
- `/changes` - Changelog
- `/blog` - 개발 블로그

### v0.9.6 - benchmark-bmb

```
benchmark-bmb/
├── benches/
│   ├── compute/      # n-body, mandelbrot
│   ├── memory/       # binary-trees
│   ├── realworld/    # json-parse
│   └── contract/     # bounds-check-elim
├── runner/           # 벤치마크 러너 (Rust)
└── dashboard/        # 웹 대시보드
```

**목표**: BMB >= C -O3

| 카테고리 | 벤치마크 |
|----------|----------|
| Compute | n-body, mandelbrot, fannkuch, spectral-norm |
| Memory | binary-trees, reverse-complement |
| Real-world | json-parse, regex-redux, http-throughput |
| Contract | bounds-check-elim, null-check-elim, purity-opt |

---

## v0.10 Sunrise (Bootstrap 진행)

> 목표: BMB로 BMB 도구 재작성 시작

### v0.10.0 - 타입 체커 BMB 작성

```
bootstrap/
├── lexer.bmb       # ✅ 완료
├── parser.bmb      # ✅ 완료
├── types.bmb       # 타입 체커 (신규)
└── ...
```

### v0.10.1 - 코드 생성기 BMB 시작

```
bootstrap/
├── mir.bmb         # MIR 정의
├── codegen.bmb     # MIR → LLVM IR
└── ...
```

### v0.10.2 - 표준 라이브러리 확장 (100개)

| 모듈 | 함수 수 |
|------|---------|
| io | 20+ (파일 읽기/쓰기) |
| fs | 15+ (디렉토리 조작) |
| net | 15+ (TCP 기초) |
| time | 15+ (시간/날짜) |

### v0.10.3 - Rust→BMB 마이그레이션 도구

```bash
gotgan migrate --analyze my_crate     # Rust crate 분석
gotgan migrate --generate my_crate    # BMB 스켈레톤 생성
```

**특징**:
- 점진적 마이그레이션 (함수 단위)
- 계약 추론 (Rust 코드에서 pre/post 조건)

---

## v0.11 Dawn (Bootstrap 완성)

> 목표: Stage 2 컴파일러 + 도구 BMB 재작성

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
| gotgan | v0.8 | v0.8.2 | v0.9 | v0.11 |
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
v0.7.3 → v0.8.0: 곳간 기초 (📈 적당)
v0.8.0 → v0.8.1: 빌드 시스템 (📈 적당)
v0.8.1 → v0.8.2: 로컬 의존성 (📈 적당)
v0.8.2 → v0.9.0: LSP 확장 (📈 적당)
v0.9.x: 생태계 모듈 순차 추가 (📈 적당)
v0.10.x: Bootstrap 점진적 진행 (📈 적당)
v0.11.x: BMB 재작성 완성 (📈 적당)
```

---

## 요약

```
v0.1-0.5: 기반 (파서 + 검증 + 실행 + LLVM + 언어확장) ✅
v0.6: 표준 라이브러리 기초 (100+개 함수) ✅
v0.7: 도구 기초 (fmt, lsp, test, action-bmb)
v0.8: 패키지 기초 (곳간 + Rust fallback)
v0.9: 생태계 (에디터, 원격 패키지, playground, site, benchmark)
v0.10: Bootstrap 진행 (타입체커, 코드생성기)
v0.11: Bootstrap 완성 (Stage 2, 도구 BMB 재작성)
v1.0: 안정성 약속 + 검증 완료

핵심 지표:
- 계약: 10,000+
- 테스트: 5,000+
- 표준 라이브러리: 200+ 함수
- 에코시스템: 8개 레포지토리
- 벤치마크: BMB >= C -O3
- 부트스트래핑: 완료
```
