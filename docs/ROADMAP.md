---

# BMB 로드맵 v0.1 → v1.0-RC

---

## 개요

| 버전 | 코드명 | 목표 |
|------|--------|------|
| v0.1 | Seed | 최소 파서 + 타입체커 |
| v0.2 | Sprout | SMT 연동 + 기본 검증 |
| v0.3 | Root | 인터프리터 + REPL |
| v0.4 | Stem | 코드젠 (LLVM) |
| v0.5 | Branch | 자기 컴파일 시작 |
| v0.6 | Leaf | 표준 라이브러리 |
| v0.7 | Bloom | 패키지 매니저 (곳간) |
| v0.8 | Fruit | 도구 체인 완성 |
| v0.9 | Harvest | 핵심 패키지 200+ |
| v1.0-RC | Golden | 부트스트래핑 완료 + 검증 |

---

## 생태계 레포지토리

| 레포지토리 | 용도 | 초기 버전 | BMB 부트스트래핑 |
|------------|------|-----------|------------------|
| [lang-bmb](https://github.com/lang-bmb/lang-bmb) | 메인 컴파일러 | v0.1 (Rust) | v0.5 (BMB) |
| [bmb-samples](https://github.com/lang-bmb/bmb-samples) | 예제 프로그램 | v0.3 | N/A (BMB 코드) |
| [gotgan](https://github.com/lang-bmb/gotgan) | 패키지 매니저 | v0.7 (Rust) | v0.8 (BMB) |
| [action-bmb](https://github.com/lang-bmb/action-bmb) | GitHub Action | v0.4 | v0.8 (BMB) |
| [tree-sitter-bmb](https://github.com/lang-bmb/tree-sitter-bmb) | 에디터 문법 | v0.8 | N/A (Tree-sitter) |
| [vscode-bmb](https://github.com/lang-bmb/vscode-bmb) | VS Code 확장 | v0.8 | N/A (TypeScript) |

### 부트스트래핑 전략

```
Phase 1 (v0.1-v0.3): Rust로 기반 구축
  - 컴파일러 프론트엔드 (Rust)
  - 인터프리터/REPL (Rust)
  - bmb-samples 시작 (BMB 코드)

Phase 2 (v0.4-v0.5): 네이티브 코드 생성 + 자기 컴파일 시작
  - LLVM 백엔드 (Rust)
  - 컴파일러 BMB 재작성 시작
  - action-bmb 초기 버전 (Rust/Shell)

Phase 3 (v0.6-v0.7): 표준 라이브러리 + 패키지 매니저
  - 표준 라이브러리 (BMB)
  - gotgan 패키지 매니저 (Rust)

Phase 4 (v0.8-v0.9): 도구 체인 완성 + 부트스트래핑
  - gotgan BMB 재작성
  - action-bmb BMB 재작성
  - tree-sitter-bmb, vscode-bmb 완성
  - 핵심 패키지 200+

Phase 5 (v1.0-RC): 완전한 자기 컴파일
  - 모든 도구 BMB로 재작성 완료
  - 검증 완료
```

---

## v0.1 Seed (최소 기반)

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

### 지원 기능

```
- 기본 타입: i32, i64, f64, bool, ()
- 함수 정의
- let 바인딩
- if/else
- 기본 연산
- pre/post (파싱만)
```

### 산출물

```
bmb-0.1/
├── src/           # Rust 소스
├── tests/         # 테스트 케이스 50+
├── examples/      # 예시 10+
└── docs/          # 스펙 문서
```

### 마일스톤

- [x] 렉서 완료 ✅ (logos 기반)
- [x] 파서 완료 ✅ (lalrpop 기반)
- [x] AST 정의 완료 ✅
- [x] 타입체커 기본 완료 ✅
- [x] CLI 완료 ✅ (bmb check/parse/tokens)
- [ ] 테스트 통과 50+ (현재 7개 valid + 2개 invalid)

---

## v0.2 Sprout (검증 기반)

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

### 지원 기능

```
- pre 검증 (만족 가능성 검사)
- post 검증 (타당성 검사)
- 반례 생성 및 출력
- Z3 경로 및 타임아웃 설정
- 검증 결과 리포트
```

### 산출물

```
- SMT 변환 규칙 (IMPLEMENTATION_v0.2.md)
- smt 모듈 (translator, solver)
- verify 모듈 (contract verifier)
- 검증 테스트 케이스 5개
```

### 마일스톤

- [x] Z3 연동 완료 ✅ (외부 프로세스)
- [x] pre 검증 작동 ✅
- [x] post 검증 작동 ✅
- [x] 반례 출력 완료 ✅
- [ ] 테스트 통과 100+ (현재 6개 단위 + 5개 검증)

---

## v0.3 Root (실행 기반)

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

### 지원 기능

```
- 완전한 표현식 평가 ✅
- 재귀 함수 (depth limit: 1000) ✅
- 렉시컬 스코프 ✅
- let 바인딩 ✅
- if/else 분기 ✅
- 블록 표현식 ✅
- pre-condition 런타임 검사 ✅
```

### 산출물

```
- bmb run <file> 명령어 ✅
- bmb repl 명령어 ✅
- 내장 함수 7개 (print, println, read_int, assert, abs, min, max)
- 인터프리터 단위 테스트 16개
```

### 생태계: bmb-samples 시작

```
bmb-samples/
├── basics/          # 기본 문법 예제
├── contracts/       # 계약 검증 예제
├── algorithms/      # 알고리즘 구현
└── tutorials/       # 튜토리얼
```

### 마일스톤

- [x] 인터프리터 완료 ✅
- [x] REPL 작동 ✅
- [x] `bmb run` 작동 ✅
- [ ] bmb-samples 기본 예제 10+
- [ ] `bmb test` 작동 (v0.4+)
- [ ] 테스트 통과 200+ (현재 16개)

---

## v0.4 Stem (네이티브 기반)

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
| 디버그 정보 | ⏳ 계획 | DWARF 생성 |

### 지원 기능

```
- 네이티브 실행 파일 생성 ✅
- LLVM IR 출력 (--emit-ir) ✅
- 최적화 레벨 선택 (--release) ✅
- 플랫폼별 빌드 (Windows/Linux/macOS) ✅
- 기본 FFI (C 호출) ⏳
```

### 산출물

```
- `bmb build` 명령어 ✅
- `bmb build --release` 명령어 ✅
- `bmb build --emit-ir` 명령어 ✅
- MIR 모듈 (mir/mod.rs, mir/lower.rs)
- Codegen 모듈 (codegen/mod.rs, codegen/llvm.rs)
- Build 모듈 (build/mod.rs)
```

### 생태계: action-bmb 시작

```
action-bmb/
├── action.yml        # GitHub Action 정의
├── src/              # Action 로직 (Rust/Shell)
└── examples/         # 사용 예제
```

- CI/CD 파이프라인에서 BMB 빌드/테스트/검증 자동화
- `bmb check`, `bmb verify`, `bmb build` 지원

### LLVM 요구사항

```
LLVM 기능은 선택적입니다:
- 기본 빌드: cargo build (LLVM 없이 컴파일됨)
- LLVM 빌드: cargo build --features llvm (LLVM 18 필요)

LLVM 없이 빌드하면 `bmb build`는 "LLVM not available" 에러 반환
```

### 마일스톤

- [x] MIR 정의 완료 ✅
- [x] LLVM IR 생성 완료 ✅
- [x] 실행 파일 생성 완료 ✅
- [x] 디버그 빌드 작동 ✅
- [x] 릴리스 빌드 작동 ✅
- [ ] C FFI 작동 (v0.5)
- [ ] action-bmb v0.1 (기본 빌드/테스트)

---

## v0.5 Branch (자기 컴파일 시작)

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
| While 루프 | ✅ 완료 | 기본 반복문 |
| Range 타입 | ✅ 완료 | start..end 표현식 |
| For 루프 | ✅ 완료 | for i in range { body } |
| 모듈 시스템 | ✅ 완료 | pub 가시성, use 문 파싱 |
| 참조 타입 | ✅ 완료 | &T, &mut T 참조 |
| 배열 타입 | ✅ 완료 | [T; N] 고정 크기, 인덱스 접근 |
| 멀티 파일 | ✅ 완료 | resolver 모듈, 모듈 로딩/파싱 |
| 메서드 호출 | ✅ 완료 | expr.method(args) 지원 |
| 렉서 (BMB) | ✅ 완료 | bootstrap/lexer.bmb |
| 파서 (BMB) | ⏳ 계획 | BMB로 재작성 |

### 지원 기능 (Phase 1-9 완료)

```
Phase 1 (완료):
- struct 정의 및 생성, 필드 접근
- enum 정의 및 variant 사용
- match 기본 패턴, wildcard

Phase 2 (완료):
- 문자열 리터럴, 연결 (+), 길이 (.len())
- let mut 가변 바인딩
- 할당 연산자 (=)
- while 루프

Phase 3 (완료):
- Range 타입 (start..end)
- for 루프 (for i in range { body })

Phase 4 (완료):
- pub 가시성 수정자
- use 문 파싱 (use path::to::item;)

Phase 5 (완료):
- &T 불변 참조
- &mut T 가변 참조
- *expr 역참조

Phase 6 (완료):
- [T; N] 고정 크기 배열
- [a, b, c] 배열 리터럴
- arr[i] 인덱스 접근

Phase 7 (완료):
- 멀티 파일 컴파일 (resolver)
- 모듈 로딩/파싱 기능
- use 문 해결 및 이름 임포트

Phase 8 (완료):
- 메서드 호출 (expr.method(args))
- 문자열 메서드: len(), char_at(), slice(), is_empty()
- 배열 메서드: len()

Phase 9 (완료):
- BMB 렉서 (bootstrap/lexer.bmb)
- 순수 함수형/재귀 스타일
- 모든 BMB 토큰 인식

Phase 10 (완료):
- Short-circuit 평가 구현 (and/or)
- 왼쪽 피연산자로 결과 결정시 오른쪽 평가 생략

Phase 11 (완료):
- stacker 크레이트로 동적 스택 확장
- 128KB red zone, 4MB grow size
- MAX_RECURSION_DEPTH 10000
- BMB 렉서 15+ 토큰 처리 가능

추후 작업:
- Phase 12: BMB 파서 작성
- 코드 생성기 BMB 재작성
```

### 산출물

```
현재 완료:
- ast/mod.rs: Struct, Enum, Match, For, Range, Visibility, UseStmt
- ast/types.rs: Type::Struct, Type::Enum, Type::String, Type::Range, Type::Ref, Type::RefMut, Type::Array
- ast/expr.rs: Expr::Ref, Expr::RefMut, Expr::Deref, Expr::ArrayLit, Expr::Index, Expr::MethodCall
- grammar.lalrpop: 모든 Phase 1-8 문법
- mir/lower.rs: Struct/Enum/Match/While/For/Ref/Array/MethodCall MIR 변환
- interp/eval.rs: 모든 새 표현식 평가 (참조, 배열, 메서드 호출)
- interp/value.rs: Value::Ref (Rc<RefCell>), Value::Array
- types/mod.rs: 모든 새 타입 검사
- resolver/mod.rs: 멀티 파일 컴파일, 모듈 로딩
- codegen/llvm.rs: 문자열 리터럴 지원
- runtime/runtime.c: 문자열 런타임 함수

Bootstrap (완료):
bootstrap/
└── lexer.bmb: BMB 렉서 (순수 함수형)

예정:
bootstrap/
├── parser.bmb
├── ast.bmb
├── types.bmb
└── codegen.bmb
```

### 마일스톤

- [x] Phase 1: Struct, Enum, Match 기본 ✅
- [x] Phase 2: String, Mutable, While ✅
- [x] Phase 3: Range, For 루프 ✅
- [x] Phase 4: pub/use 모듈 시스템 ✅
- [x] Phase 5: &T, &mut T 참조 타입 ✅
- [x] Phase 6: [T; N] 배열 타입 ✅
- [x] Phase 7: 멀티 파일 컴파일 (resolver) ✅
- [x] Phase 8: 메서드 호출 (expr.method(args)) ✅
- [x] Phase 9: BMB 렉서 자기 작성 (bootstrap/lexer.bmb) ✅
- [x] Phase 10: Short-circuit 평가 (and/or) ✅
- [x] Phase 11: 스택 깊이 확장 (stacker) ✅
- [ ] Phase 12: BMB 파서 작성
- [ ] BMB 컴파일러가 자기 렉서/파서 컴파일

---

## v0.6 Leaf (표준 라이브러리)

### 목표
```
완전한 표준 라이브러리
```

### 구성요소

| 모듈 | 함수 수 | 설명 |
|------|---------|------|
| core | 50+ | 기본 타입, 연산 |
| collections | 30+ | Vec, Map, Set |
| string | 25+ | 문자열 처리 |
| io | 20+ | 파일, 스트림 |
| fs | 15+ | 파일 시스템 |
| net | 15+ | 네트워크 |
| sync | 15+ | 동기화 |
| async | 20+ | 비동기 |
| math | 30+ | 수학 함수 |
| rand | 10+ | 난수 |
| time | 15+ | 시간/날짜 |
| fmt | 15+ | 포매팅 |
| parse | 20+ | 파싱 |
| test | 10+ | 테스트 유틸 |
| bench | 10+ | 벤치마크 유틸 |

**총: 300+ 함수**

### 모든 함수 요구사항

```
- 계약 (pre/post) 명시
- 테스트 커버리지 90%+
- 문서화 100%
- 예시 코드
```

### 마일스톤

- [ ] core 완료
- [ ] collections 완료
- [ ] io/fs 완료
- [ ] 모든 함수 계약 검증
- [ ] 문서화 완료

---

## v0.7 Bloom (곳간)

### 목표
```
패키지 매니저 완성
```

### 구성요소

| 구성요소 | 상태 | 설명 |
|----------|------|------|
| 곳간 CLI | 필수 | 명령어 인터페이스 |
| 레지스트리 | 필수 | 패키지 저장소 |
| 의존성 해결 | 필수 | SAT 기반 |
| 빌드 시스템 | 필수 | 프로젝트 빌드 |
| 캐시 | 필수 | 로컬 캐시 |
| Cargo 호환 | 필수 | Rust 패키지 사용 |
| 락파일 | 필수 | 재현 가능 빌드 |
| 워크스페이스 | 기본 | 다중 패키지 |

### 명령어

```bash
gotgan new <name>        # 프로젝트 생성
gotgan init              # 현재 디렉토리 초기화
gotgan build             # 빌드
gotgan run               # 실행
gotgan test              # 테스트
gotgan bench             # 벤치마크
gotgan verify            # 계약 검증
gotgan add <pkg>         # 의존성 추가
gotgan remove <pkg>      # 의존성 제거
gotgan update            # 의존성 업데이트
gotgan publish           # 패키지 배포
gotgan search <query>    # 패키지 검색
gotgan doc               # 문서 생성
gotgan fmt               # 포매팅
gotgan lint              # 린트
gotgan clean             # 정리
```

### 생태계: gotgan 레포지토리

```
gotgan/                  # https://github.com/lang-bmb/gotgan
├── src/                 # Rust 구현 (v0.7)
├── bmb-src/             # BMB 재작성 (v0.8+)
├── registry-api/        # 레지스트리 서버
└── docs/                # 패키지 매니저 문서
```

### 곳간 부트스트래핑 전략

```
v0.7: Rust로 초기 구현 (Cargo 참고)
v0.8: BMB로 점진적 재작성 시작
v1.0: 완전한 BMB 구현
```

### 마일스톤

- [ ] CLI 완료 (Rust)
- [ ] 레지스트리 API 완료
- [ ] 의존성 해결 완료
- [ ] Cargo 호환 완료
- [ ] `gotgan build` 작동
- [ ] `gotgan publish` 작동

---

## v0.8 Fruit (도구 체인)

### 목표
```
완전한 개발 도구 체인
```

### 도구 목록

| 도구 | 명령어 | 설명 |
|------|--------|------|
| 컴파일러 | `bmb build` | 빌드 |
| 검증기 | `bmb verify` | 계약 검증 |
| 린터 | `bmb lint` | 코드 품질 |
| 포매터 | `bmb fmt` | 코드 정렬 |
| 문서 생성기 | `bmb doc` | API 문서 |
| 테스트 러너 | `bmb test` | 테스트 실행 |
| 벤치마크 러너 | `bmb bench` | 성능 측정 |
| REPL | `bmb repl` | 대화형 |
| LSP 서버 | `bmb lsp` | IDE 지원 |
| 디버거 | `bmb debug` | 디버깅 |
| 프로파일러 | `bmb profile` | 성능 분석 |
| 커버리지 | `bmb coverage` | 커버리지 분석 |
| WASM 빌드 | `bmb wasm` | WebAssembly |

### IDE 지원

| IDE | 지원 |
|-----|------|
| VS Code | 확장 필수 |
| Neovim | LSP 설정 |
| JetBrains | 플러그인 기본 |
| Zed | 플러그인 기본 |

### 웹 인프라

| 구성요소 | 설명 |
|----------|------|
| bmb-lang.org | 공식 홈페이지 |
| docs.bmb-lang.org | 문서 |
| play.bmb-lang.org | 플레이그라운드 |
| registry.bmb-lang.org | 패키지 레지스트리 |
| bench.bmb-lang.org | 벤치마크 대시보드 |

### 플레이그라운드 기능

```
- 온라인 편집기
- 실시간 타입 체크
- 실시간 계약 검증
- 실행 결과 표시
- 공유 링크
- 예시 불러오기
- WASM 기반 실행
```

### 생태계 레포지토리

#### tree-sitter-bmb
```
tree-sitter-bmb/         # https://github.com/lang-bmb/tree-sitter-bmb
├── grammar.js           # Tree-sitter 문법 정의
├── src/                 # 생성된 파서
├── queries/             # 하이라이팅, 폴딩 쿼리
└── bindings/            # 언어별 바인딩
```

#### vscode-bmb
```
vscode-bmb/              # https://github.com/lang-bmb/vscode-bmb
├── package.json         # 확장 매니페스트
├── src/                 # TypeScript 소스
│   ├── extension.ts     # 진입점
│   ├── lsp-client.ts    # LSP 클라이언트
│   └── debugger.ts      # 디버거 어댑터
├── syntaxes/            # TextMate 문법
└── snippets/            # 코드 스니펫
```

### BMB 부트스트래핑 (v0.8)

| 도구 | v0.7 구현 | v0.8 재작성 | 상태 |
|------|-----------|-------------|------|
| gotgan | Rust | BMB | 진행 중 |
| action-bmb | Shell/Rust | BMB | 진행 중 |
| LSP 서버 | Rust | BMB | 계획 |
| 포매터 | Rust | BMB | 계획 |
| 린터 | Rust | BMB | 계획 |

### 마일스톤

- [ ] LSP 서버 완료
- [ ] VS Code 확장 완료 (vscode-bmb)
- [ ] tree-sitter-bmb 문법 완료
- [ ] 포매터 완료
- [ ] 린터 완료 (규칙 50+)
- [ ] 문서 생성기 완료
- [ ] 플레이그라운드 완료
- [ ] 홈페이지 완료
- [ ] gotgan BMB 재작성 시작
- [ ] action-bmb BMB 재작성

---

## v0.9 Harvest (생태계)

### 목표
```
핵심 패키지 200+ 완성
```

### 패키지 카테고리

| 카테고리 | 패키지 수 | 예시 |
|----------|-----------|------|
| 데이터 구조 | 20 | btree, trie, graph |
| 알고리즘 | 20 | sort, search, compress |
| 직렬화 | 15 | json, msgpack, protobuf |
| 암호화 | 15 | sha, aes, rsa |
| 네트워크 | 20 | http, websocket, grpc |
| 데이터베이스 | 15 | sqlite, postgres, redis |
| 파싱 | 15 | regex, parser-combinator |
| CLI | 10 | args, terminal |
| 로깅 | 5 | log, tracing |
| 테스트 | 10 | mock, property-test |
| 비동기 | 15 | runtime, channel |
| FFI | 10 | c-ffi, python-ffi |
| 수학 | 15 | linear-algebra, stats |
| 이미지 | 10 | png, jpeg, webp |
| 오디오 | 5 | wav, mp3 |
| 날짜/시간 | 5 | chrono, timezone |
| 국제화 | 5 | i18n, locale |
| 기타 | 10 | uuid, url, mime |

**총: 200+ 패키지**

### 패키지 요구사항

```
모든 패키지:
- 계약 완전 명시
- 테스트 커버리지 80%+
- 문서화 100%
- 예시 코드
- 벤치마크
- 라이선스 (MIT/Apache 2.0)
```

### 샘플 프로젝트

| 프로젝트 | 설명 | 복잡도 |
|----------|------|--------|
| hello | Hello World | 최소 |
| calculator | 계산기 | 기본 |
| todo-cli | 할일 관리 | 기본 |
| json-parser | JSON 파서 | 중간 |
| http-server | HTTP 서버 | 중간 |
| chat-server | 채팅 서버 | 중간 |
| markdown | 마크다운 파서 | 중간 |
| regex | 정규식 엔진 | 고급 |
| sqlite | SQLite 클라이언트 | 고급 |
| compiler | 미니 언어 컴파일러 | 고급 |
| game-of-life | 콘웨이 생명 게임 | 중간 |
| ray-tracer | 레이트레이서 | 고급 |
| neural-net | 신경망 | 고급 |
| blockchain | 블록체인 | 고급 |
| database | 키-값 저장소 | 고급 |

### 벤치마크 스위트

| 벤치마크 | 설명 | 비교 대상 |
|----------|------|-----------|
| n-body | N체 시뮬레이션 | C, Rust |
| binary-trees | 트리 할당/해제 | C, Rust |
| fannkuch | 순열 | C, Rust |
| spectral-norm | 행렬 연산 | C, Rust |
| mandelbrot | 프랙탈 (SIMD) | C, Rust |
| regex-redux | 정규식 | C, Rust |
| fasta | 시퀀스 생성 | C, Rust |
| pidigits | 임의 정밀도 | C, Rust |
| json-parse | JSON 파싱 | C, Rust |
| http-throughput | HTTP 처리량 | C, Rust |

### 벤치마크 인프라

| 구성요소 | 설명 |
|----------|------|
| 벤치마크 러너 | 자동 실행 |
| 결과 수집기 | 데이터 저장 |
| 대시보드 | 시각화 |
| 회귀 감지 | 성능 저하 알림 |
| CI 연동 | 자동 실행 |

### 마일스톤

- [ ] 패키지 200+ 완료
- [ ] 모든 패키지 계약 검증
- [ ] 샘플 프로젝트 15+ 완료
- [ ] 벤치마크 스위트 완료
- [ ] 벤치마크 대시보드 완료
- [ ] BMB >= C -O3 달성

---

## v1.0-RC Golden (부트스트래핑 완료)

### 목표
```
완전한 자기 컴파일 + 검증
```

### 부트스트래핑 체크리스트

| 구성요소 | Rust 버전 | BMB 버전 | 검증 |
|----------|-----------|----------|------|
| 렉서 | ✅ | ✅ | ✅ |
| 파서 | ✅ | ✅ | ✅ |
| AST | ✅ | ✅ | ✅ |
| 타입체커 | ✅ | ✅ | ✅ |
| SMT 변환기 | ✅ | ✅ | ✅ |
| MIR | ✅ | ✅ | ✅ |
| LLVM IR 생성 | ✅ | ✅ | ✅ |
| 최적화 패스 | ✅ | ✅ | ✅ |
| 링커 | ✅ | ✅ | ✅ |
| CLI | ✅ | ✅ | ✅ |
| 곳간 | ✅ | ✅ | ✅ |
| 표준 라이브러리 | - | ✅ | ✅ |
| 린터 | ✅ | ✅ | ✅ |
| 포매터 | ✅ | ✅ | ✅ |
| 문서 생성기 | ✅ | ✅ | ✅ |
| LSP 서버 | ✅ | ✅ | ✅ |
| 테스트 러너 | ✅ | ✅ | ✅ |
| 벤치마크 러너 | ✅ | ✅ | ✅ |

### 자기 컴파일 검증

```bash
# 1단계: Rust 컴파일러로 BMB 컴파일러 빌드
$ cargo build --release
$ ./bmb-rust build bmb-compiler -> bmb-stage1

# 2단계: Stage1으로 BMB 컴파일러 빌드
$ ./bmb-stage1 build bmb-compiler -> bmb-stage2

# 3단계: Stage2로 BMB 컴파일러 빌드
$ ./bmb-stage2 build bmb-compiler -> bmb-stage3

# 검증: Stage2와 Stage3 바이너리 동일
$ diff bmb-stage2 bmb-stage3
# 차이 없음 = 부트스트래핑 성공
```

### 전체 검증 매트릭스

| 검증 항목 | 방법 | 기준 |
|-----------|------|------|
| 컴파일러 정확성 | 자기 컴파일 | Stage2 == Stage3 |
| 계약 검증 | SMT | 모든 계약 Verified |
| 테스트 | 테스트 스위트 | 100% 통과 |
| 벤치마크 | 벤치마크 스위트 | BMB >= C |
| 메모리 안전 | Miri/Valgrind | 에러 0 |
| 스레드 안전 | ThreadSanitizer | 에러 0 |
| 퍼징 | AFL/libFuzzer | 크래시 0 |

### 문서 완료

| 문서 | 상태 |
|------|------|
| 언어 명세서 | 완료 |
| 튜토리얼 | 완료 |
| API 레퍼런스 | 완료 (자동 생성) |
| 내부 설계 문서 | 완료 |
| 기여 가이드 | 완료 |
| 변경 로그 | 완료 |

### 릴리스 체크리스트

- [ ] 부트스트래핑 3단계 통과
- [ ] 모든 계약 검증됨 (10,000+ 계약)
- [ ] 테스트 100% 통과 (5,000+ 테스트)
- [ ] 벤치마크 목표 달성 (BMB >= C)
- [ ] 메모리/스레드 안전 검증
- [ ] 퍼징 48시간 크래시 없음
- [ ] 문서 100% 완료
- [ ] 플레이그라운드 작동
- [ ] 패키지 레지스트리 작동
- [ ] VS Code 확장 배포
- [ ] 홈페이지 배포
- [ ] 릴리스 노트 작성

---

## 타임라인 (예상)

| 버전 | 기간 | 누적 |
|------|------|------|
| v0.1 | 2개월 | 2개월 |
| v0.2 | 2개월 | 4개월 |
| v0.3 | 2개월 | 6개월 |
| v0.4 | 3개월 | 9개월 |
| v0.5 | 3개월 | 12개월 |
| v0.6 | 3개월 | 15개월 |
| v0.7 | 3개월 | 18개월 |
| v0.8 | 3개월 | 21개월 |
| v0.9 | 4개월 | 25개월 |
| v1.0-RC | 3개월 | 28개월 |

**총: 약 2년 4개월**

---

## 팀 구성 (이상적)

| 역할 | 인원 | 담당 |
|------|------|------|
| 컴파일러 | 2-3 | 프론트엔드, 백엔드 |
| 검증 | 1-2 | SMT, 타입 시스템 |
| 표준 라이브러리 | 2-3 | 라이브러리 구현 |
| 도구 | 1-2 | LSP, 린터, 포매터 |
| 인프라 | 1-2 | CI, 웹, 레지스트리 |
| 문서 | 1 | 문서, 튜토리얼 |
| 커뮤니티 | 1 | 패키지, 샘플 |

**총: 10-15명**

---

## 요약

```
v0.1-0.3: 기반 (파서 + 검증 + 실행)
v0.4-0.5: 네이티브 (LLVM + 자기 컴파일 시작)
v0.6-0.7: 생태계 (표준 라이브러리 + 패키지 매니저)
v0.8-0.9: 완성 (도구 + 패키지 200+)
v1.0-RC: 검증 (부트스트래핑 + 전체 검증)

핵심 지표:
- 계약: 10,000+
- 테스트: 5,000+
- 패키지: 200+
- 벤치마크: BMB >= C
- 부트스트래핑: 완료
```